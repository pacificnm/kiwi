import { useCallback, useEffect, useMemo, useState, type MouseEvent as ReactMouseEvent } from "react";
import { ContextMenu, type ContextMenuItem } from "../../components/ContextMenu";
import { formatIpcError } from "../../lib/agent";
import {
  faChevronLeft,
  faCircle,
  faCircleCheck,
  faCircleExclamation,
  faCodeBranch,
  faComment,
  faMagnifyingGlass,
  faPlus,
  faRotateRight,
  faCalendar,
  faTag,
} from "../../lib/fontawesome";
import {
  githubAuthStatus,
  githubIssueList,
  githubIssueView,
  githubRepo,
  issueHtmlUrl,
  type GitHubAuthStatus,
  type GitHubIssueListItem,
  type GitHubRepoInfo,
} from "../../lib/github";
import { Icon, isTauri, useToast } from "../../shell";
import { useIssuesActions, useIssuesRefreshToken } from "./issuesActions";
import { filterIssues, formatRelativeTime, labelStyle } from "./issueUtils";
import { useWorkbench } from "../state";

type IssueStateFilter = "open" | "closed";
type PanelTab = "issues" | "pulls";

type IssuesPanelProps = {
  onToggleCollapse?: () => void;
};

export function IssuesPanel({ onToggleCollapse }: IssuesPanelProps) {
  const { workspace, openIssue } = useWorkbench();
  const { openNewComment, openNewIssue, openManageLabels, openManageMilestones } =
    useIssuesActions();
  const issuesRefreshToken = useIssuesRefreshToken();
  const toast = useToast();

  const [auth, setAuth] = useState<GitHubAuthStatus | null>(null);
  const [repo, setRepo] = useState<GitHubRepoInfo | null>(null);
  const [openIssues, setOpenIssues] = useState<GitHubIssueListItem[]>([]);
  const [closedIssues, setClosedIssues] = useState<GitHubIssueListItem[]>([]);
  const [issueState, setIssueState] = useState<IssueStateFilter>("open");
  const [panelTab, setPanelTab] = useState<PanelTab>("issues");
  const [search, setSearch] = useState("");
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [menu, setMenu] = useState<{
    x: number;
    y: number;
    issue: GitHubIssueListItem;
  } | null>(null);

  const refresh = useCallback(async () => {
    if (!isTauri()) {
      return;
    }
    setBusy(true);
    try {
      const [nextAuth, nextRepo] = await Promise.all([githubAuthStatus(), githubRepo()]);
      setAuth(nextAuth);
      setRepo(nextRepo);
      if (nextRepo) {
        const [openIssues, closedIssues] = await Promise.all([
          githubIssueList("open", 100),
          githubIssueList("closed", 100),
        ]);
        setOpenIssues(openIssues);
        setClosedIssues(closedIssues);
      } else {
        setOpenIssues([]);
        setClosedIssues([]);
      }
      setError(null);
    } catch (caught) {
      setError(formatIpcError(caught));
    } finally {
      setBusy(false);
    }
  }, []);

  useEffect(() => {
    void refresh();
  }, [refresh, workspace, issuesRefreshToken]);

  const issues = issueState === "open" ? openIssues : closedIssues;
  const openCount = openIssues.length;
  const closedCount = closedIssues.length;
  const filteredIssues = useMemo(() => filterIssues(issues, search), [issues, search]);

  const openIssueTab = useCallback(
    async (number: number) => {
      try {
        const issue = await githubIssueView(number);
        openIssue(issue);
      } catch (caught) {
        toast.error(formatIpcError(caught));
      }
    },
    [openIssue, toast],
  );

  const onIssueContextMenu = useCallback(
    (event: ReactMouseEvent, issue: GitHubIssueListItem) => {
      event.preventDefault();
      setMenu({ x: event.clientX, y: event.clientY, issue });
    },
    [],
  );

  const copyIssueLink = useCallback(
    (issue: GitHubIssueListItem) => {
      if (!repo) {
        toast.error("No GitHub repository configured");
        return;
      }
      const url = issueHtmlUrl(repo, issue.number);
      void navigator.clipboard
        .writeText(url)
        .then(() => toast.success("Copied issue link"))
        .catch((error) => toast.error(formatIpcError(error)));
    },
    [repo, toast],
  );

  const menuItems: ContextMenuItem[] = menu
    ? [
        {
          id: "new-comment",
          label: "New Comment…",
          onSelect: () => openNewComment(menu.issue.number),
        },
        { kind: "separator", id: "sep-copy" },
        {
          id: "copy-link",
          label: "Copy Link",
          disabled: !repo,
          onSelect: () => copyIssueLink(menu.issue),
        },
      ]
    : [];

  if (!isTauri()) {
    return (
      <PanelFrame onToggleCollapse={onToggleCollapse}>
        <p className="px-3 py-2 text-xs text-nest-muted">
          GitHub Issues are available in the desktop app.
        </p>
      </PanelFrame>
    );
  }

  return (
    <>
      <PanelFrame onRefresh={() => void refresh()} onToggleCollapse={onToggleCollapse}>
        {error ? <p className="px-3 py-2 text-xs text-nest-error">{error}</p> : null}

        <div className="border-b border-nest-border px-3 py-2 text-[11px] text-nest-muted">
          {auth?.authenticated ? (
            <p className="truncate">Signed in as {auth.login ?? "GitHub"}</p>
          ) : (
            <p>Not signed in — run `gh auth login`</p>
          )}
          {repo ? (
            <a
              href={repo.htmlUrl}
              className="mt-0.5 block truncate font-medium text-nest-accent hover:underline"
              target="_blank"
              rel="noreferrer"
            >
              {repo.repo}
            </a>
          ) : (
            <p className="mt-0.5">No GitHub origin remote</p>
          )}
        </div>

        <div className="flex border-b border-nest-border text-xs">
          <TabButton
            active={panelTab === "issues"}
            onClick={() => setPanelTab("issues")}
            label="Issues"
          />
          <TabButton
            active={panelTab === "pulls"}
            onClick={() => setPanelTab("pulls")}
            label="Pull requests"
          />
        </div>

        {panelTab === "issues" ? (
          <>
            <div className="space-y-2 border-b border-nest-border px-3 py-2">
              <label className="relative block">
                <Icon
                  icon={faMagnifyingGlass}
                  className="pointer-events-none absolute left-2 top-1/2 size-3 -translate-y-1/2 text-nest-muted"
                />
                <input
                  type="search"
                  value={search}
                  onChange={(event) => setSearch(event.target.value)}
                  placeholder="Search issues"
                  className="w-full rounded-nest-md border border-nest-border bg-nest-surface py-1.5 pl-7 pr-2 text-xs text-nest-foreground placeholder:text-nest-muted focus:border-nest-accent focus:outline-none"
                />
              </label>

              <div className="flex flex-wrap items-center gap-1">
                <ToolbarButton
                  label="Labels"
                  icon={faTag}
                  onClick={() => void openManageLabels()}
                />
                <ToolbarButton
                  label="Milestones"
                  icon={faCalendar}
                  onClick={() => void openManageMilestones()}
                />
                <button
                  type="button"
                  onClick={() => openNewIssue()}
                  className="ml-auto inline-flex items-center gap-1 rounded-nest-md bg-green-600 px-2.5 py-1 text-[11px] font-medium text-white hover:bg-green-500"
                >
                  <Icon icon={faPlus} className="size-3" />
                  New issue
                </button>
              </div>
            </div>

            <div className="flex border-b border-nest-border text-xs">
              <StateTab
                active={issueState === "open"}
                count={openCount}
                label="Open"
                onClick={() => setIssueState("open")}
              />
              <StateTab
                active={issueState === "closed"}
                count={closedCount}
                label="Closed"
                onClick={() => setIssueState("closed")}
              />
            </div>

            {busy && filteredIssues.length === 0 ? (
              <p className="px-3 py-4 text-xs text-nest-muted">Loading issues…</p>
            ) : filteredIssues.length === 0 ? (
              <div className="flex flex-col items-center gap-2 px-4 py-8 text-center text-xs text-nest-muted">
                <Icon icon={faCircleExclamation} className="size-5 opacity-40" />
                <p>{search.trim() ? "No issues match your search" : `No ${issueState} issues`}</p>
              </div>
            ) : (
              <ul className="divide-y divide-nest-border/60">
                {filteredIssues.map((issue) => (
                  <IssueRow
                    key={issue.number}
                    issue={issue}
                    onOpen={() => void openIssueTab(issue.number)}
                    onContextMenu={(event) => onIssueContextMenu(event, issue)}
                  />
                ))}
              </ul>
            )}
          </>
        ) : (
          <div className="flex flex-col items-center gap-2 px-4 py-10 text-center text-xs text-nest-muted">
            <Icon icon={faCodeBranch} className="size-5 opacity-40" />
            <p className="font-medium text-nest-foreground">Pull requests</p>
            <p>PR list and merge actions are coming soon.</p>
          </div>
        )}
      </PanelFrame>

      {menu ? (
        <ContextMenu x={menu.x} y={menu.y} items={menuItems} onClose={() => setMenu(null)} />
      ) : null}
    </>
  );
}

function IssueRow({
  issue,
  onOpen,
  onContextMenu,
}: {
  issue: GitHubIssueListItem;
  onOpen: () => void;
  onContextMenu: (event: ReactMouseEvent) => void;
}) {
  const isOpen = issue.state.toLowerCase() === "open";
  const author = issue.author?.login ?? "unknown";

  return (
    <li>
      <button
        type="button"
        onClick={onOpen}
        onContextMenu={onContextMenu}
        className="flex w-full gap-2 px-3 py-2.5 text-left hover:bg-nest-muted/10"
      >
        <span className="mt-0.5 shrink-0">
          <Icon
            icon={isOpen ? faCircle : faCircleCheck}
            className={[
              "size-3",
              isOpen ? "text-green-500" : "text-purple-500",
            ].join(" ")}
            title={isOpen ? "Open" : "Closed"}
          />
        </span>
        <span className="min-w-0 flex-1">
          <span className="block text-xs font-medium leading-snug text-nest-foreground">
            {issue.title}
          </span>
          <span className="mt-0.5 block text-[11px] text-nest-muted">
            #{issue.number} opened by {author} · {formatRelativeTime(issue.createdAt)}
          </span>
          {issue.labels.length > 0 ? (
            <span className="mt-1.5 flex flex-wrap gap-1">
              {issue.labels.slice(0, 5).map((label) => (
                <span
                  key={label.name}
                  className="inline-flex rounded-full px-1.5 py-0.5 text-[10px] font-medium"
                  style={labelStyle(label)}
                >
                  {label.name}
                </span>
              ))}
            </span>
          ) : null}
        </span>
        {issue.comments > 0 ? (
          <span className="flex shrink-0 items-center gap-1 self-start pt-0.5 text-[11px] text-nest-muted">
            <Icon icon={faComment} className="size-3" />
            {issue.comments}
          </span>
        ) : null}
      </button>
    </li>
  );
}

function TabButton({
  active,
  label,
  onClick,
}: {
  active: boolean;
  label: string;
  onClick: () => void;
}) {
  return (
    <button
      type="button"
      onClick={onClick}
      className={[
        "border-b-2 px-3 py-2 font-medium transition-colors",
        active
          ? "border-nest-accent text-nest-foreground"
          : "border-transparent text-nest-muted hover:text-nest-foreground",
      ].join(" ")}
    >
      {label}
    </button>
  );
}

function StateTab({
  active,
  count,
  label,
  onClick,
}: {
  active: boolean;
  count: number;
  label: string;
  onClick: () => void;
}) {
  return (
    <button
      type="button"
      onClick={onClick}
      className={[
        "flex-1 border-b-2 px-3 py-2 text-left font-medium transition-colors",
        active
          ? "border-nest-accent text-nest-foreground"
          : "border-transparent text-nest-muted hover:text-nest-foreground",
      ].join(" ")}
    >
      {label}
      <span className="ml-1 rounded-full bg-nest-muted/15 px-1.5 py-0.5 text-[10px] tabular-nums">
        {count}
      </span>
    </button>
  );
}

function ToolbarButton({
  label,
  icon,
  onClick,
}: {
  label: string;
  icon: typeof faTag;
  onClick: () => void;
}) {
  return (
    <button
      type="button"
      onClick={onClick}
      className="inline-flex items-center gap-1 rounded-nest-md border border-nest-border px-2 py-1 text-[11px] text-nest-muted hover:bg-nest-muted/10 hover:text-nest-foreground"
    >
      <Icon icon={icon} className="size-3" />
      {label}
    </button>
  );
}

function PanelFrame({
  children,
  onRefresh,
  onToggleCollapse,
}: {
  children: React.ReactNode;
  onRefresh?: () => void;
  onToggleCollapse?: () => void;
}) {
  return (
    <div className="flex h-full min-h-0 flex-col bg-nest-background">
      <header className="flex h-9 shrink-0 items-center gap-1 border-b border-nest-border px-3">
        <span className="truncate text-xs font-semibold uppercase tracking-wide text-nest-muted">
          Issues
        </span>
        <div className="ml-auto flex items-center gap-0.5">
          {onRefresh ? (
            <button
              type="button"
              onClick={onRefresh}
              title="Refresh"
              aria-label="Refresh"
              className="flex size-6 items-center justify-center rounded-nest-sm text-nest-muted hover:bg-nest-muted/10 hover:text-nest-foreground"
            >
              <Icon icon={faRotateRight} className="size-3" />
            </button>
          ) : null}
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
      <div className="min-h-0 flex-1 overflow-auto">{children}</div>
    </div>
  );
}
