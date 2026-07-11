import { kiwiInvoke } from "./ipc";

export type LogLine = {
  level: string;
  target: string;
  message: string;
  timestamp: string;
};

/** Returns buffered tracing log lines for the Logs panel. */
export async function logsSnapshot(): Promise<LogLine[]> {
  return kiwiInvoke<LogLine[]>("logs_snapshot");
}

/** Clears the in-memory log buffer. */
export async function logsClear(): Promise<void> {
  return kiwiInvoke<void>("logs_clear");
}
