import { kiwiInvoke } from "./ipc";

export type DocEntry = {
  path: string;
  name: string;
  depth: number;
};

export async function docsList(projectId: string): Promise<DocEntry[]> {
  return kiwiInvoke<DocEntry[]>("docs_list", { projectId });
}

export async function docsRead(projectId: string, path: string): Promise<string> {
  return kiwiInvoke<string>("docs_read", { projectId, path });
}

export function docTabKey(projectId: string, path: string): string {
  return `help-doc:${projectId}:${path}`;
}

export function isDocTab(relPath: string): boolean {
  return relPath.startsWith("help-doc:");
}
