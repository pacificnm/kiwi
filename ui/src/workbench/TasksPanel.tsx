import { useCallback, useEffect, useMemo, useState } from "react";
import { formatIpcError } from "../lib/agent";
import { faChevronLeft, faDiamond, faLink, faRotateRight, faUnlink } from "../lib/fontawesome";
import {
  swiftLinkWorkspace,
  swiftTasksOverview,
  swiftUnlinkWorkspace,
  type SwiftProjectSummary,
  type SwiftTaskSummary,
  type SwiftTasksOverview,
} from "../lib/swift";
import { Icon, isTauri, useToast } from "../shell";
import { useWorkbench } from "./state";

type TasksPanelProps = {
  onToggleCollapse?: () => void;
};

export function TasksPanel({ onToggleCollapse }: TasksPanelProps) {
  const toast = useToast();
  const { workspace, refreshToken, openTask } = useWorkbench();
  const [overview, setOverview] = useState<SwiftTasksOverview | null>(null);
  const [busy, setBusy] = useState(false);
  const [selectedProjectId, setSelectedProjectId] = useState("");

  const refresh = useCallback(async () => {
    if (!isTauri()) {
      return;
    }
    setBusy(true);
    try {
      const next = await swiftTasksOverview();
      setOverview(next);
      if (next.link?.projectId) {
        setSelectedProjectId(next.link.projectId);
      }
    } catch (error) {
      toast.error(formatIpcError(error));
    } finally {
      setBusy(false);
    }
  }, [toast]);

  useEffect(() => {
    if (!isTauri()) {
      return;
    }
    void refresh();
  }, [refresh, workspace?.root, refreshToken]);

  const activeProjects = useMemo(
    () => (overview?.projects ?? []).filter((project) => !project.archived),
    [overview],
  );

  const visibleTasks = useMemo(() => {
    const tasks = overview?.tasks ?? [];
    return tasks.filter((task) => !task.isSummary);
  }, [overview]);

  const handleLink = useCallback(async () => {
    if (!selectedProjectId) {
      toast.info("Choose a Swift project first");
      return;
    }
    setBusy(true);
    try {
      await swiftLinkWorkspace(selectedProjectId);
      await refresh();
      toast.success("Linked workspace to Swift project");
    } catch (error) {
      toast.error(formatIpcError(error));
    } finally {
      setBusy(false);
    }
  }, [refresh, selectedProjectId, toast]);

  const handleUnlink = useCallback(async () => {
    setBusy(true);
    try {
      await swiftUnlinkWorkspace();
      await refresh();
      toast.success("Unlinked Swift project");
    } catch (error) {
      toast.error(formatIpcError(error));
    } finally {
      setBusy(false);
    }
  }, [refresh, toast]);

  if (!isTauri()) {
    return (
      <p className="px-3 py-2 text-xs text-nest-muted">Tasks require the desktop app.</p>
    );
  }

  const linked = overview?.link;
  const status = overview?.status;

  return (
    <div className="flex h-full min-h-0 flex-col bg-nest-background">
      <header className="flex h-9 shrink-0 items-center gap-2 border-b border-nest-border px-3">
        <span className="truncate text-xs font-semibold uppercase tracking-wide text-nest-muted">
          Tasks
        </span>
        <div className="ml-auto flex items-center gap-0.5">
          <button
            type="button"
            onClick={() => void refresh()}
            disabled={busy}
            title="Refresh tasks"
            aria-label="Refresh tasks"
            className="flex size-6 items-center justify-center rounded-nest-sm text-nest-muted hover:bg-nest-muted/10 hover:text-nest-foreground disabled:opacity-40"
          >
            <Icon icon={faRotateRight} className={["size-3", busy ? "animate-spin" : ""].join(" ")} />
          </button>
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

      <div className="min-h-0 flex-1 overflow-auto p-3">
        {status && !status.connected ? (
          <div className="mb-3 rounded-nest-sm border border-nest-error/30 bg-nest-error/5 px-3 py-2 text-xs text-nest-error">
            Swift database unavailable: {status.error ?? "not configured"}
          </div>
        ) : null}

        <section className="mb-4 rounded-nest-sm border border-nest-border bg-nest-surface p-3">
          <div className="mb-2 text-xs font-semibold uppercase tracking-wide text-nest-muted">
            Swift project
          </div>
          {linked ? (
            <div className="flex items-start justify-between gap-2">
              <div className="min-w-0">
                <p className="truncate text-sm font-medium text-nest-foreground">
                  {linked.projectName ?? overview?.project?.name ?? "Linked project"}
                </p>
                <p className="truncate text-[11px] text-nest-muted">{workspace?.name ?? "Workspace"}</p>
              </div>
              <button
                type="button"
                onClick={() => void handleUnlink()}
                disabled={busy}
                title="Unlink Swift project"
                className="inline-flex shrink-0 items-center gap-1 rounded-nest-sm px-2 py-1 text-[11px] text-nest-muted hover:bg-nest-muted/10 hover:text-nest-foreground"
              >
                <Icon icon={faUnlink} className="size-3" />
                Unlink
              </button>
            </div>
          ) : (
            <div className="flex flex-col gap-2">
              <p className="text-xs text-nest-muted">
                Link this repo to a Swift project to load its tasks.
              </p>
              <ProjectPicker
                projects={activeProjects}
                value={selectedProjectId}
                onChange={setSelectedProjectId}
              />
              <button
                type="button"
                onClick={() => void handleLink()}
                disabled={busy || !selectedProjectId}
                className="inline-flex h-7 items-center justify-center gap-1 rounded-nest-sm bg-nest-accent px-3 text-xs font-semibold text-nest-background hover:brightness-110 disabled:opacity-50"
              >
                <Icon icon={faLink} className="size-3" />
                Link project
              </button>
            </div>
          )}
        </section>

        {linked ? (
          <section>
            <div className="mb-2 flex items-center justify-between gap-2">
              <span className="text-xs font-semibold uppercase tracking-wide text-nest-muted">
                Tasks
              </span>
              <span className="text-[11px] text-nest-muted">
                {visibleTasks.length} shown
                {overview?.project?.percentComplete != null
                  ? ` · ${overview.project.percentComplete}% complete`
                  : ""}
              </span>
            </div>
            {visibleTasks.length === 0 ? (
              <p className="text-xs text-nest-muted">No tasks in this Swift project.</p>
            ) : (
              <div className="flex flex-col gap-1">
                {visibleTasks.map((task) => (
                  <TaskRow key={task.id} task={task} onOpen={() => openTask(task.id, task.title)} />
                ))}
              </div>
            )}
          </section>
        ) : null}
      </div>
    </div>
  );
}

function ProjectPicker({
  projects,
  value,
  onChange,
}: {
  projects: SwiftProjectSummary[];
  value: string;
  onChange: (value: string) => void;
}) {
  return (
    <select
      value={value}
      onChange={(event) => onChange(event.target.value)}
      className="h-8 w-full rounded-nest-sm border border-nest-border bg-nest-background px-2 text-sm text-nest-foreground focus:outline-none focus:ring-2 focus:ring-nest-accent/50"
    >
      <option value="">Select Swift project…</option>
      {projects.map((project) => (
        <option key={project.id} value={project.id}>
          {project.name}
        </option>
      ))}
    </select>
  );
}

function TaskRow({ task, onOpen }: { task: SwiftTaskSummary; onOpen: () => void }) {
  const indent = Math.max(0, task.outlineLevel - 1) * 12;
  const done = task.percentComplete >= 100;

  return (
    <button
      type="button"
      onClick={onOpen}
      className="w-full rounded-nest-sm border border-nest-border bg-nest-surface px-2 py-1.5 text-left hover:bg-nest-muted/10"
      style={{ marginLeft: indent }}
    >
      <div className="flex items-start gap-2">
        {task.isMilestone ? (
          <Icon icon={faDiamond} className="mt-0.5 size-3 shrink-0 text-nest-accent" />
        ) : (
          <span
            className={[
              "mt-1 size-2.5 shrink-0 rounded-full border",
              done ? "border-nest-accent bg-nest-accent" : "border-nest-muted",
            ].join(" ")}
          />
        )}
        <div className="min-w-0 flex-1">
          <p
            className={[
              "truncate text-sm",
              done ? "text-nest-muted line-through" : "text-nest-foreground",
            ].join(" ")}
          >
            {task.title}
          </p>
          <p className="truncate text-[11px] text-nest-muted">
            {task.percentComplete}%
            {task.startDate ? ` · ${task.startDate}` : ""}
            {task.finishDate ? ` → ${task.finishDate}` : ""}
            {task.priority ? ` · ${task.priority}` : ""}
          </p>
        </div>
      </div>
    </button>
  );
}
