import { useState } from "react";
import { Menu, MenuItem } from "@nest/components";
import { Icon, useToast } from "../../shell";
import {
  faChevronDown,
  faCircle,
  faCircleCheck,
  faComment,
  faDiagramProject,
  faLink,
  faSignsPost,
  faUser,
} from "../../lib/fontawesome";
import { formatIpcError } from "../../lib/agent";
import {
  githubIssueComment,
  githubIssueView,
  type GitHubIssue,
  type GitHubIssueDependency,
} from "../../lib/github";
import { useWorkbench } from "../state";
import { CreateBranchModal } from "./CreateBranchModal";
import { IssueMarkdown } from "./IssueMarkdown";
import { formatIssueDate, formatRelativeTime, labelStyle } from "./issueUtils";

type IssueDetailViewProps = {
  issue: GitHubIssue;
};

export function IssueDetailView({ issue }: IssueDetailViewProps) {
  const toast = useToast();
  const isOpen = issue.state.toLowerCase() === "open";
  const author = issue.author?.login ?? "unknown";
  const [menuOpen, setMenuOpen] = useState(false);
  const [branchModalOpen, setBranchModalOpen] = useState(false);

  const copyLink = async () => {
    try {
      await navigator.clipboard.writeText(issue.htmlUrl);
      toast.success("Issue link copied");
    } catch {
      toast.error("Could not copy link");
    }
  };

  return (
    <div className="h-full min-h-0 overflow-auto bg-nest-background">
      <div className="mx-auto max-w-4xl px-6 py-6">
        <header className="border-b border-nest-border pb-4">
          <h1 className="text-2xl font-semibold leading-snug text-nest-foreground">
            {issue.title}{" "}
            <span className="font-normal text-nest-muted">#{issue.number}</span>
          </h1>

          <div className="mt-3 flex flex-wrap items-center gap-2">
            <StateBadge open={isOpen} />
            {issue.labels.map((label) => (
              <span
                key={label.name}
                className="inline-flex rounded-full px-2 py-0.5 text-xs font-medium"
                style={labelStyle(label)}
              >
                {label.name}
              </span>
            ))}
          </div>

          {issue.milestone ? (
            <div className="mt-2 flex flex-wrap items-center gap-1.5 text-sm text-nest-muted">
              <Icon icon={faSignsPost} className="size-3 opacity-70" />
              <span className="font-medium text-nest-foreground">{issue.milestone.title}</span>
              {issue.milestone.dueOn ? (
                <span className="text-xs">· due {formatIssueDate(issue.milestone.dueOn)}</span>
              ) : null}
            </div>
          ) : null}

          {issue.blockedBy.length > 0 ? (
            <DependencyRow label="Blocked by" items={issue.blockedBy} />
          ) : null}
          {issue.blocking.length > 0 ? (
            <DependencyRow label="Blocking" items={issue.blocking} />
          ) : null}

          <p className="mt-2 text-sm text-nest-muted">
            <Icon icon={faUser} className="mr-1 inline size-3 opacity-70" />
            <span className="font-medium text-nest-foreground">{author}</span> opened this issue on{" "}
            {formatIssueDate(issue.createdAt)}
            {issue.comments > 0 ? (
              <>
                {" "}
                · <Icon icon={faComment} className="mx-0.5 inline size-3 opacity-70" />
                {issue.comments} comment{issue.comments === 1 ? "" : "s"}
              </>
            ) : null}
            {" · "}updated {formatRelativeTime(issue.updatedAt)}
          </p>

          <div className="mt-4 flex flex-wrap items-center gap-2">
            <a
              href={issue.htmlUrl}
              target="_blank"
              rel="noreferrer"
              className="inline-flex items-center gap-1.5 rounded-nest-md border border-nest-border bg-nest-surface px-3 py-1.5 text-xs font-medium text-nest-foreground hover:bg-nest-muted/10"
            >
              <Icon icon={faLink} className="size-3" />
              Open on GitHub
            </a>
            <div className="relative inline-block">
              <button
                type="button"
                onClick={() => setMenuOpen((value) => !value)}
                aria-haspopup="menu"
                aria-expanded={menuOpen}
                className="inline-flex items-center gap-1.5 rounded-nest-md border border-nest-border bg-nest-surface px-3 py-1.5 text-xs font-medium text-nest-foreground hover:bg-nest-muted/10"
              >
                Actions
                <Icon icon={faChevronDown} className="size-2.5" />
              </button>
              <Menu open={menuOpen} onClose={() => setMenuOpen(false)}>
                <MenuItem
                  onClick={() => {
                    setMenuOpen(false);
                    void copyLink();
                  }}
                >
                  Copy Link
                </MenuItem>
                <MenuItem
                  onClick={() => {
                    setMenuOpen(false);
                    setBranchModalOpen(true);
                  }}
                >
                  Create Branch
                </MenuItem>
              </Menu>
            </div>
          </div>
        </header>

        <article className="border-b border-nest-border py-6">
          <div className="flex gap-3">
            <div className="flex size-8 shrink-0 items-center justify-center rounded-full bg-nest-muted/15 text-xs font-semibold uppercase text-nest-muted">
              {author.slice(0, 1)}
            </div>
            <div className="min-w-0 flex-1">
              <div className="mb-3 flex items-baseline gap-2 text-sm">
                <span className="font-semibold text-nest-foreground">{author}</span>
                <span className="text-nest-muted">commented {formatRelativeTime(issue.createdAt)}</span>
              </div>
              <IssueMarkdown markdown={issue.body} />
            </div>
          </div>
        </article>

        {issue.comments > 0 ? (
          <section className="border-b border-nest-border py-6">
            <h2 className="text-sm font-semibold text-nest-foreground">
              {issue.comments} comment{issue.comments === 1 ? "" : "s"}
            </h2>
            <p className="mt-2 text-sm text-nest-muted">
              View the full conversation on{" "}
              <a href={issue.htmlUrl} className="text-nest-accent hover:underline" target="_blank" rel="noreferrer">
                GitHub
              </a>
              .
            </p>
          </section>
        ) : null}

        <CommentComposer issue={issue} />
      </div>
      {branchModalOpen ? (
        <CreateBranchModal issue={issue} onClose={() => setBranchModalOpen(false)} />
      ) : null}
    </div>
  );
}

function CommentComposer({ issue }: { issue: GitHubIssue }) {
  const toast = useToast();
  const { openIssue } = useWorkbench();
  const [body, setBody] = useState("");
  const [submitting, setSubmitting] = useState(false);

  const canSubmit = body.trim().length > 0 && !submitting;

  const submit = async () => {
    if (!canSubmit) {
      return;
    }
    setSubmitting(true);
    try {
      await githubIssueComment(issue.number, body.trim());
      setBody("");
      toast.success(`Comment posted on #${issue.number}`);
      // Best-effort refresh so the comment count updates in place.
      try {
        openIssue(await githubIssueView(issue.number));
      } catch {
        // Comment already posted; refresh is non-critical.
      }
    } catch (error) {
      toast.error(formatIpcError(error));
    } finally {
      setSubmitting(false);
    }
  };

  return (
    <section className="py-6">
      <div className="flex gap-3">
        <div className="flex size-8 shrink-0 items-center justify-center rounded-full bg-nest-muted/15 text-nest-muted">
          <Icon icon={faComment} className="size-3.5" />
        </div>
        <div className="min-w-0 flex-1">
          <label htmlFor="issue-comment-composer" className="mb-2 block text-sm font-semibold text-nest-foreground">
            Add a comment
          </label>
          <textarea
            id="issue-comment-composer"
            value={body}
            onChange={(event) => setBody(event.target.value)}
            onKeyDown={(event) => {
              if ((event.metaKey || event.ctrlKey) && event.key === "Enter") {
                event.preventDefault();
                void submit();
              }
            }}
            rows={4}
            placeholder="Leave a comment"
            className="w-full resize-y rounded-nest-md border border-nest-border bg-nest-background px-3 py-2 text-sm text-nest-foreground placeholder:text-nest-muted focus:outline-none focus:ring-2 focus:ring-nest-accent/50"
          />
          <div className="mt-2 flex items-center gap-2">
            <span className="mr-auto text-xs text-nest-muted">Cmd/Ctrl+Enter to submit</span>
            <button
              type="button"
              disabled={!canSubmit}
              onClick={() => void submit()}
              className="inline-flex items-center gap-1.5 rounded-nest-md bg-nest-accent px-3 py-1.5 text-xs font-semibold text-nest-background disabled:opacity-50"
            >
              <Icon icon={faComment} className="size-3" />
              {submitting ? "Posting…" : "Comment"}
            </button>
          </div>
        </div>
      </div>
    </section>
  );
}

function DependencyRow({ label, items }: { label: string; items: GitHubIssueDependency[] }) {
  return (
    <div className="mt-2 flex flex-wrap items-center gap-x-2 gap-y-1 text-sm text-nest-muted">
      <span className="inline-flex items-center gap-1.5">
        <Icon icon={faDiagramProject} className="size-3 opacity-70" />
        <span className="font-medium text-nest-foreground">{label}</span>
      </span>
      {items.map((dep) => {
        const closed = dep.state.toLowerCase() === "closed";
        return (
          <a
            key={dep.number}
            href={dep.htmlUrl}
            target="_blank"
            rel="noreferrer"
            title={`#${dep.number} ${dep.title}`}
            className="inline-flex max-w-[18rem] items-center gap-1 rounded border border-nest-border bg-nest-surface px-1.5 py-0.5 text-xs hover:bg-nest-muted/10"
          >
            <span className={closed ? "text-nest-muted line-through" : "text-nest-foreground"}>
              #{dep.number}
            </span>
            <span className="truncate text-nest-muted">{dep.title}</span>
          </a>
        );
      })}
    </div>
  );
}

function StateBadge({ open }: { open: boolean }) {
  if (open) {
    return (
      <span className="inline-flex items-center gap-1 rounded-full border border-green-500/30 bg-green-500/10 px-2 py-0.5 text-xs font-medium text-green-600 dark:text-green-400">
        <Icon icon={faCircle} className="size-2" />
        Open
      </span>
    );
  }
  return (
    <span className="inline-flex items-center gap-1 rounded-full border border-purple-500/30 bg-purple-500/10 px-2 py-0.5 text-xs font-medium text-purple-600 dark:text-purple-400">
      <Icon icon={faCircleCheck} className="size-2.5" />
      Closed
    </span>
  );
}
