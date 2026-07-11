import { kiwiInvoke } from "./ipc";

export type DocEntry = {
  path: string;
  name: string;
  depth: number;
};

export async function docsList(): Promise<DocEntry[]> {
  return kiwiInvoke<DocEntry[]>("docs_list");
}

export async function docsRead(path: string): Promise<string> {
  return kiwiInvoke<string>("docs_read", { path });
}

export function docTabKey(path: string): string {
  return `help-doc:${path}`;
}

export function isDocTab(relPath: string): boolean {
  return relPath.startsWith("help-doc:");
}
