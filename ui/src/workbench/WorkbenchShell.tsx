import { useCallback, useEffect, useRef, useState, type ReactNode } from "react";
import {
  Panel,
  PanelGroup,
  PanelResizeHandle,
  type ImperativePanelHandle,
} from "react-resizable-panels";
import { open } from "@tauri-apps/plugin-dialog";
import { ErrorBoundary, Icon, StatusBar, ToastViewport, isTauri, useToast } from "../shell";
import { formatIpcError } from "../lib/agent";
import { scheduleProblemsRun, onProblemsRunRequested, problemsRun } from "../lib/problems";
import { faChevronUp, faTerminal } from "../lib/fontawesome";
import { type ActivityId } from "./activity";
import { ActivityBar } from "./ActivityBar";
import { AgentPanel } from "./AgentPanel";
import { BottomPanel } from "./BottomPanel";
import { EditorArea } from "./EditorArea";
import { MenuBar } from "./MenuBar";
import { Sidebar } from "./Sidebar";
import { WorkbenchProvider, useWorkbench } from "./state";
import { AgentSettingsProvider, useAgentSettings } from "./agentSettings";
import { EditorCommandsProvider } from "./editorCommands";
import { EditorNavigationProvider } from "./editorNavigation";
import { SearchPanelFocusProvider, useSearchPanelFocus } from "./searchPanelFocus";
import { IssuesModalsProvider } from "./issues/issuesActions";

type WorkbenchShellProps = {
  statusLeft?: ReactNode;
  statusRight?: ReactNode;
};

/**
 * VS Code / Cursor workbench grid — not the ribbon {@code AppShell}.
 *
 * ```text
 * Menu bar (title + File/Git + window controls)
 * [Activity 48px][Sidebar][Editor + Bottom][AI panel]
 * Status bar
 * ```
 */
export function WorkbenchShell(props: WorkbenchShellProps) {
  return (
    <WorkbenchProvider>
      <AgentSettingsProvider>
        <EditorCommandsProvider>
          <EditorNavigationProvider>
            <SearchPanelFocusProvider>
              <WorkbenchBody {...props} />
            </SearchPanelFocusProvider>
          </EditorNavigationProvider>
        </EditorCommandsProvider>
      </AgentSettingsProvider>
    </WorkbenchProvider>
  );
}

function WorkbenchBody({ statusLeft, statusRight }: WorkbenchShellProps) {
  const [activity, setActivity] = useState<ActivityId>("explorer");
  const { workspace, openWorkspace } = useWorkbench();
  const { settings } = useAgentSettings();
  const { requestFocus } = useSearchPanelFocus();
  const toast = useToast();
  const bottomPanelRef = useRef<ImperativePanelHandle>(null);
  const sidebarPanelRef = useRef<ImperativePanelHandle>(null);
  const [bottomCollapsed, setBottomCollapsed] = useState(false);
  const [sidebarCollapsed, setSidebarCollapsed] = useState(false);

  const toggleBottomPanel = useCallback(() => {
    const panel = bottomPanelRef.current;
    if (!panel) {
      return;
    }
    if (panel.isCollapsed()) {
      panel.expand();
    } else {
      panel.collapse();
    }
  }, []);

  const toggleSidebarPanel = useCallback(() => {
    const panel = sidebarPanelRef.current;
    if (!panel) {
      return;
    }
    if (panel.isCollapsed()) {
      panel.expand();
    } else {
      panel.collapse();
    }
  }, []);

  // VS Code behavior: clicking the active activity toggles the sidebar; clicking
  // a different one switches to it and ensures the sidebar is expanded.
  const handleSelectActivity = useCallback(
    (id: ActivityId) => {
      const panel = sidebarPanelRef.current;
      if (id === activity) {
        panel?.[panel.isCollapsed() ? "expand" : "collapse"]();
        return;
      }
      setActivity(id);
      if (panel?.isCollapsed()) {
        panel.expand();
      }
    },
    [activity],
  );

  const handleOpenFolder = useCallback(async () => {
    if (!isTauri()) {
      toast.info("Open Folder requires the desktop app");
      return;
    }
    try {
      const selected = await open({
        directory: true,
        multiple: false,
        title: "Open folder",
      });
      if (typeof selected === "string") {
        await openWorkspace(selected);
      }
    } catch (error) {
      toast.error(formatIpcError(error));
    }
  }, [openWorkspace, toast]);

  const handleOpenSettings = useCallback(() => {
    setActivity("agent");
    const panel = sidebarPanelRef.current;
    if (panel?.isCollapsed()) {
      panel.expand();
    }
  }, []);

  const focusIssues = useCallback(() => {
    setActivity("issues");
    const panel = sidebarPanelRef.current;
    if (panel?.isCollapsed()) {
      panel.expand();
    }
  }, []);

  const focusSearch = useCallback(
    (target: "find" | "replace") => {
      setActivity("search");
      const panel = sidebarPanelRef.current;
      if (panel?.isCollapsed()) {
        panel.expand();
      }
      requestFocus(target);
    },
    [requestFocus],
  );

  useEffect(() => {
    const onKeyDown = (event: KeyboardEvent) => {
      if (!(event.ctrlKey || event.metaKey) || !event.shiftKey || event.altKey) {
        return;
      }
      const key = event.key.toLowerCase();
      if (key === "f") {
        event.preventDefault();
        focusSearch("find");
        return;
      }
      if (key === "h") {
        event.preventDefault();
        focusSearch("replace");
      }
    };
    window.addEventListener("keydown", onKeyDown);
    return () => window.removeEventListener("keydown", onKeyDown);
  }, [focusSearch]);

  useEffect(() => {
    if (workspace && isTauri()) {
      scheduleProblemsRun(800);
    }
  }, [workspace]);

  useEffect(() => {
    if (!isTauri()) {
      return;
    }
    return onProblemsRunRequested(() => {
      void problemsRun().catch(() => undefined);
    });
  }, []);

  const left = workspace ? (
    <span className="text-nest-foreground/80">{workspace.name}</span>
  ) : (
    statusLeft ?? <span className="text-nest-muted">No folder opened</span>
  );

  return (
    <IssuesModalsProvider>
      <div className="flex h-full min-h-0 flex-col bg-nest-background">
        <MenuBar
          title={workspace?.name ?? "Kiwi"}
          model={settings.model}
          onOpenFolder={() => void handleOpenFolder()}
          onOpenSettings={handleOpenSettings}
          onFocusIssues={focusIssues}
          onFind={() => focusSearch("find")}
          onReplace={() => focusSearch("replace")}
        />
      <div className="flex min-h-0 flex-1">
        <ActivityBar active={sidebarCollapsed ? null : activity} onSelect={handleSelectActivity} />
        <PanelGroup direction="horizontal" className="min-w-0 flex-1">
          <Panel
            ref={sidebarPanelRef}
            collapsible
            collapsedSize={0}
            defaultSize={21}
            minSize={14}
            maxSize={32}
            id="kiwi-sidebar"
            onCollapse={() => setSidebarCollapsed(true)}
            onExpand={() => setSidebarCollapsed(false)}
          >
            <ErrorBoundary label="sidebar">
              <Sidebar activity={activity} onToggleCollapse={toggleSidebarPanel} />
            </ErrorBoundary>
          </Panel>
          <PanelResizeHandle
            className={[
              "w-px bg-nest-border hover:bg-nest-accent/60 data-[resize-handle-active]:bg-nest-accent",
              sidebarCollapsed ? "pointer-events-none opacity-0" : "",
            ].join(" ")}
          />
          <Panel minSize={30} id="kiwi-center">
            <div className="flex h-full min-h-0 flex-col">
              <PanelGroup direction="vertical" className="min-h-0 flex-1">
                <Panel defaultSize={73} minSize={35} id="kiwi-editor">
                  <ErrorBoundary label="editor">
                    <EditorArea />
                  </ErrorBoundary>
                </Panel>
                <PanelResizeHandle
                  className={[
                    "h-px bg-nest-border hover:bg-nest-accent/60 data-[resize-handle-active]:bg-nest-accent",
                    bottomCollapsed ? "pointer-events-none opacity-0" : "",
                  ].join(" ")}
                />
                <Panel
                  ref={bottomPanelRef}
                  collapsible
                  collapsedSize={0}
                  defaultSize={27}
                  minSize={12}
                  id="kiwi-bottom"
                  onCollapse={() => setBottomCollapsed(true)}
                  onExpand={() => setBottomCollapsed(false)}
                >
                  <ErrorBoundary label="bottom-panel">
                    <BottomPanel onToggleCollapse={toggleBottomPanel} />
                  </ErrorBoundary>
                </Panel>
              </PanelGroup>
              {bottomCollapsed ? (
                <button
                  type="button"
                  onClick={toggleBottomPanel}
                  title="Show panel"
                  className="flex h-8 shrink-0 items-center gap-2 border-t border-nest-border bg-nest-surface px-3 text-[11px] uppercase tracking-wide text-nest-muted hover:text-nest-foreground"
                >
                  <Icon icon={faTerminal} className="size-3" />
                  <span className="font-medium">Terminal</span>
                  <Icon icon={faChevronUp} className="ml-auto size-3" />
                </button>
              ) : null}
            </div>
          </Panel>
          <PanelResizeHandle className="w-px bg-nest-border hover:bg-nest-accent/60 data-[resize-handle-active]:bg-nest-accent" />
          <Panel defaultSize={29} minSize={18} id="kiwi-ai">
            <ErrorBoundary label="agent-panel">
              <AgentPanel />
            </ErrorBoundary>
          </Panel>
        </PanelGroup>
      </div>
      <StatusBar left={left} right={statusRight} />
      <ToastViewport />
      </div>
    </IssuesModalsProvider>
  );
}
