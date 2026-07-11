import { kiwiInvoke } from "./ipc";

export type DocProject = {
  id: string;
  name: string;
  repoUrl: string;
  docsPath: string;
  branch: string | null;
  synced: boolean;
  lastSyncedAt: number | null;
};

export type DocProjectInput = {
  name: string;
  repoUrl: string;
  docsPath: string;
  branch: string | null;
};

export async function docSourcesList(): Promise<DocProject[]> {
  return kiwiInvoke<DocProject[]>("doc_sources_list");
}

export async function docSourceAdd(input: DocProjectInput): Promise<DocProject> {
  return kiwiInvoke<DocProject>("doc_sources_add", { input });
}

export async function docSourceRemove(id: string): Promise<void> {
  return kiwiInvoke<void>("doc_sources_remove", { id });
}

export async function docSourceSync(id: string): Promise<DocProject> {
  return kiwiInvoke<DocProject>("doc_sources_sync", { id });
}
