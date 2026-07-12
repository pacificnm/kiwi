import { useEffect, useState } from "react";
import { faXmark } from "../../lib/fontawesome";
import { Icon, useToast } from "../../shell";
import { formatIpcError } from "../../lib/agent";
import { gitBranchList, gitCreateBranch } from "../../lib/git";
import type { GitHubIssue } from "../../lib/github";
import { useWorkbench } from "../state";

/** GitHub-style default branch name: `<number>-<slugified-title>`. */
function defaultBranchName(issue: GitHubIssue): string {
  const slug = issue.title
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, "-")
    .replace(/^-+|-+$/g, "")
    .slice(0, 60)
    .replace(/-+$/g, "");
  return slug ? `${issue.number}-${slug}` : `issue-${issue.number}`;
}

type CreateBranchModalProps = {
  issue: GitHubIssue;
  onClose: () => void;
};

/** Creates and checks out a new branch off a base branch (default `main`). */
export function CreateBranchModal({ issue, onClose }: CreateBranchModalProps) {
  const toast = useToast();
  const { refreshWorkspace } = useWorkbench();
  const [branches, setBranches] = useState<string[]>([]);
  const [base, setBase] = useState("main");
  const [name, setName] = useState(() => defaultBranchName(issue));
  const [loading, setLoading] = useState(true);
  const [submitting, setSubmitting] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    let active = true;
    gitBranchList()
      .then((list) => {
        if (!active) {
          return;
        }
        setBranches(list);
        setBase((current) =>
          list.includes(current) ? current : list.includes("main") ? "main" : list[0] ?? current,
        );
      })
      .catch((err: unknown) => {
        if (active) {
          setError(formatIpcError(err));
        }
      })
      .finally(() => {
        if (active) {
          setLoading(false);
        }
      });
    return () => {
      active = false;
    };
  }, []);

  useEffect(() => {
    const onKeyDown = (event: KeyboardEvent) => {
      if (event.key === "Escape") {
        onClose();
      }
    };
    window.addEventListener("keydown", onKeyDown);
    return () => window.removeEventListener("keydown", onKeyDown);
  }, [onClose]);

  const submit = async () => {
    const trimmed = name.trim();
    if (!trimmed) {
      setError("Branch name is required");
      return;
    }
    setSubmitting(true);
    setError(null);
    try {
      await gitCreateBranch(trimmed, base);
      toast.success(`Created and checked out ${trimmed}`);
      refreshWorkspace();
      onClose();
    } catch (err: unknown) {
      setError(formatIpcError(err));
      setSubmitting(false);
    }
  };

  return (
    <div
      className="fixed inset-0 z-[70] flex items-center justify-center bg-black/40 p-4"
      onClick={onClose}
      role="presentation"
    >
      <div
        role="dialog"
        aria-label="Create branch"
        className="w-full max-w-lg rounded-nest-md border border-nest-border bg-nest-surface shadow-xl"
        onClick={(event) => event.stopPropagation()}
      >
        <header className="flex items-center justify-between border-b border-nest-border px-4 py-3">
          <h2 className="text-sm font-semibold text-nest-foreground">
            Create branch for #{issue.number}
          </h2>
          <button
            type="button"
            onClick={onClose}
            aria-label="Close"
            className="flex size-7 items-center justify-center rounded-nest-sm text-nest-muted hover:bg-nest-muted/10"
          >
            <Icon icon={faXmark} className="size-3.5" />
          </button>
        </header>
        <div className="space-y-3 p-4">
          <label className="block text-xs text-nest-muted">
            Branch name
            <input
              value={name}
              onChange={(event) => setName(event.target.value)}
              onKeyDown={(event) => {
                if (event.key === "Enter") {
                  event.preventDefault();
                  void submit();
                }
              }}
              autoFocus
              className="mt-1 h-8 w-full rounded-nest-sm border border-nest-border bg-nest-background px-2 font-mono text-sm text-nest-foreground focus:outline-none focus:ring-2 focus:ring-nest-accent/50"
            />
          </label>
          <label className="block text-xs text-nest-muted">
            Base branch
            <select
              value={base}
              onChange={(event) => setBase(event.target.value)}
              disabled={loading}
              className="mt-1 h-8 w-full rounded-nest-sm border border-nest-border bg-nest-background px-2 text-sm text-nest-foreground focus:outline-none focus:ring-2 focus:ring-nest-accent/50"
            >
              {branches.length === 0 ? <option value={base}>{base}</option> : null}
              {branches.map((branch) => (
                <option key={branch} value={branch}>
                  {branch}
                </option>
              ))}
            </select>
          </label>
          {error ? <p className="text-xs text-nest-error">{error}</p> : null}
        </div>
        <footer className="flex justify-end gap-2 border-t border-nest-border px-4 py-3">
          <button
            type="button"
            onClick={onClose}
            className="h-8 rounded-nest-sm px-3 text-xs text-nest-muted hover:bg-nest-muted/10"
          >
            Cancel
          </button>
          <button
            type="button"
            disabled={submitting || loading}
            onClick={() => void submit()}
            className="h-8 rounded-nest-sm bg-nest-accent px-3 text-xs font-semibold text-nest-background disabled:opacity-50"
          >
            {submitting ? "Creating…" : "Create & checkout"}
          </button>
        </footer>
      </div>
    </div>
  );
}
