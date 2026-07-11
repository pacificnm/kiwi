import { kiwiInvoke } from "./ipc";

export type SwiftStatus = {
  connected: boolean;
  database: string;
  error?: string;
};

export type SwiftProjectSummary = {
  id: string;
  slug: string;
  name: string;
  archived: boolean;
  percentComplete: number;
};

export type SwiftTaskSummary = {
  id: string;
  projectId: string;
  parentId?: string;
  outlineLevel: number;
  isSummary: boolean;
  isMilestone: boolean;
  title: string;
  percentComplete: number;
  startDate?: string;
  finishDate?: string;
  priority?: string;
  sortOrder: number;
};

export type SwiftTaskDetail = {
  id: string;
  projectId: string;
  parentId?: string;
  outlineLevel: number;
  isSummary: boolean;
  isMilestone: boolean;
  title: string;
  notes?: string;
  durationDays: number;
  durationMinutes?: number;
  startDate?: string;
  finishDate?: string;
  percentComplete: number;
  resourceNames: string;
  priority?: string;
  constraintType?: string;
  constraintDate?: string;
  deadline?: string;
  effortDriven: boolean;
  taskType?: string;
  sortOrder: number;
  actualStart?: string;
  actualFinish?: string;
  createdAt: string;
  updatedAt: string;
};

export type SwiftTaskDetailResponse = {
  task: SwiftTaskDetail;
  project: SwiftProjectSummary;
  subtasks: SwiftTaskSummary[];
};

export type SwiftWorkspaceLinkSummary = {
  workspaceRoot: string;
  projectId: string;
  projectName?: string;
};

export type SwiftTasksOverview = {
  status: SwiftStatus;
  workspaceRoot: string;
  link?: SwiftWorkspaceLinkSummary;
  project?: SwiftProjectSummary;
  tasks: SwiftTaskSummary[];
  projects: SwiftProjectSummary[];
};

export async function swiftStatus(): Promise<SwiftStatus> {
  return kiwiInvoke<SwiftStatus>("swift_status");
}

export async function swiftTasksOverview(): Promise<SwiftTasksOverview> {
  return kiwiInvoke<SwiftTasksOverview>("swift_tasks_overview");
}

export async function swiftListProjects(): Promise<SwiftProjectSummary[]> {
  return kiwiInvoke<SwiftProjectSummary[]>("swift_list_projects");
}

export async function swiftLinkWorkspace(projectId: string): Promise<SwiftWorkspaceLinkSummary> {
  return kiwiInvoke<SwiftWorkspaceLinkSummary>("swift_link_workspace", { projectId });
}

export async function swiftUnlinkWorkspace(): Promise<void> {
  return kiwiInvoke<void>("swift_unlink_workspace");
}

export async function swiftGetTask(taskId: string): Promise<SwiftTaskDetailResponse> {
  return kiwiInvoke<SwiftTaskDetailResponse>("swift_get_task", { taskId });
}

/** Virtual editor tab key for a Swift task. */
export function taskTabKey(taskId: string): string {
  return `swift-task:${taskId}`;
}

export function isTaskTab(relPath: string): boolean {
  return relPath.startsWith("swift-task:");
}

export function taskIdFromTab(relPath: string): string | null {
  if (!isTaskTab(relPath)) {
    return null;
  }
  const id = relPath.slice("swift-task:".length).trim();
  return id.length > 0 ? id : null;
}
