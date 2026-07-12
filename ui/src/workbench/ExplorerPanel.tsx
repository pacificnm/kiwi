import {
  useCallback,
  useEffect,
  useMemo,
  useRef,
  useState,
  useTransition,
  type MouseEvent as ReactMouseEvent,
  type ReactNode,
} from "react";
import { Icon, isTauri, useToast } from "../shell";
import { formatIpcError } from "../lib/agent";
import { klog } from "../lib/log";
import {
  faChevronDown,
  faChevronLeft,
  faChevronRight,
  faFile,
  faFolder,
  faFolderOpen,
  faRotateRight,
} from "../lib/fontawesome";
import {
  baseName,
  copyPath,
  createDir,
  createFile,
  deletePath,
  joinRel,
  listDir,
  parentRel,
  renamePath,
  revealPath,
  type FsEntry,
} from "../lib/workspace";
import { ContextMenu, type ContextMenuItem } from "../components/ContextMenu";
import { ConfirmDialog } from "../components/ConfirmDialog";
import { PromptDialog } from "../components/PromptDialog";
import { useWorkbench } from "./state";

const ROOT = ".";

/** Right-click target: a tree row, or the panel background (project root). */
type MenuTarget = { relPath: string; isDir: boolean };

type MenuState = { x: number; y: number; target: MenuTarget };

/** Internal cut/copy clipboard (project-relative path + operation). */
type Clipboard = { op: "cut" | "copy"; relPath: string };

type PromptState =
  | { mode: "newFile" | "newFolder"; parentDir: string }
  | { mode: "rename"; target: MenuTarget };

type VisibleRow = {
  entry: FsEntry;
  depth: number;
};

/** Walks expanded directories into a flat row list (no recursive React components). */
function buildVisibleRows(
  entriesByDir: Record<string, FsEntry[]>,
  expanded: ReadonlySet<string>,
  rel: string,
  depth: number,
  rows: VisibleRow[],
): void {
  const entries = entriesByDir[rel];
  if (!entries) {
    return;
  }
  for (const entry of entries) {
    rows.push({ entry, depth });
    if (entry.isDir && expanded.has(entry.relPath)) {
      buildVisibleRows(entriesByDir, expanded, entry.relPath, depth + 1, rows);
    }
  }
}

/** Removes cached listings / expansion for `rel` and everything beneath it. */
function pruneSubtree<T>(map: Record<string, T>, rel: string): Record<string, T> {
  const prefix = `${rel}/`;
  const next: Record<string, T> = {};
  for (const [key, value] of Object.entries(map)) {
    if (key === rel || key.startsWith(prefix)) {
      continue;
    }
    next[key] = value;
  }
  return next;
}

export function ExplorerPanel({ onToggleCollapse }: { onToggleCollapse?: () => void }) {
  const { workspace, activePath, refreshToken, openFile, closeTab, refreshWorkspace } =
    useWorkbench();
  const toast = useToast();
  const [entriesByDir, setEntriesByDir] = useState<Record<string, FsEntry[]>>({});
  const [expanded, setExpanded] = useState<Set<string>>(new Set());
  const [loading, setLoading] = useState<Set<string>>(new Set());
  const [errors, setErrors] = useState<Record<string, string>>({});
  const [menu, setMenu] = useState<MenuState | null>(null);
  const [clipboard, setClipboard] = useState<Clipboard | null>(null);
  const [prompt, setPrompt] = useState<PromptState | null>(null);
  const [confirmDelete, setConfirmDelete] = useState<MenuTarget | null>(null);
  const [, startTransition] = useTransition();

  // Track the current expansion and the workspace we last built the tree for,
  // so a refresh can reload in place without collapsing the tree.
  const expandedRef = useRef(expanded);
  expandedRef.current = expanded;
  const builtRootRef = useRef<string | null>(null);
  // WebKitGTK (Linux) fires a synthetic `click` right after `contextmenu`.
  // Ignore row clicks until this timestamp so right-clicking a folder doesn't
  // also toggle (collapse) it.
  const suppressClickUntilRef = useRef(0);

  const loadDir = useCallback((rel: string) => {
    klog("explorer", `loadDir start rel=${rel}`);
    setLoading((current) => new Set(current).add(rel));
    void listDir(rel)
      .then((entries) => {
        klog("explorer", `loadDir done rel=${rel} count=${entries.length}`);
        setEntriesByDir((current) => ({ ...current, [rel]: entries }));
        setErrors((current) => {
          const next = { ...current };
          delete next[rel];
          return next;
        });
      })
      .catch((error) => {
        setErrors((current) => ({ ...current, [rel]: formatIpcError(error) }));
      })
      .finally(() => {
        setLoading((current) => {
          const next = new Set(current);
          next.delete(rel);
          return next;
        });
      });
  }, []);

  useEffect(() => {
    klog("explorer", `root effect (workspace=${workspace?.name ?? "null"} token=${refreshToken})`);
    if (!isTauri() || !workspace) {
      return;
    }
    if (builtRootRef.current !== workspace.root) {
      // Switched projects (or first load): rebuild the tree from scratch.
      builtRootRef.current = workspace.root;
      setEntriesByDir({});
      setExpanded(new Set());
      setErrors({});
      loadDir(ROOT);
      return;
    }
    // Same project, explicit refresh: reload the root and every expanded folder
    // in place, keeping the tree's expansion state intact.
    loadDir(ROOT);
    for (const rel of expandedRef.current) {
      loadDir(rel);
    }
  }, [workspace, refreshToken, loadDir]);

  const toggleDir = useCallback(
    (rel: string) => {
      const willExpand = !expanded.has(rel);
      klog("explorer", `toggleDir rel=${rel} willExpand=${willExpand} cached=${!!entriesByDir[rel]}`);
      startTransition(() => {
        setExpanded((current) => {
          const next = new Set(current);
          if (next.has(rel)) {
            next.delete(rel);
          } else {
            next.add(rel);
          }
          return next;
        });
      });
      if (willExpand && !entriesByDir[rel]) {
        loadDir(rel);
      }
    },
    [expanded, entriesByDir, loadDir, startTransition],
  );

  const expandDir = useCallback((rel: string) => {
    if (rel === ROOT) {
      return;
    }
    setExpanded((current) => {
      if (current.has(rel)) {
        return current;
      }
      const next = new Set(current);
      next.add(rel);
      return next;
    });
  }, []);

  const collapseAll = useCallback(() => setExpanded(new Set()), []);

  const visibleRows = useMemo(() => {
    const rows: VisibleRow[] = [];
    buildVisibleRows(entriesByDir, expanded, ROOT, 0, rows);
    klog("explorer", `visibleRows count=${rows.length} expanded=[${[...expanded].join(",")}]`);
    return rows;
  }, [entriesByDir, expanded]);

  // --- Context-menu command handlers ------------------------------------

  const forgetSubtree = useCallback((rel: string) => {
    setEntriesByDir((current) => pruneSubtree(current, rel));
    setExpanded((current) => {
      const pruned = pruneSubtree(
        Object.fromEntries([...current].map((key) => [key, true])),
        rel,
      );
      return new Set(Object.keys(pruned));
    });
  }, []);

  const submitPrompt = useCallback(
    async (value: string) => {
      if (!prompt) {
        return;
      }
      if (prompt.mode === "rename") {
        const { target } = prompt;
        const dest = joinRel(parentRel(target.relPath), value);
        if (dest === target.relPath) {
          setPrompt(null);
          return;
        }
        await renamePath(target.relPath, dest);
        forgetSubtree(target.relPath);
        if (!target.isDir) {
          closeTab(target.relPath);
        }
        loadDir(parentRel(target.relPath));
        toast.success(`Renamed to ${baseName(dest)}`);
        setPrompt(null);
        return;
      }

      const { parentDir, mode } = prompt;
      const rel = joinRel(parentDir, value);
      if (mode === "newFolder") {
        await createDir(rel);
        expandDir(parentDir);
        loadDir(parentDir);
        toast.success(`Created folder ${value}`);
      } else {
        await createFile(rel);
        expandDir(parentDir);
        loadDir(parentDir);
        openFile(rel, baseName(rel));
        toast.success(`Created file ${value}`);
      }
      setPrompt(null);
    },
    [prompt, forgetSubtree, closeTab, loadDir, expandDir, openFile, toast],
  );

  const runDelete = useCallback(
    (target: MenuTarget) => {
      void deletePath(target.relPath)
        .then(() => {
          forgetSubtree(target.relPath);
          if (!target.isDir) {
            closeTab(target.relPath);
          }
          loadDir(parentRel(target.relPath));
          toast.success(`Deleted ${baseName(target.relPath)}`);
        })
        .catch((error) => toast.error(formatIpcError(error)))
        .finally(() => setConfirmDelete(null));
    },
    [forgetSubtree, closeTab, loadDir, toast],
  );

  const runPaste = useCallback(
    (target: MenuTarget) => {
      if (!clipboard) {
        return;
      }
      const destDir = target.isDir ? target.relPath : parentRel(target.relPath);
      const dest = joinRel(destDir, baseName(clipboard.relPath));
      const op = clipboard.op;
      const source = clipboard.relPath;
      const action = op === "cut" ? renamePath(source, dest) : copyPath(source, dest);
      void action
        .then(() => {
          if (op === "cut") {
            forgetSubtree(source);
            loadDir(parentRel(source));
            setClipboard(null);
          }
          expandDir(destDir);
          loadDir(destDir);
          toast.success(`Pasted ${baseName(source)} into ${destDir === ROOT ? "root" : destDir}`);
        })
        .catch((error) => toast.error(formatIpcError(error)));
    },
    [clipboard, forgetSubtree, loadDir, expandDir, toast],
  );

  const copyToSystemClipboard = useCallback(
    (text: string, what: string) => {
      void navigator.clipboard
        .writeText(text)
        .then(() => toast.success(`Copied ${what}`))
        .catch((error) => toast.error(formatIpcError(error)));
    },
    [toast],
  );

  const runReveal = useCallback(
    (target: MenuTarget) => {
      void revealPath(target.relPath).catch((error) => toast.error(formatIpcError(error)));
    },
    [toast],
  );

  const buildMenuItems = useCallback(
    (target: MenuTarget): ContextMenuItem[] => {
      const isRoot = target.relPath === ROOT;
      const canPaste = clipboard !== null;
      const dirForCreate = target.isDir ? target.relPath : parentRel(target.relPath);
      const absPath =
        isRoot || !workspace
          ? (workspace?.root ?? target.relPath)
          : `${workspace.root}/${target.relPath}`;

      return [
        {
          id: "new-file",
          label: "New File",
          onSelect: () => setPrompt({ mode: "newFile", parentDir: dirForCreate }),
        },
        {
          id: "new-folder",
          label: "New Folder",
          onSelect: () => setPrompt({ mode: "newFolder", parentDir: dirForCreate }),
        },
        {
          id: "reveal",
          label: "Open Containing Folder",
          onSelect: () => runReveal(target),
        },
        { kind: "separator", id: "sep-1" },
        {
          id: "cut",
          label: "Cut",
          disabled: isRoot,
          onSelect: () => setClipboard({ op: "cut", relPath: target.relPath }),
        },
        {
          id: "copy",
          label: "Copy",
          disabled: isRoot,
          onSelect: () => setClipboard({ op: "copy", relPath: target.relPath }),
        },
        {
          id: "paste",
          label: "Paste",
          disabled: !canPaste,
          onSelect: () => runPaste(target),
        },
        { kind: "separator", id: "sep-2" },
        {
          id: "copy-path",
          label: "Copy Path",
          onSelect: () => copyToSystemClipboard(absPath, "path"),
        },
        {
          id: "copy-rel-path",
          label: "Copy Relative Path",
          disabled: isRoot,
          onSelect: () => copyToSystemClipboard(target.relPath, "relative path"),
        },
        { kind: "separator", id: "sep-3" },
        {
          id: "rename",
          label: "Rename",
          disabled: isRoot,
          onSelect: () => setPrompt({ mode: "rename", target }),
        },
        {
          id: "delete",
          label: "Delete",
          danger: true,
          disabled: isRoot,
          onSelect: () => setConfirmDelete(target),
        },
      ];
    },
    [clipboard, workspace, runReveal, runPaste, copyToSystemClipboard],
  );

  const openMenu = useCallback((event: ReactMouseEvent, target: MenuTarget) => {
    event.preventDefault();
    event.stopPropagation();
    suppressClickUntilRef.current = Date.now() + 400;
    setMenu({ x: event.clientX, y: event.clientY, target });
  }, []);

  if (!isTauri()) {
    return (
      <PanelFrame onRefresh={refreshWorkspace} onCollapse={collapseAll} onToggleCollapse={onToggleCollapse} label="Explorer"
        content={
        <p className="px-3 py-2 text-xs text-nest-muted">
          The file tree is available in the desktop app.
        </p>
        }
      />
    );
  }

  const rootError = errors[ROOT];
  const rootLoading = loading.has(ROOT) && !entriesByDir[ROOT];

  const promptProps = (() => {
    if (!prompt) {
      return null;
    }
    if (prompt.mode === "rename") {
      const name = baseName(prompt.target.relPath);
      return {
        title: "Rename",
        label: "New name",
        description: `Rename ${name}.`,
        initialValue: name,
        confirmLabel: "Rename",
      };
    }
    const where = prompt.parentDir === ROOT ? "project root" : prompt.parentDir;
    if (prompt.mode === "newFolder") {
      return {
        title: "New Folder",
        label: "Folder name",
        description: `Create a new folder in ${where}.`,
        placeholder: "components",
        confirmLabel: "Create",
      };
    }
    return {
      title: "New File",
      label: "File name",
      description: `Create a new file in ${where}.`,
      placeholder: "example.ts",
      confirmLabel: "Create",
    };
  })();

  return (
    <>
      <PanelFrame
        onRefresh={refreshWorkspace}
        onCollapse={collapseAll}
        onToggleCollapse={onToggleCollapse}
        label={workspace?.name ?? "Explorer"}
        content={
          rootError ? (
            <p className="px-3 py-2 text-xs text-nest-error">{rootError}</p>
          ) : rootLoading ? (
            <p className="px-3 py-2 text-xs text-nest-muted">Loading…</p>
          ) : (
            <div
              className="min-h-full"
              onContextMenu={(event) => openMenu(event, { relPath: ROOT, isDir: true })}
            >
              <ul className="py-1 text-[13px]" role="tree">
                {visibleRows.map(({ entry, depth }) => {
                  const isExpanded = entry.isDir && expanded.has(entry.relPath);
                  const isSelected = !entry.isDir && activePath === entry.relPath;
                  const isCut = clipboard?.op === "cut" && clipboard.relPath === entry.relPath;
                  const indent = 8 + depth * 12;
                  const dirLoading = entry.isDir && isExpanded && loading.has(entry.relPath);

                  return (
                    <li
                      key={entry.relPath}
                      role="treeitem"
                      aria-expanded={entry.isDir ? isExpanded : undefined}
                    >
                      <button
                        type="button"
                        onClick={() => {
                          if (Date.now() < suppressClickUntilRef.current) {
                            suppressClickUntilRef.current = 0;
                            return;
                          }
                          if (entry.isDir) {
                            toggleDir(entry.relPath);
                          } else {
                            openFile(entry.relPath, entry.name);
                          }
                        }}
                        onContextMenu={(event) =>
                          openMenu(event, { relPath: entry.relPath, isDir: entry.isDir })
                        }
                        title={entry.relPath}
                        className={[
                          "flex w-full items-center gap-1.5 py-0.5 pr-2 text-left transition-colors",
                          isCut ? "opacity-50" : "",
                          isSelected
                            ? "bg-nest-accent/15 text-nest-foreground"
                            : "text-nest-foreground/90 hover:bg-nest-muted/10",
                        ].join(" ")}
                        style={{ paddingLeft: `${indent}px` }}
                      >
                        {entry.isDir ? (
                          <Icon
                            icon={isExpanded ? faChevronDown : faChevronRight}
                            className="size-2.5 shrink-0 text-nest-muted"
                          />
                        ) : (
                          <span className="w-2.5 shrink-0" />
                        )}
                        <Icon
                          icon={entry.isDir ? (isExpanded ? faFolderOpen : faFolder) : faFile}
                          className={[
                            "size-3.5 shrink-0",
                            entry.isDir ? "text-nest-accent" : "text-nest-muted",
                          ].join(" ")}
                        />
                        <span className="truncate">{entry.name}</span>
                      </button>
                      {dirLoading ? (
                        <p
                          className="py-0.5 pr-2 text-xs text-nest-muted"
                          style={{ paddingLeft: `${indent + 20}px` }}
                        >
                          Loading…
                        </p>
                      ) : null}
                      {errors[entry.relPath] ? (
                        <p
                          className="py-0.5 pr-2 text-xs text-nest-error"
                          style={{ paddingLeft: `${indent + 20}px` }}
                        >
                          {errors[entry.relPath]}
                        </p>
                      ) : null}
                    </li>
                  );
                })}
              </ul>
            </div>
          )
        }
      />

      {menu ? (
        <ContextMenu
          x={menu.x}
          y={menu.y}
          items={buildMenuItems(menu.target)}
          onClose={() => setMenu(null)}
        />
      ) : null}

      {promptProps ? (
        <PromptDialog
          open
          title={promptProps.title}
          label={promptProps.label}
          description={promptProps.description}
          initialValue={promptProps.initialValue}
          placeholder={promptProps.placeholder}
          confirmLabel={promptProps.confirmLabel}
          onSubmit={submitPrompt}
          onCancel={() => setPrompt(null)}
        />
      ) : null}

      <ConfirmDialog
        open={confirmDelete !== null}
        title="Delete"
        danger
        confirmLabel="Delete"
        message={
          confirmDelete
            ? `Delete ${confirmDelete.isDir ? "folder" : "file"} "${baseName(
                confirmDelete.relPath,
              )}"?${confirmDelete.isDir ? "\nThis removes all of its contents." : ""}`
            : ""
        }
        onConfirm={() => confirmDelete && runDelete(confirmDelete)}
        onCancel={() => setConfirmDelete(null)}
      />
    </>
  );
}

function PanelFrame({
  label,
  onRefresh,
  onCollapse,
  onToggleCollapse,
  content,
}: {
  label: string;
  onRefresh: () => void;
  onCollapse: () => void;
  onToggleCollapse?: () => void;
  content: ReactNode;
}) {
  return (
    <div className="flex h-full min-h-0 flex-col bg-nest-background">
      <header className="flex h-9 shrink-0 items-center gap-1 border-b border-nest-border px-3">
        <span className="truncate text-xs font-semibold uppercase tracking-wide text-nest-muted">
          {label}
        </span>
        <div className="ml-auto flex items-center gap-0.5">
          <button
            type="button"
            onClick={onRefresh}
            title="Refresh"
            aria-label="Refresh"
            className="flex size-6 items-center justify-center rounded-nest-sm text-nest-muted hover:bg-nest-muted/10 hover:text-nest-foreground"
          >
            <Icon icon={faRotateRight} className="size-3" />
          </button>
          <button
            type="button"
            onClick={onCollapse}
            title="Collapse all"
            aria-label="Collapse all"
            className="flex size-6 items-center justify-center rounded-nest-sm text-nest-muted hover:bg-nest-muted/10 hover:text-nest-foreground"
          >
            <Icon icon={faChevronDown} className="size-3" />
          </button>
          {onToggleCollapse ? (
            <button
              type="button"
              onClick={onToggleCollapse}
              title="Hide sidebar"
              aria-label="Hide sidebar"
              className="flex size-6 items-center justify-center rounded-nest-sm text-nest-muted hover:bg-nest-muted/10 hover:text-nest-foreground"
            >
              <Icon icon={faChevronLeft} className="size-3" />
            </button>
          ) : null}
        </div>
      </header>
      <div className="min-h-0 flex-1 overflow-auto">{content}</div>
    </div>
  );
}
