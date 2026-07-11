import { useCallback, useEffect, useMemo, useState, type MouseEvent as ReactMouseEvent } from "react";
import { ContextMenu, type ContextMenuItem } from "../components/ContextMenu";
import { formatIpcError } from "../lib/agent";
import { faCircleExclamation, faRotateRight, faTriangleExclamation } from "../lib/fontawesome";
import {
  problemsRun,
  problemsSnapshot,
  type ProblemDiagnostic,
  type ProblemsReport,
} from "../lib/problems";
import { baseName } from "../lib/workspace";
import { Icon, isTauri, useToast } from "../shell";
import { useEditorNavigation } from "./editorNavigation";
import { useWorkbench } from "./state";

type ProblemsPanelProps = {
  active: boolean;
};

type FileGroup = {
  relPath: string;
  items: ProblemDiagnostic[];
};

export function ProblemsPanel({ active }: ProblemsPanelProps) {
  const toast = useToast();
  const { openFile } = useWorkbench();
  const { openAt } = useEditorNavigation();
  const [report, setReport] = useState<ProblemsReport | null>(null);
  const [busy, setBusy] = useState(false);
  const [menu, setMenu] = useState<{ x: number; y: number; item: ProblemDiagnostic } | null>(null);

  const refresh = useCallback(async () => {
    if (!isTauri()) {
      return;
    }
    try {
      const next = await problemsSnapshot();
      setReport(next);
    } catch {
      // Ignore transient poll errors.
    }
  }, []);

  const run = useCallback(async () => {
    if (!isTauri()) {
      toast.info("Problems require the desktop app");
      return;
    }
    setBusy(true);
    try {
      const next = await problemsRun();
      setReport(next);
    } catch (error) {
      toast.error(formatIpcError(error));
    } finally {
      setBusy(false);
    }
  }, [toast]);

  useEffect(() => {
    if (!active || !isTauri()) {
      return;
    }
    void refresh();
    const timer = window.setInterval(() => void refresh(), 1_000);
    return () => window.clearInterval(timer);
  }, [active, refresh]);

  const groups = useMemo(() => groupByFile(report?.diagnostics ?? []), [report]);

  const openDiagnostic = useCallback(
    (item: ProblemDiagnostic) => {
      openFile(item.relPath, baseName(item.relPath));
      openAt(item.relPath, item.line, item.col);
    },
    [openAt, openFile],
  );

  const copyProblem = useCallback(
    (item: ProblemDiagnostic) => {
      void navigator.clipboard
        .writeText(formatProblemForCopy(item))
        .then(() => toast.success("Copied problem"))
        .catch((error) => toast.error(formatIpcError(error)));
    },
    [toast],
  );

  const onDiagnosticContextMenu = useCallback(
    (event: ReactMouseEvent, item: ProblemDiagnostic) => {
      event.preventDefault();
      setMenu({ x: event.clientX, y: event.clientY, item });
    },
    [],
  );

  const menuItems: ContextMenuItem[] = menu
    ? [
        {
          id: "copy",
          label: "Copy",
          onSelect: () => copyProblem(menu.item),
        },
      ]
    : [];

  if (!isTauri()) {
    return (
      <p className="px-3 py-2 text-xs text-nest-muted">
        Problems are available in the desktop app.
      </p>
    );
  }

  const running = busy || report?.running;

  return (
    <div className="flex h-full min-h-0 flex-col bg-nest-background">
      <div className="flex h-7 shrink-0 items-center gap-3 border-b border-nest-border px-3 text-[11px]">
        <span className="text-nest-error">{report?.errorCount ?? 0} errors</span>
        <span className="text-nest-warning">{report?.warningCount ?? 0} warnings</span>
        <span className="truncate text-nest-muted">{report?.summary ?? "Checking…"}</span>
        <button
          type="button"
          onClick={() => void run()}
          disabled={running}
          title="Re-run diagnostics"
          aria-label="Re-run diagnostics"
          className="ml-auto inline-flex items-center gap-1 rounded-nest-sm px-2 py-0.5 text-nest-muted hover:bg-nest-muted/10 hover:text-nest-foreground disabled:opacity-40"
        >
          <Icon icon={faRotateRight} className={["size-3", running ? "animate-spin" : ""].join(" ")} />
          Refresh
        </button>
      </div>

      <div className="min-h-0 flex-1 overflow-auto">
        {groups.length === 0 ? (
          <p className="px-3 py-4 text-xs text-nest-muted">
            {running ? "Running cargo, clippy, tsc, and eslint…" : "No problems detected."}
          </p>
        ) : (
          groups.map((group) => (
            <div key={group.relPath} className="border-b border-nest-border/60">
              <div className="sticky top-0 z-[1] bg-nest-surface px-3 py-1 text-xs font-medium text-nest-foreground">
                {group.relPath}
                <span className="ml-2 text-nest-muted">{group.items.length}</span>
              </div>
              {group.items.map((item) => (
                <button
                  key={item.id}
                  type="button"
                  onClick={() => openDiagnostic(item)}
                  onContextMenu={(event) => onDiagnosticContextMenu(event, item)}
                  title={item.message}
                  className="flex w-full items-start gap-2 px-3 py-1 text-left text-xs hover:bg-nest-muted/10"
                >
                  <SeverityIcon severity={item.severity} />
                  <span className="min-w-0 flex-1 truncate text-nest-foreground">{item.message}</span>
                  <span className="shrink-0 tabular-nums text-nest-muted">
                    {item.line}:{item.col}
                  </span>
                  <span className="shrink-0 text-[10px] uppercase tracking-wide text-nest-muted">
                    {item.source}
                    {item.code ? ` · ${item.code}` : ""}
                  </span>
                </button>
              ))}
            </div>
          ))
        )}
      </div>

      {menu ? (
        <ContextMenu x={menu.x} y={menu.y} items={menuItems} onClose={() => setMenu(null)} />
      ) : null}
    </div>
  );
}

/** Formats a diagnostic as `path:line:col - severity[code]: message`, ready to paste into an agent. */
function formatProblemForCopy(item: ProblemDiagnostic): string {
  const location = `${item.relPath}:${item.line}:${item.col}`;
  const tag = item.code ? `${item.severity}[${item.code}]` : item.severity;
  return `${location} - ${tag}: ${item.message}`;
}

function SeverityIcon({ severity }: { severity: ProblemDiagnostic["severity"] }) {
  if (severity === "error") {
    return <Icon icon={faCircleExclamation} className="mt-0.5 size-3 shrink-0 text-nest-error" />;
  }
  if (severity === "warning") {
    return (
      <Icon icon={faTriangleExclamation} className="mt-0.5 size-3 shrink-0 text-nest-warning" />
    );
  }
  return <span className="mt-1 size-3 shrink-0 rounded-full bg-nest-muted/50" />;
}

function groupByFile(items: ProblemDiagnostic[]): FileGroup[] {
  const map = new Map<string, ProblemDiagnostic[]>();
  for (const item of items) {
    const bucket = map.get(item.relPath) ?? [];
    bucket.push(item);
    map.set(item.relPath, bucket);
  }
  return [...map.entries()]
    .sort(([a], [b]) => a.localeCompare(b))
    .map(([relPath, diagnostics]) => ({ relPath, items: diagnostics }));
}
