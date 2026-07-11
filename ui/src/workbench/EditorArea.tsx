import { Icon } from "../shell";
import { faCircle, faFile, faXmark } from "../lib/fontawesome";
import { MonacoEditor } from "./MonacoEditor";
import { IssueDetailView } from "./issues/IssueDetailView";
import { CommitChangesView } from "./CommitChangesView";
import { TaskDetailView } from "./tasks/TaskDetailView";
import { DocView } from "./DocView";
import { ComponentDetailView } from "./ComponentDetailView";
import { ThemeDetailView } from "./ThemeDetailView";
import { SettingsDetailView } from "./SettingsDetailView";
import { FetchSourceView } from "./FetchSourceView";
import { DocSourcesSettingsView } from "./DocSourcesSettingsView";
import { KiwiConfigSettingsView } from "./KiwiConfigSettingsView";
import { useWorkbench } from "./state";
import type { ThemeDefinition } from "../lib/themes";

export function EditorArea() {
  const { tabs, activePath, activeThemeId, openTheme, focusTab, closeTab, updateTabContent, saveTab } =
    useWorkbench();
  const active = tabs.find((tab) => tab.relPath === activePath) ?? null;

  if (tabs.length === 0) {
    return (
      <div className="flex h-full min-h-0 flex-col items-center justify-center bg-nest-background text-nest-muted">
        <Icon icon={faFile} className="size-8 opacity-30" />
        <p className="mt-3 text-sm">Select a file in the Explorer to open it.</p>
        <p className="mt-1 text-xs opacity-70">Edits save with Ctrl/Cmd+S.</p>
      </div>
    );
  }

  return (
    <div className="flex h-full min-h-0 flex-col bg-nest-background">
      <div className="flex h-9 shrink-0 items-stretch overflow-x-auto border-b border-nest-border">
        {tabs.map((tab) => {
          const selected = tab.relPath === activePath;
          return (
            <div
              key={tab.relPath}
              className={[
                "group flex items-center gap-2 border-r border-nest-border px-3 text-[13px]",
                selected
                  ? "bg-nest-background text-nest-foreground"
                  : "bg-nest-surface text-nest-muted hover:text-nest-foreground",
              ].join(" ")}
            >
              <button
                type="button"
                onClick={() => focusTab(tab.relPath)}
                title={tab.relPath}
                className="max-w-[16rem] truncate"
              >
                {tab.name}
              </button>
              <span className="relative flex size-4 items-center justify-center">
                {tab.dirty ? (
                  <Icon
                    icon={faCircle}
                    className="size-2 text-nest-foreground group-hover:hidden"
                    title="Unsaved changes"
                  />
                ) : null}
                <button
                  type="button"
                  onClick={() => closeTab(tab.relPath)}
                  title="Close"
                  aria-label={`Close ${tab.name}`}
                  className={[
                    "flex size-4 items-center justify-center rounded-nest-sm text-nest-muted hover:bg-nest-muted/20 hover:text-nest-foreground",
                    tab.dirty
                      ? "hidden group-hover:flex"
                      : "opacity-0 group-hover:opacity-100",
                  ].join(" ")}
                >
                  <Icon icon={faXmark} className="size-2.5" />
                </button>
              </span>
            </div>
          );
        })}
      </div>
      <div className="min-h-0 flex-1 overflow-hidden">
        {active ? (
          active.loading ? (
            <p className="p-3 text-xs text-nest-muted">Loading {active.name}…</p>
          ) : active.error ? (
            <p className="p-3 text-xs text-nest-error">{active.error}</p>
          ) : active.isFetchSource ? (
            <FetchSourceView />
          ) : active.gitCommitChanges ? (
            <CommitChangesView changes={active.gitCommitChanges} />
          ) : active.githubIssue ? (
            <IssueDetailView issue={active.githubIssue} />
          ) : active.swiftTaskDetail ? (
            <TaskDetailView detail={active.swiftTaskDetail} />
          ) : active.docContent != null ? (
            <DocView content={active.docContent} />
          ) : active.componentId != null ? (
            <ComponentDetailView componentId={active.componentId} />
          ) : active.settingsItemId === "doc-sources" ? (
            <DocSourcesSettingsView />
          ) : active.settingsItemId === "kiwi-config" ? (
            <KiwiConfigSettingsView />
          ) : active.settingsItemId != null ? (
            <SettingsDetailView itemId={active.settingsItemId} />
          ) : active.themeData ? (
            <ThemeTabContent theme={active.themeData} activeThemeId={activeThemeId} onApply={openTheme} />
          ) : (
            <MonacoEditor tab={active} onChange={updateTabContent} onSave={saveTab} />
          )
        ) : null}
      </div>
    </div>
  );
}

function ThemeTabContent({
  theme,
  activeThemeId,
  onApply,
}: {
  theme: ThemeDefinition;
  activeThemeId: string | null;
  onApply: (theme: ThemeDefinition) => void;
}) {
  return (
    <ThemeDetailView
      theme={theme}
      isActive={activeThemeId === theme.id}
      onApply={() => onApply(theme)}
    />
  );
}
