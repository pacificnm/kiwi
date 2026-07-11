/**
 * Lightweight console tracing for the Kiwi workbench.
 *
 * Every line is prefixed `[kiwi +Nms]` (ms since page load) so the last line
 * printed before a freeze pinpoints where execution stopped.
 */
function stamp(): string {
  return `+${Math.round(performance.now())}ms`;
}

export function klog(scope: string, message: string, ...args: unknown[]): void {
  console.debug(`[kiwi ${stamp()}] ${scope}: ${message}`, ...args);
}

export function kerr(scope: string, message: string, ...args: unknown[]): void {
  console.error(`[kiwi ${stamp()}] ${scope}: ${message}`, ...args);
}

/** Logs uncaught errors and unhandled promise rejections to the console. */
export function installGlobalErrorHandlers(): void {
  window.addEventListener("error", (event) => {
    kerr("global", "uncaught error", event.error ?? event.message);
  });
  window.addEventListener("unhandledrejection", (event) => {
    kerr("global", "unhandled rejection", event.reason);
  });
}
