import { useCallback, useEffect, useState } from "react";
import { formatIpcError } from "../lib/agent";
import {
  docSourceAdd,
  docSourceRemove,
  docSourceSync,
  docSourcesList,
  type DocProject,
} from "../lib/docSources";
import { isTauri } from "../shell";

const inputClass =
  "h-7 w-full rounded-nest-sm border border-nest-border bg-nest-surface px-2 text-xs";

/** Settings detail view for the "Doc Sources" item — add/sync/remove Help doc projects. */
export function DocSourcesSettingsView() {
  const [projects, setProjects] = useState<DocProject[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [busyId, setBusyId] = useState<string | null>(null);

  const [name, setName] = useState("");
  const [repoUrl, setRepoUrl] = useState("");
  const [docsPath, setDocsPath] = useState("docs");
  const [branch, setBranch] = useState("");
  const [adding, setAdding] = useState(false);

  const refresh = useCallback(async () => {
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
    if (!isTauri()) {
      setLoading(false);
      return;
    }
    void refresh();
  }, [refresh]);

  const handleAdd = useCallback(async () => {
    if (!name.trim() || !repoUrl.trim()) {
      setError("Name and repo URL are required.");
      return;
    }
    setAdding(true);
    setError(null);
    try {
      await docSourceAdd({
        name: name.trim(),
        repoUrl: repoUrl.trim(),
        docsPath: docsPath.trim() || "docs",
        branch: branch.trim() || null,
      });
      setName("");
      setRepoUrl("");
      setDocsPath("docs");
      setBranch("");
      await refresh();
    } catch (addError) {
      setError(formatIpcError(addError));
    } finally {
      setAdding(false);
    }
  }, [name, repoUrl, docsPath, branch, refresh]);

  const handleSync = useCallback(
    async (id: string) => {
      setBusyId(id);
      setError(null);
      try {
        await docSourceSync(id);
        await refresh();
      } catch (syncError) {
        setError(formatIpcError(syncError));
      } finally {
        setBusyId(null);
      }
    },
    [refresh],
  );

  const handleRemove = useCallback(
    async (id: string) => {
      setBusyId(id);
      setError(null);
      try {
        await docSourceRemove(id);
        await refresh();
      } catch (removeError) {
        setError(formatIpcError(removeError));
      } finally {
        setBusyId(null);
      }
    },
    [refresh],
  );

  if (!isTauri()) {
    return (
      <div className="flex h-full items-center justify-center text-sm text-nest-muted">
        Doc Sources are managed in the desktop app.
      </div>
    );
  }

  return (
    <div className="flex h-full min-h-0 flex-col bg-nest-background">
      <header className="border-b border-nest-border bg-nest-surface/60 px-6 py-5">
        <h1 className="text-xl font-semibold text-nest-foreground">Doc Sources</h1>
        <p className="mt-0.5 text-sm text-nest-muted">
          Register a project&rsquo;s docs by git repo + subpath. Each source is sparse-checked-out
          into <code className="font-mono text-[12px]">~/.config/kiwi/docs/&lt;id&gt;</code> and
          shown as a group in Help.
        </p>
      </header>
      <div className="min-h-0 flex-1 overflow-y-auto px-6 py-6">
        {error ? <p className="mb-4 text-sm text-nest-error">{error}</p> : null}

        <section className="mb-8 max-w-[520px] rounded-nest-lg border border-nest-border bg-nest-surface p-4">
          <h2 className="mb-3 text-sm font-semibold text-nest-foreground">Add a doc source</h2>
          <div className="space-y-2">
            <label className="block">
              <span className="mb-0.5 block text-[11px] text-nest-muted">Name</span>
              <input
                className={inputClass}
                value={name}
                onChange={(event) => setName(event.target.value)}
                placeholder="Nest"
              />
            </label>
            <label className="block">
              <span className="mb-0.5 block text-[11px] text-nest-muted">Repo URL</span>
              <input
                className={inputClass}
                value={repoUrl}
                onChange={(event) => setRepoUrl(event.target.value)}
                placeholder="https://github.com/org/nest.git"
              />
            </label>
            <div className="flex gap-2">
              <label className="min-w-0 flex-1">
                <span className="mb-0.5 block text-[11px] text-nest-muted">Docs path</span>
                <input
                  className={inputClass}
                  value={docsPath}
                  onChange={(event) => setDocsPath(event.target.value)}
                  placeholder="docs"
                />
              </label>
              <label className="min-w-0 flex-1">
                <span className="mb-0.5 block text-[11px] text-nest-muted">Branch (optional)</span>
                <input
                  className={inputClass}
                  value={branch}
                  onChange={(event) => setBranch(event.target.value)}
                  placeholder="main"
                />
              </label>
            </div>
            <button
              type="button"
              onClick={() => void handleAdd()}
              disabled={adding}
              className="h-7 rounded-nest-sm border border-nest-border px-3 text-xs hover:bg-nest-muted/10 disabled:opacity-50"
            >
              {adding ? "Adding…" : "Add"}
            </button>
          </div>
        </section>

        <section>
          <h2 className="mb-3 text-sm font-semibold text-nest-foreground">Registered sources</h2>
          {loading ? (
            <p className="text-xs text-nest-muted">Loading…</p>
          ) : projects.length === 0 ? (
            <p className="text-xs text-nest-muted">No doc sources registered yet.</p>
          ) : (
            <ul className="space-y-2">
              {projects.map((project) => (
                <li
                  key={project.id}
                  className="flex items-center gap-3 rounded-nest-lg border border-nest-border bg-nest-surface p-3"
                >
                  <div className="min-w-0 flex-1">
                    <p className="truncate text-sm font-medium text-nest-foreground">
                      {project.name}
                    </p>
                    <p className="truncate text-[11px] text-nest-muted" title={project.repoUrl}>
                      {project.repoUrl} &middot; {project.docsPath}
                      {project.branch ? ` @ ${project.branch}` : ""}
                    </p>
                    <p className="text-[11px] text-nest-muted">
                      {project.synced ? (
                        <span className="text-nest-success">
                          Synced{project.lastSyncedAt ? ` ${formatSyncedAt(project.lastSyncedAt)}` : ""}
                        </span>
                      ) : (
                        <span>Not synced yet</span>
                      )}
                    </p>
                  </div>
                  <button
                    type="button"
                    onClick={() => void handleSync(project.id)}
                    disabled={busyId === project.id}
                    className="h-7 shrink-0 rounded-nest-sm border border-nest-border px-3 text-xs hover:bg-nest-muted/10 disabled:opacity-50"
                  >
                    {busyId === project.id ? "Working…" : project.synced ? "Pull" : "Sync"}
                  </button>
                  <button
                    type="button"
                    onClick={() => void handleRemove(project.id)}
                    disabled={busyId === project.id}
                    className="h-7 shrink-0 rounded-nest-sm border border-nest-border px-3 text-xs hover:bg-nest-error/10 hover:text-nest-error disabled:opacity-50"
                  >
                    Remove
                  </button>
                </li>
              ))}
            </ul>
          )}
        </section>
      </div>
    </div>
  );
}

function formatSyncedAt(unixSeconds: number): string {
  return new Date(unixSeconds * 1000).toLocaleString();
}
