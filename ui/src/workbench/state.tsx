import {
  createContext,
  useCallback,
  useContext,
  useEffect,
  useMemo,
  useRef,
  useState,
  type ReactNode,
} from "react";
import { formatIpcError } from "../lib/agent";
import { klog } from "../lib/log";
import { applyThemeRootBlock, fetchThemeCss } from "../lib/nest";
import { isTauri, useToast } from "../shell";
import {
  openWorkspace as openWorkspaceIpc,
  readTextFile,
  workspaceInfo,
  writeTextFile,
  type WorkspaceInfo,
} from "../lib/workspace";
import { scheduleProblemsRun } from "../lib/problems";
import { commitTabKey, isCommitTab, type GitCommitChanges } from "../lib/git";
import { componentTabKey, isComponentTab, type ComponentDef } from "../lib/componentsLibrary";
import { docsRead, docTabKey, isDocTab, type DocEntry } from "../lib/docs";
import { issueTabKey, isIssueTab, type GitHubIssue } from "../lib/github";
import { isTaskTab, swiftGetTask, taskTabKey, type SwiftTaskDetailResponse } from "../lib/swift";
import { isThemeTab, themeSetActive, themeTabKey, type ThemeDefinition } from "../lib/themes";
import { settingsTabKey, type SettingsItem } from "../lib/settings";

/** An open editor tab backed by a workspace file. */
export type EditorTab = {
  /** Path relative to the project root. */
  relPath: string;
  /** Display name (file basename). */
  name: string;
  /** Current editor contents. */
  content: string;
  /** Contents last read from / written to disk (baseline for the dirty flag). */
  savedContent: string;
  /** True when `content` differs from `savedContent`. */
  dirty: boolean;
  /** True while the read IPC is in flight. */
  loading: boolean;
  /** True while a save IPC is in flight. */
  saving: boolean;
  /** Read/save error message, if the file could not be opened or written. */
  error: string | null;
  /** Full GitHub issue metadata when this tab is a virtual issue tab. */
  githubIssue?: GitHubIssue | null;
  /** Commit diff payload when this tab is a virtual commit tab. */
  gitCommitChanges?: GitCommitChanges | null;
  /** Swift task detail when this tab is a virtual task tab. */
  swiftTaskDetail?: SwiftTaskDetailResponse | null;
  /** Rendered Markdown when this tab is a virtual Help doc tab. */
  docContent?: string | null;
  /** Component id when this tab is a virtual @nest/components viewer tab. */
  componentId?: string | null;
  /** Full theme definition when this tab is a virtual Theme viewer tab. */
  themeData?: ThemeDefinition | null;
  /** Settings item id when this tab is a virtual Settings detail tab. */
  settingsItemId?: string | null;
};

type WorkbenchContextValue = {
  /** Active workspace, or null before load / outside the desktop host. */
  workspace: WorkspaceInfo | null;
  /** Monotonically increasing token; bump to force Explorer re-listing. */
  refreshToken: number;
  /** Open editor tabs, in order. */
  tabs: EditorTab[];
  /** Relative path of the focused tab, if any. */
  activePath: string | null;
  /** Opens (or focuses) a file tab and lazily loads its contents. */
  openFile: (relPath: string, name: string) => void;
  /** Opens (or focuses) a GitHub issue tab with metadata and body. */
  openIssue: (issue: GitHubIssue) => void;
  /** Opens (or focuses) a commit changes tab. */
  openCommit: (changes: GitCommitChanges) => void;
  /** Opens (or focuses) a Swift task detail tab. */
  openTask: (taskId: string, title: string) => void;
  /** Opens (or focuses) a Help doc tab and lazily loads its Markdown. */
  openDoc: (projectId: string, entry: DocEntry) => void;
  /** Opens (or focuses) a @nest/components viewer tab. */
  openComponent: (def: ComponentDef) => void;
  /** Opens (or focuses) a Theme viewer tab and applies it as the active theme. */
  openTheme: (theme: ThemeDefinition) => void;
  /** Opens (or focuses) a Settings detail tab. */
  openSetting: (item: SettingsItem) => void;
  /** The currently active theme's id, once known. */
  activeThemeId: string | null;
  /** Focuses an already-open tab. */
  focusTab: (relPath: string) => void;
  /** Updates a tab's in-editor content (marks it dirty). */
  updateTabContent: (relPath: string, content: string) => void;
  /** Writes a tab's content to disk and clears its dirty flag. */
  saveTab: (relPath: string) => Promise<void>;
  /** Closes a tab. */
  closeTab: (relPath: string) => void;
  /** Switches the workspace root and clears open tabs. */
  openWorkspace: (root: string) => Promise<void>;
  /** Re-lists the Explorer tree from the current root. */
  refreshWorkspace: () => void;
};

const WorkbenchContext = createContext<WorkbenchContextValue | null>(null);

export function WorkbenchProvider({ children }: { children: ReactNode }) {
  const [workspace, setWorkspace] = useState<WorkspaceInfo | null>(null);
  const [tabs, setTabs] = useState<EditorTab[]>([]);
  const [activePath, setActivePath] = useState<string | null>(null);
  const [refreshToken, setRefreshToken] = useState(0);
  const [activeThemeId, setActiveThemeId] = useState<string | null>(null);
  const toast = useToast();
  const tabsRef = useRef<EditorTab[]>([]);
  useEffect(() => {
    tabsRef.current = tabs;
  }, [tabs]);

  useEffect(() => {
    if (!isTauri()) {
      return;
    }
    void workspaceInfo()
      .then(setWorkspace)
      .catch((error) => toast.error(formatIpcError(error)));
  }, [toast]);

  useEffect(() => {
    if (!isTauri()) {
      return;
    }
    void fetchThemeCss()
      .then((css) => setActiveThemeId(css.id))
      .catch(() => {
        // App.tsx already surfaces theme-load failures on startup.
      });
  }, []);

  const focusTab = useCallback((relPath: string) => {
    setActivePath(relPath);
  }, []);

  const updateTabContent = useCallback((relPath: string, content: string) => {
    setTabs((current) =>
      current.map((tab) =>
        tab.relPath === relPath
          ? { ...tab, content, dirty: content !== tab.savedContent }
          : tab,
      ),
    );
  }, []);

  const saveTab = useCallback(
    async (relPath: string) => {
      if (isIssueTab(relPath) || isCommitTab(relPath)) {
        return;
      }
      if (
        isTaskTab(relPath) ||
        isDocTab(relPath) ||
        isComponentTab(relPath) ||
        isThemeTab(relPath)
      ) {
        return;
      }
      const tab = tabsRef.current.find((entry) => entry.relPath === relPath);
      if (!tab || tab.loading || tab.saving || !tab.dirty) {
        return;
      }
      const content = tab.content;
      klog("workbench", `saveTab rel=${relPath} bytes=${content.length}`);
      setTabs((current) =>
        current.map((entry) =>
          entry.relPath === relPath ? { ...entry, saving: true, error: null } : entry,
        ),
      );
      try {
        await writeTextFile(relPath, content);
        setTabs((current) =>
          current.map((entry) =>
            entry.relPath === relPath
              ? {
                  ...entry,
                  savedContent: content,
                  dirty: entry.content !== content,
                  saving: false,
                  error: null,
                }
              : entry,
          ),
        );
        scheduleProblemsRun();
      } catch (error) {
        const message = formatIpcError(error);
        setTabs((current) =>
          current.map((entry) =>
            entry.relPath === relPath ? { ...entry, saving: false, error: message } : entry,
          ),
        );
        toast.error(message);
      }
    },
    [toast],
  );

  const openFile = useCallback(
    (relPath: string, name: string) => {
      klog("workbench", `openFile rel=${relPath}`);
      setActivePath(relPath);
      setTabs((current) => {
        const existing = current.find((tab) => tab.relPath === relPath);
        if (existing) {
          if (!existing.loading && !existing.error) {
            return current;
          }
          return current.map((tab) =>
            tab.relPath === relPath
              ? {
                  ...tab,
                  name,
                  loading: true,
                  saving: false,
                  error: null,
                }
              : tab,
          );
        }
        return [
          ...current,
          {
            relPath,
            name,
            content: "",
            savedContent: "",
            dirty: false,
            loading: true,
            saving: false,
            error: null,
          },
        ];
      });

      void readTextFile(relPath)
        .then((file) => {
          setTabs((current) =>
            current.map((tab) =>
              tab.relPath === relPath
                ? {
                    ...tab,
                    content: file.content,
                    savedContent: file.content,
                    dirty: false,
                    loading: false,
                    error: null,
                  }
                : tab,
            ),
          );
        })
        .catch((error) => {
          const message = formatIpcError(error);
          setTabs((current) =>
            current.map((tab) =>
              tab.relPath === relPath
                ? { ...tab, loading: false, error: message }
                : tab,
            ),
          );
        });
    },
    [],
  );

  const openIssue = useCallback((issue: GitHubIssue) => {
    const relPath = issueTabKey(issue.number);
    const name = `#${issue.number} ${issue.title}`;
    klog("workbench", `openIssue number=${issue.number}`);
    setActivePath(relPath);
    setTabs((current) => {
      const existing = current.find((tab) => tab.relPath === relPath);
      if (existing) {
        return current.map((tab) =>
          tab.relPath === relPath
            ? {
                ...tab,
                name,
                content: issue.body,
                savedContent: issue.body,
                dirty: false,
                loading: false,
                error: null,
                githubIssue: issue,
              }
            : tab,
        );
      }
      return [
        ...current,
        {
          relPath,
          name,
          content: issue.body,
          savedContent: issue.body,
          dirty: false,
          loading: false,
          saving: false,
          error: null,
          githubIssue: issue,
        },
      ];
    });
  }, []);

  const openCommit = useCallback((changes: GitCommitChanges) => {
    const relPath = commitTabKey(changes.hash);
    const name = `${changes.shortHash} ${changes.subject}`;
    klog("workbench", `openCommit hash=${changes.shortHash}`);
    setActivePath(relPath);
    setTabs((current) => {
      const existing = current.find((tab) => tab.relPath === relPath);
      if (existing) {
        return current.map((tab) =>
          tab.relPath === relPath
            ? {
                ...tab,
                name,
                content: "",
                savedContent: "",
                dirty: false,
                loading: false,
                error: null,
                gitCommitChanges: changes,
              }
            : tab,
        );
      }
      return [
        ...current,
        {
          relPath,
          name,
          content: "",
          savedContent: "",
          dirty: false,
          loading: false,
          saving: false,
          error: null,
          gitCommitChanges: changes,
        },
      ];
    });
  }, []);

  const openTask = useCallback(
    (taskId: string, title: string) => {
      const relPath = taskTabKey(taskId);
      const name = title.length > 40 ? `${title.slice(0, 37)}…` : title;
      klog("workbench", `openTask id=${taskId}`);
      setActivePath(relPath);
      setTabs((current) => {
        const existing = current.find((tab) => tab.relPath === relPath);
        if (existing?.swiftTaskDetail && !existing.loading) {
          return current.map((tab) =>
            tab.relPath === relPath ? { ...tab, name, loading: false, error: null } : tab,
          );
        }
        if (existing) {
          return current.map((tab) =>
            tab.relPath === relPath
              ? {
                  ...tab,
                  name,
                  loading: true,
                  error: null,
                  swiftTaskDetail: null,
                }
              : tab,
          );
        }
        return [
          ...current,
          {
            relPath,
            name,
            content: "",
            savedContent: "",
            dirty: false,
            loading: true,
            saving: false,
            error: null,
            swiftTaskDetail: null,
          },
        ];
      });

      void swiftGetTask(taskId)
        .then((detail) => {
          setTabs((current) =>
            current.map((tab) =>
              tab.relPath === relPath
                ? {
                    ...tab,
                    name: detail.task.title.length > 40
                      ? `${detail.task.title.slice(0, 37)}…`
                      : detail.task.title,
                    loading: false,
                    error: null,
                    swiftTaskDetail: detail,
                  }
                : tab,
            ),
          );
        })
        .catch((error) => {
          const message = formatIpcError(error);
          setTabs((current) =>
            current.map((tab) =>
              tab.relPath === relPath ? { ...tab, loading: false, error: message } : tab,
            ),
          );
          toast.error(message);
        });
    },
    [toast],
  );

  const openDoc = useCallback(
    (projectId: string, entry: DocEntry) => {
      const relPath = docTabKey(projectId, entry.path);
      const name = entry.name;
      klog("workbench", `openDoc project=${projectId} path=${entry.path}`);
      setActivePath(relPath);
      setTabs((current) => {
        const existing = current.find((tab) => tab.relPath === relPath);
        if (existing?.docContent != null && !existing.loading) {
          return current.map((tab) => (tab.relPath === relPath ? { ...tab, name } : tab));
        }
        if (existing) {
          return current.map((tab) =>
            tab.relPath === relPath
              ? { ...tab, name, loading: true, error: null }
              : tab,
          );
        }
        return [
          ...current,
          {
            relPath,
            name,
            content: "",
            savedContent: "",
            dirty: false,
            loading: true,
            saving: false,
            error: null,
            docContent: null,
          },
        ];
      });

      void docsRead(projectId, entry.path)
        .then((markdown) => {
          setTabs((current) =>
            current.map((tab) =>
              tab.relPath === relPath
                ? { ...tab, loading: false, error: null, docContent: markdown }
                : tab,
            ),
          );
        })
        .catch((error) => {
          const message = formatIpcError(error);
          setTabs((current) =>
            current.map((tab) =>
              tab.relPath === relPath ? { ...tab, loading: false, error: message } : tab,
            ),
          );
          toast.error(message);
        });
    },
    [toast],
  );

  const openComponent = useCallback((def: ComponentDef) => {
    const relPath = componentTabKey(def.id);
    klog("workbench", `openComponent id=${def.id}`);
    setActivePath(relPath);
    setTabs((current) => {
      const existing = current.find((tab) => tab.relPath === relPath);
      if (existing) {
        return current;
      }
      return [
        ...current,
        {
          relPath,
          name: def.name,
          content: "",
          savedContent: "",
          dirty: false,
          loading: false,
          saving: false,
          error: null,
          componentId: def.id,
        },
      ];
    });
  }, []);

  const openTheme = useCallback(
    (theme: ThemeDefinition) => {
      const relPath = themeTabKey(theme.id);
      klog("workbench", `openTheme id=${theme.id}`);
      setActivePath(relPath);
      setTabs((current) => {
        const existing = current.find((tab) => tab.relPath === relPath);
        if (existing) {
          return current.map((tab) =>
            tab.relPath === relPath ? { ...tab, themeData: theme } : tab,
          );
        }
        return [
          ...current,
          {
            relPath,
            name: theme.id,
            content: "",
            savedContent: "",
            dirty: false,
            loading: false,
            saving: false,
            error: null,
            themeData: theme,
          },
        ];
      });

      void themeSetActive(theme.id)
        .then(() => fetchThemeCss())
        .then((css) => {
          applyThemeRootBlock(css.root_block);
          setActiveThemeId(css.id);
        })
        .catch((error) => {
          toast.error(formatIpcError(error));
        });
    },
    [toast],
  );

  const openSetting = useCallback((item: SettingsItem) => {
    const relPath = settingsTabKey(item.id);
    klog("workbench", `openSetting id=${item.id}`);
    setActivePath(relPath);
    setTabs((current) => {
      const existing = current.find((tab) => tab.relPath === relPath);
      if (existing) {
        return current;
      }
      return [
        ...current,
        {
          relPath,
          name: item.label,
          content: "",
          savedContent: "",
          dirty: false,
          loading: false,
          saving: false,
          error: null,
          settingsItemId: item.id,
        },
      ];
    });
  }, []);

  const closeTab = useCallback((relPath: string) => {
    setTabs((current) => {
      const next = current.filter((tab) => tab.relPath !== relPath);
      setActivePath((active) => {
        if (active !== relPath) {
          return active;
        }
        return next.length > 0 ? next[next.length - 1].relPath : null;
      });
      return next;
    });
  }, []);

  const openWorkspace = useCallback(
    async (root: string) => {
      try {
        const info = await openWorkspaceIpc(root);
        setWorkspace(info);
        setTabs([]);
        setActivePath(null);
        setRefreshToken((token) => token + 1);
        scheduleProblemsRun(800);
        toast.success(`Opened ${info.name}`);
      } catch (error) {
        toast.error(formatIpcError(error));
      }
    },
    [toast],
  );

  const refreshWorkspace = useCallback(() => {
    setRefreshToken((token) => token + 1);
  }, []);

  const value = useMemo<WorkbenchContextValue>(
    () => ({
      workspace,
      refreshToken,
      tabs,
      activePath,
      openFile,
      openIssue,
      openCommit,
      openTask,
      openDoc,
      openComponent,
      openTheme,
      openSetting,
      activeThemeId,
      focusTab,
      updateTabContent,
      saveTab,
      closeTab,
      openWorkspace,
      refreshWorkspace,
    }),
    [
      workspace,
      refreshToken,
      tabs,
      activePath,
      openFile,
      openIssue,
      openCommit,
      openTask,
      openDoc,
      openComponent,
      openTheme,
      openSetting,
      activeThemeId,
      focusTab,
      updateTabContent,
      saveTab,
      closeTab,
      openWorkspace,
      refreshWorkspace,
    ],
  );

  return <WorkbenchContext.Provider value={value}>{children}</WorkbenchContext.Provider>;
}

export function useWorkbench(): WorkbenchContextValue {
  const value = useContext(WorkbenchContext);
  if (!value) {
    throw new Error("useWorkbench must be used within a WorkbenchProvider");
  }
  return value;
}
