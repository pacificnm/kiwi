import { useCallback, useState } from "react";
import { formatIpcError } from "../lib/agent";
import { gitFetchSource, gitPull } from "../lib/git";
import { isTauri } from "../shell";
import { useWorkbench } from "./state";

const DEFAULT_URL = "https://github.com/pacificnm/nest.git";

const inputClass =
  "h-7 w-full rounded-nest-sm border border-nest-border bg-nest-surface px-2 text-xs";

/**
 * File → Fetch Nest Source: clones a repo into the (empty) workspace root
 * so a fresh folder can be turned into a working Nest checkout.
 */
export function FetchSourceView() {
  const { workspace, refreshWorkspace } = useWorkbench();
  const [url, setUrl] = useState(DEFAULT_URL);
  const [branch, setBranch] = useState("main");
  const [fetching, setFetching] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [done, setDone] = useState(false);
  const [pulling, setPulling] = useState(false);
  const [pullError, setPullError] = useState<string | null>(null);
  const [pullDone, setPullDone] = useState(false);

  const handleFetch = useCallback(async () => {
    if (!url.trim()) {
      setError("Git URL is required.");
      return;
    }
    setFetching(true);
    setError(null);
    setDone(false);
    try {
      await gitFetchSource(url.trim(), branch.trim() || "main");
      setDone(true);
      refreshWorkspace();
    } catch (fetchError) {
      setError(formatIpcError(fetchError));
    } finally {
      setFetching(false);
    }
  }, [url, branch, refreshWorkspace]);

  const handlePull = useCallback(async () => {
    setPulling(true);
    setPullError(null);
    setPullDone(false);
    try {
      await gitPull();
      setPullDone(true);
      refreshWorkspace();
    } catch (pullFailure) {
      setPullError(formatIpcError(pullFailure));
    } finally {
      setPulling(false);
    }
  }, [refreshWorkspace]);

  if (!isTauri()) {
    return (
      <div className="flex h-full items-center justify-center text-sm text-nest-muted">
        Fetch Nest Source is available in the desktop app.
      </div>
    );
  }

  return (
    <div className="flex h-full min-h-0 flex-col bg-nest-background">
      <header className="border-b border-nest-border bg-nest-surface/60 px-6 py-5">
        <h1 className="text-xl font-semibold text-nest-foreground">Fetch Nest Source</h1>
        <p className="mt-0.5 text-sm text-nest-muted">
          Fetches Nest&rsquo;s repository into the current workspace
          {workspace ? (
            <>
              {" "}
              (<code className="font-mono text-[12px]">{workspace.root}</code>)
            </>
          ) : null}
          {" "}
          — the folder must be empty. Fetching also installs the shared UI crate dependencies,
          so it may take a minute. Once checked out, use Pull to bring in the latest changes
          for the current branch.
        </p>
      </header>
      <div className="min-h-0 flex-1 overflow-y-auto px-6 py-6">
        <section className="max-w-[520px] rounded-nest-lg border border-nest-border bg-nest-surface p-4">
          <div className="space-y-2">
            <label className="block">
              <span className="mb-0.5 block text-[11px] text-nest-muted">Git URL</span>
              <input
                className={inputClass}
                value={url}
                onChange={(event) => setUrl(event.target.value)}
                placeholder={DEFAULT_URL}
              />
            </label>
            <label className="block">
              <span className="mb-0.5 block text-[11px] text-nest-muted">Branch</span>
              <input
                className={inputClass}
                value={branch}
                onChange={(event) => setBranch(event.target.value)}
                placeholder="main"
              />
            </label>
            <div className="flex gap-2">
              <button
                type="button"
                onClick={() => void handleFetch()}
                disabled={fetching}
                className="h-7 rounded-nest-sm border border-nest-border px-3 text-xs hover:bg-nest-muted/10 disabled:opacity-50"
              >
                {fetching ? "Fetching…" : "Fetch"}
              </button>
              <button
                type="button"
                onClick={() => void handlePull()}
                disabled={pulling}
                title="Pull the latest changes for the current branch"
                className="h-7 rounded-nest-sm border border-nest-border px-3 text-xs hover:bg-nest-muted/10 disabled:opacity-50"
              >
                {pulling ? "Pulling…" : "Pull"}
              </button>
            </div>
            {error ? <p className="text-xs text-nest-error">{error}</p> : null}
            {done && !error ? (
              <p className="text-xs text-nest-success">
                Cloned successfully. The Explorer has been refreshed.
              </p>
            ) : null}
            {pullError ? <p className="text-xs text-nest-error">{pullError}</p> : null}
            {pullDone && !pullError ? (
              <p className="text-xs text-nest-success">
                Pulled the latest changes. The Explorer has been refreshed.
              </p>
            ) : null}
          </div>
        </section>
      </div>
    </div>
  );
}
