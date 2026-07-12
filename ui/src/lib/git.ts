import { kiwiInvoke } from "./ipc";
import type { WorkspaceInfo } from "./workspace";

/** Kind of working-tree change (drives the badge letter). */
export type GitChangeKind =
  | "modified"
  | "added"
  | "deleted"
  | "renamed"
  | "copied"
  | "untracked"
  | "other";

/** One changed path in the repository. */
export type GitChange = {
  path: string;
  staged: boolean;
  unstaged: boolean;
  kind: GitChangeKind;
};

/** Parsed repository status. */
export type GitStatus = {
  isRepo: boolean;
  branch: string;
  changes: GitChange[];
  ahead: number;
  behind: number;
  hasUpstream: boolean;
};

/** One commit in the history graph. */
export type GitCommit = {
  hash: string;
  shortHash: string;
  author: string;
  email: string;
  date: string;
  relativeDate: string;
  subject: string;
  parents: string[];
};

export type GitCommitFileChange = {
  path: string;
  oldPath: string | null;
  status: string;
  diff: string;
};

export type GitCommitChanges = {
  hash: string;
  shortHash: string;
  subject: string;
  files: GitCommitFileChange[];
};

/** Virtual editor tab key for a commit diff view. */
export function commitTabKey(hash: string): string {
  return `git-commit:${hash}`;
}

export function isCommitTab(relPath: string): boolean {
  return relPath.startsWith("git-commit:");
}

export function commitHtmlUrl(repo: { htmlUrl: string }, hash: string): string {
  return `${repo.htmlUrl}/commit/${hash}`;
}

/** Single-letter badge for a change kind (VS Code style). */
export function changeBadge(kind: GitChangeKind): string {
  switch (kind) {
    case "modified":
      return "M";
    case "added":
      return "A";
    case "deleted":
      return "D";
    case "renamed":
      return "R";
    case "copied":
      return "C";
    case "untracked":
      return "U";
    default:
      return "?";
  }
}

/** Virtual editor tab key for the Fetch Source form (singleton tab). */
const FETCH_SOURCE_TAB_KEY = "git-fetch-source";

/** Virtual editor tab key for the New Application Wizard (singleton tab). */
const NEW_APP_WIZARD_TAB_KEY = "new-app-wizard";

export function fetchSourceTabKey(): string {
  return FETCH_SOURCE_TAB_KEY;
}

export function isFetchSourceTab(relPath: string): boolean {
  return relPath === FETCH_SOURCE_TAB_KEY;
}

export function newAppWizardTabKey(): string {
  return NEW_APP_WIZARD_TAB_KEY;
}

export function isNewAppWizardTab(relPath: string): boolean {
  return relPath === NEW_APP_WIZARD_TAB_KEY;
}

/** Clones `url` at `branch` into the (empty) workspace root. */
export async function gitFetchSource(url: string, branch: string): Promise<WorkspaceInfo> {
  return kiwiInvoke<WorkspaceInfo>("git_fetch_source", { url, branch });
}

/** Reads the repository status (branch, ahead/behind, changed files). */
export async function gitStatus(): Promise<GitStatus> {
  return kiwiInvoke<GitStatus>("git_status");
}

/** Lists local branch names (for the Create Branch base selector). */
export async function gitBranchList(): Promise<string[]> {
  return kiwiInvoke<string[]>("git_branch_list");
}

/** Creates a new branch from `base` and checks it out. */
export async function gitCreateBranch(name: string, base: string): Promise<void> {
  return kiwiInvoke<void>("git_create_branch", { name, base });
}

/** Stages one repository-relative path. */
export async function gitStage(path: string): Promise<void> {
  return kiwiInvoke<void>("git_stage", { path });
}

/** Stages all changes. */
export async function gitStageAll(): Promise<void> {
  return kiwiInvoke<void>("git_stage_all");
}

/** Unstages one repository-relative path. */
export async function gitUnstage(path: string): Promise<void> {
  return kiwiInvoke<void>("git_unstage", { path });
}

/** Discards working-tree edits for one path. */
export async function gitDiscard(path: string): Promise<void> {
  return kiwiInvoke<void>("git_discard", { path });
}

/** Commits staged changes; optionally stages everything first. */
export async function gitCommit(message: string, stageAll: boolean): Promise<void> {
  return kiwiInvoke<void>("git_commit", { message, stageAll });
}

/** Pushes the current branch (sets upstream on first publish). */
export async function gitPush(): Promise<void> {
  return kiwiInvoke<void>("git_push");
}

/** Pulls from the current branch's upstream. */
export async function gitPull(): Promise<void> {
  return kiwiInvoke<void>("git_pull");
}

/** Reads up to `limit` recent commits for the graph section. */
export async function gitLog(limit = 100): Promise<GitCommit[]> {
  return kiwiInvoke<GitCommit[]>("git_log", { limit });
}

/** Loads per-file diffs for a commit. */
export async function gitCommitChanges(hash: string): Promise<GitCommitChanges> {
  return kiwiInvoke<GitCommitChanges>("git_commit_changes", { hash });
}
