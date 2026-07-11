import { kiwiInvoke } from "./ipc";

export type ProblemSeverity = "error" | "warning" | "info";

export type ProblemDiagnostic = {
  id: string;
  source: string;
  severity: ProblemSeverity;
  message: string;
  relPath: string;
  line: number;
  col: number;
  endLine?: number;
  endCol?: number;
  code?: string;
};

export type ProblemsReport = {
  diagnostics: ProblemDiagnostic[];
  errorCount: number;
  warningCount: number;
  infoCount: number;
  running: boolean;
  ranAt?: string;
  summary: string;
};

const RUN_EVENT = "kiwi:problems-run";
let runTimer: number | null = null;

/** Returns the latest diagnostics snapshot. */
export async function problemsSnapshot(): Promise<ProblemsReport> {
  return kiwiInvoke<ProblemsReport>("problems_snapshot");
}

/** Runs workspace diagnostics (cargo, clippy, tsc, eslint). */
export async function problemsRun(): Promise<ProblemsReport> {
  return kiwiInvoke<ProblemsReport>("problems_run");
}

/** Debounced request to re-run diagnostics (e.g. after save). */
export function scheduleProblemsRun(delayMs = 1_500) {
  if (runTimer !== null) {
    window.clearTimeout(runTimer);
  }
  runTimer = window.setTimeout(() => {
    runTimer = null;
    window.dispatchEvent(new Event(RUN_EVENT));
  }, delayMs);
}

/** Subscribe to debounced diagnostics refresh requests. */
export function onProblemsRunRequested(listener: () => void): () => void {
  const handler = () => listener();
  window.addEventListener(RUN_EVENT, handler);
  return () => window.removeEventListener(RUN_EVENT, handler);
}
