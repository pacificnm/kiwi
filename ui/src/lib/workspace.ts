import { kiwiInvoke } from "./ipc";

/** Active workspace metadata (project root + display name). */
export type WorkspaceInfo = {
  root: string;
  name: string;
};

/** One entry in an Explorer directory listing. */
export type FsEntry = {
  name: string;
  relPath: string;
  isDir: boolean;
};

/** UTF-8 contents of a file opened in the editor. */
export type FileContent = {
  relPath: string;
  content: string;
};

export type WorkspaceSearchQuery = {
  query: string;
  includes?: string;
  excludes?: string;
  matchCase: boolean;
  wholeWord: boolean;
  useRegex: boolean;
  maxMatches?: number;
};

export type WorkspaceSearchMatch = {
  line: number;
  col: number;
  matchText: string;
  lineText: string;
};

export type WorkspaceSearchFile = {
  relPath: string;
  matches: WorkspaceSearchMatch[];
};

export type WorkspaceSearchResponse = {
  files: WorkspaceSearchFile[];
  matchCount: number;
  fileCount: number;
  truncated: boolean;
};

export type WorkspaceReplaceRequest = {
  search: WorkspaceSearchQuery;
  replace: string;
};

export type WorkspaceReplaceResponse = {
  fileCount: number;
  matchCount: number;
};

/** Returns the active workspace root + name. */
export async function workspaceInfo(): Promise<WorkspaceInfo> {
  return kiwiInvoke<WorkspaceInfo>("workspace_info");
}

/** Lists a directory (dirs first, ignored names hidden). `"."` is the root. */
export async function listDir(rel: string): Promise<FsEntry[]> {
  return kiwiInvoke<FsEntry[]>("workspace_list", { rel });
}

/** Reads a UTF-8 text file relative to the project root. */
export async function readTextFile(rel: string): Promise<FileContent> {
  return kiwiInvoke<FileContent>("workspace_read", { rel });
}

/** Writes editor `content` back to `rel` (Save). Returns the saved rel path. */
export async function writeTextFile(rel: string, content: string): Promise<string> {
  return kiwiInvoke<string>("workspace_write", { rel, content });
}

/** Switches the workspace to a new root directory. */
export async function openWorkspace(root: string): Promise<WorkspaceInfo> {
  return kiwiInvoke<WorkspaceInfo>("workspace_open", { root });
}

/** Creates an empty file at `rel`; returns the created relative path. */
export async function createFile(rel: string): Promise<string> {
  return kiwiInvoke<string>("workspace_create_file", { rel });
}

/** Creates a directory at `rel`; returns the created relative path. */
export async function createDir(rel: string): Promise<string> {
  return kiwiInvoke<string>("workspace_create_dir", { rel });
}

/** Renames / moves `from` → `to`; returns the new relative path. */
export async function renamePath(from: string, to: string): Promise<string> {
  return kiwiInvoke<string>("workspace_rename", { from, to });
}

/** Deletes a file or directory tree at `rel`; returns the removed path. */
export async function deletePath(rel: string): Promise<string> {
  return kiwiInvoke<string>("workspace_delete", { rel });
}

/** Copies a file or directory tree `from` → `to`; returns the new path. */
export async function copyPath(from: string, to: string): Promise<string> {
  return kiwiInvoke<string>("workspace_copy", { from, to });
}

/** Reveals a path in the OS file manager (Open Containing Folder). */
export async function revealPath(rel: string): Promise<void> {
  return kiwiInvoke<void>("workspace_reveal", { rel });
}

/** Searches the workspace (VS Code-style search sidebar). */
export async function workspaceSearch(query: WorkspaceSearchQuery): Promise<WorkspaceSearchResponse> {
  return kiwiInvoke<WorkspaceSearchResponse>("workspace_search", { query });
}

/** Replaces all matches for a query across the workspace ("Replace All"). */
export async function workspaceReplaceAll(
  request: WorkspaceReplaceRequest,
): Promise<WorkspaceReplaceResponse> {
  return kiwiInvoke<WorkspaceReplaceResponse>("workspace_replace_all", { request });
}

/** Joins a directory and a child name into a normalized relative path. */
export function joinRel(dir: string, name: string): string {
  const clean = name.trim().replace(/^\/+|\/+$/g, "");
  if (dir === "." || dir === "") {
    return clean;
  }
  return `${dir}/${clean}`;
}

/** Parent directory of `rel`, or `"."` for a top-level entry. */
export function parentRel(rel: string): string {
  if (rel === "." || !rel.includes("/")) {
    return ".";
  }
  return rel.slice(0, rel.lastIndexOf("/"));
}

/** Final path segment (file or folder name) of `rel`. */
export function baseName(rel: string): string {
  if (rel === "." || rel === "") {
    return rel;
  }
  const idx = rel.lastIndexOf("/");
  return idx === -1 ? rel : rel.slice(idx + 1);
}
