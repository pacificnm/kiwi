import { kiwiInvoke } from "./ipc";

export type GitHubAuthStatus = {
  authenticated: boolean;
  login: string | null;
};

export type GitHubRepoInfo = {
  repo: string;
  htmlUrl: string;
};

export type GitHubLabel = {
  name: string;
  color: string | null;
  description: string | null;
};

export type GitHubMilestone = {
  number: number;
  title: string;
  state: string;
  description: string | null;
  dueOn: string | null;
};

export type GitHubUser = {
  login: string;
};

export type GitHubIssueListItem = {
  number: number;
  title: string;
  state: string;
  labels: GitHubLabel[];
  updatedAt: string;
  createdAt: string;
  author: GitHubUser | null;
  comments: number;
};

export type GitHubIssue = {
  number: number;
  title: string;
  body: string;
  state: string;
  labels: GitHubLabel[];
  htmlUrl: string;
  createdAt: string;
  updatedAt: string;
  author: GitHubUser | null;
  comments: number;
};

export type GitHubIssueActionResult = {
  number: number;
  htmlUrl: string;
};

/** Virtual editor tab key for a GitHub issue. */
export function issueTabKey(number: number): string {
  return `github-issue:#${number}`;
}

export function isIssueTab(relPath: string): boolean {
  return relPath.startsWith("github-issue:#");
}

export function issueNumberFromTab(relPath: string): number | null {
  if (!isIssueTab(relPath)) {
    return null;
  }
  const raw = relPath.slice("github-issue:#".length);
  const parsed = Number.parseInt(raw, 10);
  return Number.isFinite(parsed) ? parsed : null;
}

export function issueHtmlUrl(repo: GitHubRepoInfo, number: number): string {
  return `${repo.htmlUrl}/issues/${number}`;
}

export async function githubAuthStatus(): Promise<GitHubAuthStatus> {
  return kiwiInvoke<GitHubAuthStatus>("github_auth_status");
}

export async function githubRepo(): Promise<GitHubRepoInfo | null> {
  return kiwiInvoke<GitHubRepoInfo | null>("github_repo");
}

export async function githubIssueList(
  state: "open" | "closed" | "all" = "open",
  limit = 100,
): Promise<GitHubIssueListItem[]> {
  return kiwiInvoke<GitHubIssueListItem[]>("github_issue_list", { state, limit });
}

export async function githubIssueView(number: number): Promise<GitHubIssue> {
  return kiwiInvoke<GitHubIssue>("github_issue_view", { number });
}

export async function githubIssueCreate(
  title: string,
  body: string,
): Promise<GitHubIssueActionResult> {
  return kiwiInvoke<GitHubIssueActionResult>("github_issue_create", { title, body });
}

export async function githubIssueComment(
  number: number,
  body: string,
): Promise<GitHubIssueActionResult> {
  return kiwiInvoke<GitHubIssueActionResult>("github_issue_comment", { number, body });
}

export async function githubLabelList(): Promise<GitHubLabel[]> {
  return kiwiInvoke<GitHubLabel[]>("github_label_list");
}

export async function githubMilestoneList(): Promise<GitHubMilestone[]> {
  return kiwiInvoke<GitHubMilestone[]>("github_milestone_list");
}
