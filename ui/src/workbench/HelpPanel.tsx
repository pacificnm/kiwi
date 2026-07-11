import { useCallback, useEffect, useState } from "react";
import { formatIpcError } from "../lib/agent";
import { docsList, docTabKey, type DocEntry } from "../lib/docs";
import { docSourcesList, docSourceSync, type DocProject } from "../lib/docSources";
import { faChevronDown, faChevronLeft, faChevronRight, faRotateRight } from "../lib/fontawesome";
import { Icon, isTauri } from "../shell";
import { useWorkbench } from "./state";

type HelpPanelProps = {
  onToggleCollapse?: () => void;
};

/** Help Activity sidebar — lists docs grouped by registered project; opens each as an editor tab. */
export function HelpPanel({ onToggleCollapse }: HelpPanelProps) {
  const [projects, setProjects] = useState<DocProject[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const loadProjects = useCallback(async () => {
    if (!isTauri()) {
      setLoading(false);
      return;
    }
    try {
      const list = await docSourcesList();
      setProjects(list);
      setError(null);
    } catch (loadError) {
      setError(formatIpcError(loadError));
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    void loadProjects();
  }, [loadProjects]);

  if (!isTauri()) {
    return (
      <PanelFrame onToggleCollapse={onToggleCollapse}>
        <p className="px-3 py-2 text-xs text-nest-muted">Help is available in the desktop app.</p>
      </PanelFrame>
    );
  }

  return (
    <PanelFrame onToggleCollapse={onToggleCollapse}>
      {error ? <p className="px-3 py-2 text-xs text-nest-error">{error}</p> : null}
      {loading ? (
        <p className="px-3 py-4 text-xs text-nest-muted">Loading…</p>
      ) : projects.length === 0 ? (
        <p className="px-3 py-4 text-xs text-nest-muted">
          No doc sources yet. Add one in Settings &rarr; Help &rarr; Doc Sources.
        </p>
      ) : (
        projects.map((project) => (
          <ProjectSection key={project.id} project={project} onSynced={loadProjects} />
        ))
      )}
    </PanelFrame>
  );
}

function ProjectSection({
  project,
  onSynced,
}: {
  project: DocProject;
  onSynced: () => void;
}) {
  const { activePath, openDoc } = useWorkbench();
  const [entries, setEntries] = useState<DocEntry[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [syncing, setSyncing] = useState(false);
  const [open, setOpen] = useState(true);

  useEffect(() => {
    if (!project.synced) {
      return;
    }
    let cancelled = false;
    setLoading(true);
    void docsList(project.id)
      .then((list) => {
        if (!cancelled) {
          setEntries(list);
          setError(null);
        }
      })
      .catch((loadError) => {
        if (!cancelled) {
          setError(formatIpcError(loadError));
        }
      })
      .finally(() => {
        if (!cancelled) {
          setLoading(false);
        }
      });
    return () => {
      cancelled = true;
    };
  }, [project.id, project.synced]);

  const handleSync = useCallback(async () => {
    setSyncing(true);
    setError(null);
    try {
      await docSourceSync(project.id);
      onSynced();
    } catch (syncError) {
      setError(formatIpcError(syncError));
    } finally {
      setSyncing(false);
    }
  }, [project.id, onSynced]);

  return (
    <div className="border-b border-nest-border">
      <div className="flex items-center gap-1.5 px-3 py-1.5">
        <button
          type="button"
          onClick={() => setOpen((value) => !value)}
          className="flex min-w-0 flex-1 items-center gap-1.5 text-left text-[11px] font-semibold uppercase tracking-wide text-nest-muted hover:text-nest-foreground"
        >
          <Icon icon={open ? faChevronDown : faChevronRight} className="size-2.5 shrink-0" />
          <span className="min-w-0 flex-1 truncate">{project.name}</span>
        </button>
        <button
          type="button"
          onClick={() => void handleSync()}
          disabled={syncing}
          title={project.synced ? "Pull latest docs" : "Sync docs"}
          className="flex size-5 shrink-0 items-center justify-center rounded-nest-sm text-nest-muted hover:bg-nest-muted/10 hover:text-nest-foreground disabled:opacity-50"
        >
          <Icon icon={faRotateRight} className={["size-3", syncing ? "animate-spin" : ""].join(" ")} />
        </button>
      </div>
      {open ? (
        <>
          {error ? <p className="px-3 pb-2 text-[11px] text-nest-error">{error}</p> : null}
          {!project.synced ? (
            <p className="px-3 pb-2 text-[11px] italic text-nest-muted/70">Not synced yet</p>
          ) : loading ? (
            <p className="px-3 pb-2 text-[11px] text-nest-muted">Loading…</p>
          ) : entries.length === 0 ? (
            <p className="px-3 pb-2 text-[11px] italic text-nest-muted/70">No documents found</p>
          ) : (
            <ul className="pb-1">
              {entries.map((entry) => {
                const selected = activePath === docTabKey(project.id, entry.path);
                return (
                  <li key={entry.path}>
                    <button
                      type="button"
                      onClick={() => openDoc(project.id, entry)}
                      title={entry.path}
                      style={{ paddingLeft: `${12 + entry.depth * 14}px` }}
                      className={[
                        "flex w-full items-center gap-1.5 py-1.5 pr-3 text-left text-xs transition-colors",
                        selected
                          ? "bg-nest-accent/15 text-nest-foreground"
                          : "text-nest-muted hover:bg-nest-muted/10 hover:text-nest-foreground",
                      ].join(" ")}
                    >
                      <span className="min-w-0 flex-1 truncate">{entry.name}</span>
                    </button>
                  </li>
                );
              })}
            </ul>
          )}
        </>
      ) : null}
    </div>
  );
}

function PanelFrame({
  children,
  onToggleCollapse,
}: {
  children: React.ReactNode;
  onToggleCollapse?: () => void;
}) {
  return (
    <div className="flex h-full min-h-0 flex-col bg-nest-background">
      <header className="flex h-9 shrink-0 items-center border-b border-nest-border px-3">
        <span className="text-xs font-semibold uppercase tracking-wide text-nest-muted">Help</span>
        {onToggleCollapse ? (
          <button
            type="button"
            onClick={onToggleCollapse}
            title="Hide sidebar"
            aria-label="Hide sidebar"
            className="ml-auto flex size-6 items-center justify-center rounded-nest-sm text-nest-muted hover:bg-nest-muted/10 hover:text-nest-foreground"
          >
            <Icon icon={faChevronLeft} className="size-3" />
          </button>
        ) : null}
      </header>
      <div className="min-h-0 flex-1 overflow-auto">{children}</div>
    </div>
  );
}
