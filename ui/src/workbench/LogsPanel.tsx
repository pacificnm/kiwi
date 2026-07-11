import { useCallback, useEffect, useRef, useState } from "react";
import { formatIpcError } from "../lib/agent";
import { faTrash } from "../lib/fontawesome";
import { logsClear, logsSnapshot, type LogLine } from "../lib/logs";
import { Icon, isTauri, useToast } from "../shell";

type LogsPanelProps = {
  /** When false, polling pauses (tab not visible). */
  active: boolean;
};

const LEVEL_CLASS: Record<string, string> = {
  error: "text-nest-error",
  warn: "text-nest-warning",
  info: "text-nest-foreground",
  debug: "text-nest-muted",
  trace: "text-nest-muted/70",
};

export function LogsPanel({ active }: LogsPanelProps) {
  const toast = useToast();
  const [lines, setLines] = useState<LogLine[]>([]);
  const [busy, setBusy] = useState(false);
  const scrollRef = useRef<HTMLDivElement>(null);
  const stickToBottomRef = useRef(true);

  const refresh = useCallback(async () => {
    if (!isTauri()) {
      return;
    }
    try {
      const next = await logsSnapshot();
      setLines(next);
    } catch {
      // Ignore transient poll errors; next tick will retry.
    }
  }, []);

  useEffect(() => {
    if (!active || !isTauri()) {
      return;
    }
    void refresh();
    const timer = window.setInterval(() => void refresh(), 750);
    return () => window.clearInterval(timer);
  }, [active, refresh]);

  useEffect(() => {
    if (!stickToBottomRef.current || !scrollRef.current) {
      return;
    }
    scrollRef.current.scrollTop = scrollRef.current.scrollHeight;
  }, [lines]);

  const handleScroll = useCallback(() => {
    const node = scrollRef.current;
    if (!node) {
      return;
    }
    const distance = node.scrollHeight - node.scrollTop - node.clientHeight;
    stickToBottomRef.current = distance < 24;
  }, []);

  const handleClear = useCallback(async () => {
    if (!isTauri()) {
      return;
    }
    setBusy(true);
    try {
      await logsClear();
      setLines([]);
      stickToBottomRef.current = true;
    } catch (caught) {
      toast.error(formatIpcError(caught));
    } finally {
      setBusy(false);
    }
  }, [toast]);

  if (!isTauri()) {
    return (
      <p className="px-3 py-2 text-xs text-nest-muted">Logs are available in the desktop app.</p>
    );
  }

  return (
    <div className="flex h-full min-h-0 flex-col bg-nest-background">
      <div className="flex h-7 shrink-0 items-center gap-2 border-b border-nest-border px-3">
        <span className="text-[10px] uppercase tracking-wide text-nest-muted">
          {lines.length} line{lines.length === 1 ? "" : "s"}
        </span>
        <button
          type="button"
          onClick={() => void handleClear()}
          disabled={busy || lines.length === 0}
          title="Clear logs"
          aria-label="Clear logs"
          className="ml-auto inline-flex items-center gap-1 rounded-nest-sm px-2 py-0.5 text-[11px] text-nest-muted hover:bg-nest-muted/10 hover:text-nest-foreground disabled:opacity-40"
        >
          <Icon icon={faTrash} className="size-3" />
          Clear
        </button>
      </div>

      <div
        ref={scrollRef}
        onScroll={handleScroll}
        className="min-h-0 flex-1 overflow-auto font-mono text-[11px] leading-5"
      >
        {lines.length === 0 ? (
          <p className="px-3 py-4 text-nest-muted">No log output yet.</p>
        ) : (
          lines.map((line, index) => (
            <div
              key={`${line.timestamp}-${index}-${line.message}`}
              className="flex gap-2 px-3 py-px hover:bg-nest-muted/5"
            >
              <span className="shrink-0 text-nest-muted">{line.timestamp}</span>
              <span
                className={[
                  "w-10 shrink-0 uppercase",
                  LEVEL_CLASS[line.level] ?? "text-nest-foreground",
                ].join(" ")}
              >
                {line.level}
              </span>
              <span className="shrink-0 text-nest-accent/80">{shortTarget(line.target)}</span>
              <span className="min-w-0 whitespace-pre-wrap break-words text-nest-foreground/90">
                {line.message}
              </span>
            </div>
          ))
        )}
      </div>
    </div>
  );
}

function shortTarget(target: string): string {
  if (target === "kiwi") {
    return "kiwi";
  }
  const parts = target.split("::");
  return parts[parts.length - 1] || target;
}
