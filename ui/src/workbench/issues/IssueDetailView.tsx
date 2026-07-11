import { Icon } from "../../shell";
import {
  faCircle,
  faCircleCheck,
  faComment,
  faLink,
  faUser,
} from "../../lib/fontawesome";
import type { GitHubIssue } from "../../lib/github";
import { useIssuesActions } from "./issuesActions";
import { IssueMarkdown } from "./IssueMarkdown";
import { formatIssueDate, formatRelativeTime, labelStyle } from "./issueUtils";

type IssueDetailViewProps = {
  issue: GitHubIssue;
};

export function IssueDetailView({ issue }: IssueDetailViewProps) {
  const { openNewComment } = useIssuesActions();
  const isOpen = issue.state.toLowerCase() === "open";
  const author = issue.author?.login ?? "unknown";

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

          <div className="mt-4 flex flex-wrap gap-2">
            <button
              type="button"
              onClick={() => openNewComment(issue.number)}
              className="inline-flex items-center gap-1.5 rounded-nest-md border border-nest-border bg-nest-surface px-3 py-1.5 text-xs font-medium text-nest-foreground hover:bg-nest-muted/10"
            >
              <Icon icon={faComment} className="size-3" />
              Comment
            </button>
            <a
              href={issue.htmlUrl}
              target="_blank"
              rel="noreferrer"
              className="inline-flex items-center gap-1.5 rounded-nest-md border border-nest-border bg-nest-surface px-3 py-1.5 text-xs font-medium text-nest-foreground hover:bg-nest-muted/10"
            >
              <Icon icon={faLink} className="size-3" />
              Open on GitHub
            </a>
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
          <section className="py-6">
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
      </div>
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
