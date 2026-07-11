import { useCallback, useEffect, useMemo, useState, type MouseEvent as ReactMouseEvent } from "react";
import { ContextMenu, type ContextMenuItem } from "../components/ContextMenu";
import { formatIpcError } from "../lib/agent";
import { githubRepo, type GitHubRepoInfo } from "../lib/github";
import { klog } from "../lib/log";
import {
  faArrowDown,
  faArrowUp,
  faChevronDown,
  faChevronLeft,
  faChevronRight,
  faCheck,
  faCodeBranch,
  faMinus,
  faPlus,
  faRobot,
  faRotateLeft,
  faRotateRight,
} from "../lib/fontawesome";
import {
  changeBadge,
  commitHtmlUrl,
  gitCommit,
  gitCommitChanges,
  gitDiscard,
  gitLog,
  gitPull,
  gitPush,
  gitStage,
  gitStageAll,
  gitStatus,
  gitUnstage,
  type GitChange,
  type GitChangeKind,
  type GitCommit,
  type GitStatus,
} from "../lib/git";
import { Icon, isTauri, useToast } from "../shell";
import { useWorkbench } from "./state";

const KIND_COLOR: Record<GitChangeKind, string> = {
  modified: "text-nest-warning",
  added: "text-nest-success",
  deleted: "text-nest-error",
  renamed: "text-nest-accent",
  copied: "text-nest-accent",
  untracked: "text-nest-success",
  other: "text-nest-muted",
};

/** Source Control sidebar: commit box + changes, agent review, commit graph. */
export function SourceControlPanel({ onToggleCollapse }: { onToggleCollapse?: () => void }) {
  const { workspace, refreshToken, openFile, openCommit } = useWorkbench();
  const toast = useToast();

  const [status, setStatus] = useState<GitStatus | null>(null);
  const [commits, setCommits] = useState<GitCommit[]>([]);
  const [githubRemote, setGithubRemote] = useState<GitHubRepoInfo | null>(null);
  const [message, setMessage] = useState("");
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const refresh = useCallback(async () => {
    if (!isTauri()) {
      return;
    }
    klog("git", "refresh status + log");
    try {
      const [nextStatus, nextCommits, nextGithub] = await Promise.all([
        gitStatus(),
        gitLog(100),
        githubRepo(),
      ]);
      setStatus(nextStatus);
      setCommits(nextCommits);
      setGithubRemote(nextGithub);
      setError(null);
    } catch (caught) {
      setError(formatIpcError(caught));
    }
  }, []);

  useEffect(() => {
    void refresh();
  }, [refresh, workspace, refreshToken]);

  const staged = useMemo(
    () => (status?.changes ?? []).filter((change) => change.staged),
    [status],
  );
  const unstaged = useMemo(
    () => (status?.changes ?? []).filter((change) => change.unstaged),
    [status],
  );

  const run = useCallback(
    async (action: () => Promise<void>, refreshAfter = true) => {
      setBusy(true);
      try {
        await action();
        if (refreshAfter) {
          await refresh();
        }
      } catch (caught) {
        toast.error(formatIpcError(caught));
      } finally {
        setBusy(false);
      }
    },
    [refresh, toast],
  );

  const handleCommit = useCallback(() => {
    if (!message.trim()) {
      toast.info("Enter a commit message");
      return;
    }
    const stageAll = staged.length === 0;
    void run(async () => {
      await gitCommit(message, stageAll);
      setMessage("");
      toast.success("Committed");
    });
  }, [message, staged.length, run, toast]);

  const handlePush = useCallback(() => {
    void run(async () => {
      await gitPush();
      toast.success(status?.hasUpstream ? "Pushed" : "Branch published");
    });
  }, [run, status?.hasUpstream, toast]);

  const handlePull = useCallback(() => {
    void run(async () => {
      await gitPull();
      toast.success("Pulled");
    });
  }, [run, toast]);

  const handleOpenCommitChanges = useCallback(
    async (commit: GitCommit) => {
      setBusy(true);
      try {
        const changes = await gitCommitChanges(commit.hash);
        openCommit(changes);
      } catch (caught) {
        toast.error(formatIpcError(caught));
      } finally {
        setBusy(false);
      }
    },
    [openCommit, toast],
  );

  const handleOpenCommitOnGitHub = useCallback(
    (commit: GitCommit) => {
      if (!githubRemote) {
        toast.error("No GitHub origin remote configured");
        return;
      }
      window.open(commitHtmlUrl(githubRemote, commit.hash), "_blank", "noopener,noreferrer");
    },
    [githubRemote, toast],
  );

  if (!isTauri()) {
    return (
      <PanelFrame onRefresh={refresh} onToggleCollapse={onToggleCollapse}>
        <p className="px-3 py-2 text-xs text-nest-muted">
          Source control is available in the desktop app.
        </p>
      </PanelFrame>
    );
  }

  if (status && !status.isRepo) {
    return (
      <PanelFrame onRefresh={refresh} onToggleCollapse={onToggleCollapse}>
        <div className="flex flex-col items-center gap-2 px-4 py-8 text-center text-xs text-nest-muted">
          <Icon icon={faCodeBranch} className="size-5 opacity-40" />
          <p>The open folder is not a Git repository.</p>
        </div>
      </PanelFrame>
    );
  }

  return (
    <PanelFrame
      onRefresh={refresh}
      onToggleCollapse={onToggleCollapse}
      branch={status ?? undefined}
      onPush={handlePush}
      onPull={handlePull}
    >
      {error ? <p className="px-3 py-2 text-xs text-nest-error">{error}</p> : null}

      <div className="space-y-2 border-b border-nest-border p-2">
        <textarea
          value={message}
          onChange={(event) => setMessage(event.target.value)}
          placeholder={
            staged.length > 0
              ? "Message (commit staged changes)"
              : "Message (will commit all changes)"
          }
          rows={2}
          className="w-full resize-none rounded-nest-sm border border-nest-border bg-nest-surface px-2 py-1 text-xs text-nest-foreground placeholder:text-nest-muted focus:border-nest-accent focus:outline-none"
          onKeyDown={(event) => {
            if ((event.metaKey || event.ctrlKey) && event.key === "Enter") {
              event.preventDefault();
              handleCommit();
            }
          }}
        />
        <div className="flex gap-2">
          <button
            type="button"
            onClick={handleCommit}
            disabled={busy}
            className="flex flex-1 items-center justify-center gap-1.5 rounded-nest-sm bg-nest-primary px-2 py-1 text-xs font-medium text-white hover:opacity-90 disabled:opacity-50"
          >
            <Icon icon={faCheck} className="size-3" />
            {staged.length > 0 ? "Commit" : "Commit All"}
          </button>
          {status && (!status.hasUpstream || status.ahead > 0) ? (
            <button
              type="button"
              onClick={handlePush}
              disabled={busy}
              title={status.hasUpstream ? `Push ${status.ahead} commit(s)` : "Publish branch to origin"}
              className="inline-flex items-center gap-1 rounded-nest-sm border border-nest-border bg-nest-surface px-2 py-1 text-xs font-medium text-nest-foreground hover:bg-nest-muted/10 disabled:opacity-50"
            >
              <Icon icon={faArrowUp} className="size-3" />
              {status.hasUpstream ? `Push${status.ahead > 0 ? ` (${status.ahead})` : ""}` : "Publish"}
            </button>
          ) : null}
          {status && status.hasUpstream && status.behind > 0 ? (
            <button
              type="button"
              onClick={handlePull}
              disabled={busy}
              title={`Pull ${status.behind} commit(s)`}
              className="inline-flex items-center gap-1 rounded-nest-sm border border-nest-border bg-nest-surface px-2 py-1 text-xs font-medium text-nest-foreground hover:bg-nest-muted/10 disabled:opacity-50"
            >
              <Icon icon={faArrowDown} className="size-3" />
              Pull ({status.behind})
            </button>
          ) : null}
        </div>
      </div>

      <Section title="Staged Changes" count={staged.length} defaultOpen>
        {staged.map((change) => (
          <ChangeRow
            key={`s-${change.path}`}
            change={change}
            onOpen={() => openFile(change.path, baseName(change.path))}
            actions={[
              {
                icon: faMinus,
                title: "Unstage",
                onClick: () => run(() => gitUnstage(change.path)),
              },
            ]}
          />
        ))}
        {staged.length === 0 ? <Empty>No staged changes.</Empty> : null}
      </Section>

      <Section title="Changes" count={unstaged.length} defaultOpen>
        {unstaged.map((change) => (
          <ChangeRow
            key={`u-${change.path}`}
            change={change}
            onOpen={() => openFile(change.path, baseName(change.path))}
            actions={[
              {
                icon: faRotateLeft,
                title: "Discard changes",
                onClick: () => run(() => gitDiscard(change.path)),
                disabled: change.kind === "untracked",
              },
              {
                icon: faPlus,
                title: "Stage",
                onClick: () => run(() => gitStage(change.path)),
              },
            ]}
          />
        ))}
        {unstaged.length > 0 ? (
          <button
            type="button"
            onClick={() => run(() => gitStageAll())}
            className="mt-1 flex w-full items-center gap-1.5 px-3 py-1 text-[11px] text-nest-muted hover:text-nest-foreground"
          >
            <Icon icon={faPlus} className="size-2.5" />
            Stage all changes
          </button>
        ) : (
          <Empty>No changes.</Empty>
        )}
      </Section>

      <Section title="Agent Review" count={null} defaultOpen>
        <div className="flex items-center gap-2 px-3 py-2 text-[11px] text-nest-muted">
          <Icon icon={faRobot} className="size-3 opacity-60" />
          <span>AI review of your changes — coming soon.</span>
        </div>
      </Section>

      <Section title="Graph" count={commits.length} defaultOpen>
        <CommitGraph
          commits={commits}
          onOpenChanges={handleOpenCommitChanges}
          onOpenInGitHub={handleOpenCommitOnGitHub}
          githubAvailable={Boolean(githubRemote)}
        />
      </Section>
    </PanelFrame>
  );
}

// --- Commit graph ----------------------------------------------------------

function CommitGraph({
  commits,
  onOpenChanges,
  onOpenInGitHub,
  githubAvailable,
}: {
  commits: GitCommit[];
  onOpenChanges: (commit: GitCommit) => void | Promise<void>;
  onOpenInGitHub: (commit: GitCommit) => void;
  githubAvailable: boolean;
}) {
  const [hover, setHover] = useState<{ commit: GitCommit; x: number; y: number } | null>(null);
  const [menu, setMenu] = useState<{
    x: number;
    y: number;
    commit: GitCommit;
  } | null>(null);

  if (commits.length === 0) {
    return <Empty>No commits yet.</Empty>;
  }

  const onEnter = (event: ReactMouseEvent, commit: GitCommit) => {
    setHover({ commit, x: event.clientX, y: event.clientY });
  };

  const menuItems: ContextMenuItem[] = menu
    ? [
        {
          id: "open-changes",
          label: "Open Changes",
          onSelect: () => void onOpenChanges(menu.commit),
        },
        { kind: "separator", id: "sep-github" },
        {
          id: "open-github",
          label: "Open on GitHub",
          disabled: !githubAvailable,
          onSelect: () => onOpenInGitHub(menu.commit),
        },
      ]
    : [];

  return (
    <>
      <div className="py-1">
        {commits.map((commit, index) => {
          const isMerge = commit.parents.length > 1;
          const isLast = index === commits.length - 1;
          return (
            <div
              key={commit.hash}
              className="group flex cursor-default items-stretch gap-2 px-3 hover:bg-nest-muted/10"
              onMouseEnter={(event) => onEnter(event, commit)}
              onMouseMove={(event) => onEnter(event, commit)}
              onMouseLeave={() => setHover(null)}
              onContextMenu={(event) => {
                event.preventDefault();
                setHover(null);
                setMenu({ x: event.clientX, y: event.clientY, commit });
              }}
            >
            <div className="relative flex w-3 shrink-0 justify-center">
              {/* graph rail: vertical line + node dot */}
              <span className="absolute top-0 h-full w-px bg-nest-border" />
              {!isLast ? null : null}
              <span
                className={[
                  "relative z-10 mt-[9px] size-2 shrink-0 rounded-full border",
                  isMerge
                    ? "border-nest-accent bg-nest-background"
                    : "border-nest-accent bg-nest-accent",
                ].join(" ")}
              />
            </div>
            <div className="min-w-0 flex-1 py-1">
              <div className="flex items-center gap-2">
                <span className="truncate text-[13px] text-nest-foreground">{commit.subject}</span>
              </div>
              <div className="flex items-center gap-2 text-[10px] text-nest-muted">
                <span className="font-mono">{commit.shortHash}</span>
                <span className="truncate">{commit.author}</span>
                <span className="ml-auto shrink-0">{commit.relativeDate}</span>
              </div>
            </div>
          </div>
        );
      })}
      {hover ? <CommitTooltip commit={hover.commit} x={hover.x} y={hover.y} /> : null}
      </div>
      {menu ? (
        <ContextMenu x={menu.x} y={menu.y} items={menuItems} onClose={() => setMenu(null)} />
      ) : null}
    </>
  );
}

function CommitTooltip({ commit, x, y }: { commit: GitCommit; x: number; y: number }) {
  // Keep the card inside the viewport (clamp near the right/bottom edges).
  const width = 300;
  const left = Math.min(x + 14, window.innerWidth - width - 8);
  const top = Math.min(y + 14, window.innerHeight - 140);
  return (
    <div
      className="pointer-events-none fixed z-50 rounded-nest-md border border-nest-border bg-nest-surface p-3 shadow-lg"
      style={{ left, top, width }}
    >
      <p className="mb-1 text-[13px] font-medium text-nest-foreground">{commit.subject}</p>
      <dl className="space-y-0.5 text-[11px] text-nest-muted">
        <div className="flex gap-2">
          <dt className="w-14 shrink-0 opacity-70">Commit</dt>
          <dd className="font-mono text-nest-foreground/80">{commit.shortHash}</dd>
        </div>
        <div className="flex gap-2">
          <dt className="w-14 shrink-0 opacity-70">Author</dt>
          <dd className="truncate">
            {commit.author}
            {commit.email ? ` <${commit.email}>` : ""}
          </dd>
        </div>
        <div className="flex gap-2">
          <dt className="w-14 shrink-0 opacity-70">Date</dt>
          <dd>
            {commit.date} ({commit.relativeDate})
          </dd>
        </div>
        {commit.parents.length > 1 ? (
          <div className="flex gap-2">
            <dt className="w-14 shrink-0 opacity-70">Merge</dt>
            <dd className="font-mono">{commit.parents.map((p) => p.slice(0, 7)).join(" ")}</dd>
          </div>
        ) : null}
      </dl>
    </div>
  );
}

// --- Building blocks -------------------------------------------------------

type RowAction = {
  icon: typeof faPlus;
  title: string;
  onClick: () => void;
  disabled?: boolean;
};

function ChangeRow({
  change,
  onOpen,
  actions,
}: {
  change: GitChange;
  onOpen: () => void;
  actions: RowAction[];
}) {
  return (
    <div className="group flex items-center gap-2 px-3 py-0.5 hover:bg-nest-muted/10">
      <button
        type="button"
        onClick={onOpen}
        title={change.path}
        className="flex min-w-0 flex-1 items-center gap-2 text-left"
      >
        <span className="truncate text-[13px] text-nest-foreground">{baseName(change.path)}</span>
        <span className="truncate text-[10px] text-nest-muted">{dirName(change.path)}</span>
      </button>
      <div className="flex shrink-0 items-center gap-0.5 opacity-0 group-hover:opacity-100">
        {actions.map((action) => (
          <button
            key={action.title}
            type="button"
            onClick={action.onClick}
            disabled={action.disabled}
            title={action.title}
            aria-label={action.title}
            className="flex size-5 items-center justify-center rounded-nest-sm text-nest-muted hover:bg-nest-muted/20 hover:text-nest-foreground disabled:opacity-30"
          >
            <Icon icon={action.icon} className="size-2.5" />
          </button>
        ))}
      </div>
      <span
        className={["w-3 shrink-0 text-center text-[11px] font-semibold", KIND_COLOR[change.kind]].join(
          " ",
        )}
        title={change.kind}
      >
        {changeBadge(change.kind)}
      </span>
    </div>
  );
}

function Section({
  title,
  count,
  defaultOpen,
  children,
}: {
  title: string;
  count: number | null;
  defaultOpen?: boolean;
  children: React.ReactNode;
}) {
  const [open, setOpen] = useState(defaultOpen ?? true);
  return (
    <div className="border-b border-nest-border">
      <button
        type="button"
        onClick={() => setOpen((value) => !value)}
        className="flex w-full items-center gap-1.5 px-2 py-1 text-[11px] font-semibold uppercase tracking-wide text-nest-muted hover:text-nest-foreground"
      >
        <Icon icon={open ? faChevronDown : faChevronRight} className="size-2.5" />
        <span>{title}</span>
        {count !== null ? (
          <span className="ml-auto rounded-full bg-nest-muted/20 px-1.5 text-[10px] font-medium">
            {count}
          </span>
        ) : null}
      </button>
      {open ? <div className="pb-1">{children}</div> : null}
    </div>
  );
}

function Empty({ children }: { children: React.ReactNode }) {
  return <p className="px-3 py-1 text-[11px] text-nest-muted">{children}</p>;
}

function PanelFrame({
  children,
  onRefresh,
  onToggleCollapse,
  branch,
  onPush,
  onPull,
}: {
  children: React.ReactNode;
  onRefresh: () => void;
  onToggleCollapse?: () => void;
  branch?: GitStatus;
  onPush?: () => void;
  onPull?: () => void;
}) {
  return (
    <div className="flex h-full min-h-0 flex-col bg-nest-background">
      <header className="flex h-9 shrink-0 items-center gap-1 border-b border-nest-border px-3">
        <span className="text-xs font-semibold uppercase tracking-wide text-nest-muted">
          Source Control
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
      {branch ? (
        <div className="flex h-7 shrink-0 items-center gap-2 border-b border-nest-border px-3 text-[11px] text-nest-muted">
          <Icon icon={faCodeBranch} className="size-3" />
          <span className="truncate text-nest-foreground/80">{branch.branch}</span>
          {branch.hasUpstream && (branch.ahead > 0 || branch.behind > 0) ? (
            <span className="ml-auto flex items-center gap-2">
              {branch.behind > 0 ? (
                <button
                  type="button"
                  onClick={onPull}
                  title={`Pull ${branch.behind} commit(s)`}
                  className="flex items-center gap-0.5 hover:text-nest-foreground"
                >
                  <Icon icon={faArrowDown} className="size-2.5" />
                  {branch.behind}
                </button>
              ) : null}
              {branch.ahead > 0 ? (
                <button
                  type="button"
                  onClick={onPush}
                  title={`Push ${branch.ahead} commit(s)`}
                  className="flex items-center gap-0.5 hover:text-nest-foreground"
                >
                  <Icon icon={faArrowUp} className="size-2.5" />
                  {branch.ahead}
                </button>
              ) : null}
            </span>
          ) : null}
        </div>
      ) : null}
      <div className="min-h-0 flex-1 overflow-auto">{children}</div>
    </div>
  );
}

function baseName(path: string): string {
  const parts = path.split("/");
  return parts[parts.length - 1] || path;
}

function dirName(path: string): string {
  const idx = path.lastIndexOf("/");
  return idx > 0 ? path.slice(0, idx) : "";
}
