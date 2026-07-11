import { Icon } from "../../shell";
import { faDiamond } from "../../lib/fontawesome";
import type { SwiftTaskDetailResponse, SwiftTaskSummary } from "../../lib/swift";
import { useWorkbench } from "../state";

type TaskDetailViewProps = {
  detail: SwiftTaskDetailResponse;
};

export function TaskDetailView({ detail }: TaskDetailViewProps) {
  const { openTask } = useWorkbench();
  const { task, project, subtasks } = detail;
  const done = task.percentComplete >= 100;

  return (
    <div className="h-full min-h-0 overflow-auto bg-nest-background">
      <div className="mx-auto max-w-4xl px-6 py-6">
        <header className="border-b border-nest-border pb-4">
          <p className="text-xs font-semibold uppercase tracking-wide text-nest-muted">
            {project.name}
          </p>
          <h1 className="mt-1 flex items-start gap-2 text-2xl font-semibold leading-snug text-nest-foreground">
            {task.isMilestone ? (
              <Icon icon={faDiamond} className="mt-1.5 size-4 shrink-0 text-nest-accent" />
            ) : null}
            <span className={done ? "text-nest-muted line-through" : undefined}>{task.title}</span>
          </h1>

          <div className="mt-3 flex flex-wrap items-center gap-2 text-sm">
            <ProgressBadge percent={task.percentComplete} />
            {task.priority ? <MetaChip label={task.priority} /> : null}
            {task.isMilestone ? <MetaChip label="Milestone" /> : null}
            {task.isSummary ? <MetaChip label="Summary" /> : null}
            {task.taskType ? <MetaChip label={task.taskType} /> : null}
          </div>

          <dl className="mt-4 grid gap-2 text-sm sm:grid-cols-2">
            <DetailField label="Start" value={task.startDate} />
            <DetailField label="Finish" value={task.finishDate} />
            <DetailField label="Duration" value={formatDuration(task.durationDays, task.durationMinutes)} />
            <DetailField label="Deadline" value={task.deadline} />
            <DetailField label="Constraint" value={formatConstraint(task.constraintType, task.constraintDate)} />
            <DetailField label="Resources" value={task.resourceNames.trim() || undefined} />
            <DetailField label="Actual start" value={task.actualStart} />
            <DetailField label="Actual finish" value={task.actualFinish} />
          </dl>

          <p className="mt-3 text-xs text-nest-muted">
            Updated {formatTimestamp(task.updatedAt)} · Created {formatTimestamp(task.createdAt)}
          </p>
        </header>

        {task.notes?.trim() ? (
          <section className="border-b border-nest-border py-6">
            <h2 className="mb-3 text-sm font-semibold uppercase tracking-wide text-nest-muted">Notes</h2>
            <pre className="whitespace-pre-wrap font-sans text-sm leading-relaxed text-nest-foreground">
              {task.notes.trim()}
            </pre>
          </section>
        ) : null}

        {subtasks.length > 0 ? (
          <section className="py-6">
            <h2 className="mb-3 text-sm font-semibold uppercase tracking-wide text-nest-muted">
              Subtasks ({subtasks.length})
            </h2>
            <div className="flex flex-col gap-1">
              {subtasks.map((subtask) => (
                <SubtaskRow
                  key={subtask.id}
                  task={subtask}
                  onOpen={() => openTask(subtask.id, subtask.title)}
                />
              ))}
            </div>
          </section>
        ) : null}
      </div>
    </div>
  );
}

function DetailField({ label, value }: { label: string; value?: string | null }) {
  if (!value) {
    return null;
  }
  return (
    <div>
      <dt className="text-xs uppercase tracking-wide text-nest-muted">{label}</dt>
      <dd className="text-nest-foreground">{value}</dd>
    </div>
  );
}

function ProgressBadge({ percent }: { percent: number }) {
  const done = percent >= 100;
  return (
    <span
      className={[
        "inline-flex rounded-full px-2.5 py-0.5 text-xs font-medium",
        done ? "bg-nest-accent/15 text-nest-accent" : "bg-nest-muted/10 text-nest-foreground",
      ].join(" ")}
    >
      {percent}% complete
    </span>
  );
}

function MetaChip({ label }: { label: string }) {
  return (
    <span className="inline-flex rounded-full border border-nest-border px-2.5 py-0.5 text-xs text-nest-muted">
      {label}
    </span>
  );
}

function SubtaskRow({ task, onOpen }: { task: SwiftTaskSummary; onOpen: () => void }) {
  const done = task.percentComplete >= 100;
  return (
    <button
      type="button"
      onClick={onOpen}
      className="flex w-full items-center gap-2 rounded-nest-sm border border-nest-border bg-nest-surface px-3 py-2 text-left hover:bg-nest-muted/10"
    >
      <span
        className={[
          "size-2.5 shrink-0 rounded-full border",
          done ? "border-nest-accent bg-nest-accent" : "border-nest-muted",
        ].join(" ")}
      />
      <span className={["min-w-0 flex-1 truncate text-sm", done ? "text-nest-muted line-through" : "text-nest-foreground"].join(" ")}>
        {task.title}
      </span>
      <span className="shrink-0 text-xs text-nest-muted">{task.percentComplete}%</span>
    </button>
  );
}

function formatDuration(days: number, minutes?: number): string | undefined {
  if (days <= 0 && !minutes) {
    return undefined;
  }
  const parts: string[] = [];
  if (days > 0) {
    parts.push(`${days} day${days === 1 ? "" : "s"}`);
  }
  if (minutes && minutes > 0) {
    parts.push(`${minutes} min`);
  }
  return parts.join(" · ");
}

function formatConstraint(type?: string, date?: string): string | undefined {
  if (!type) {
    return undefined;
  }
  return date ? `${type} (${date})` : type;
}

function formatTimestamp(value: string): string {
  const date = new Date(value);
  if (Number.isNaN(date.getTime())) {
    return value;
  }
  return date.toLocaleString(undefined, {
    year: "numeric",
    month: "short",
    day: "numeric",
    hour: "2-digit",
    minute: "2-digit",
  });
}
