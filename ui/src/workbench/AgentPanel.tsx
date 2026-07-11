import { useCallback, useEffect, useRef, useState } from "react";
import { ClipboardAddon } from "@xterm/addon-clipboard";
import { Terminal } from "@xterm/xterm";
import { FitAddon } from "@xterm/addon-fit";
import "@xterm/xterm/css/xterm.css";
import type { UnlistenFn } from "@tauri-apps/api/event";
import { Icon, useStatusBar, useToast } from "../shell";
import { loadTerminalClipboard } from "../lib/terminalClipboard";
import { klog } from "../lib/log";
import { cssVar, safeFit, xtermTheme } from "../lib/xterm";
import { faPlay, faXmark } from "../lib/fontawesome";
import {
  formatIpcError,
  launchAgent,
  onAgentExit,
  onAgentOutput,
  resizeAgent,
  sendAgentInput,
  stopAgent,
  type AgentRuntime,
} from "../lib/agent";
import { useAgentSettings } from "./agentSettings";
import { useWorkbench } from "./state";

export function AgentPanel() {
  const { settings, ollamaHost } = useAgentSettings();
  const runtime = settings.runtime as AgentRuntime;
  const model = settings.model;
  const direct = settings.connection === "account";
  const [running, setRunning] = useState(false);
  const toast = useToast();
  const { setStatus } = useStatusBar();
  const { workspace } = useWorkbench();

  const cwd = workspace?.root ?? "";

  const hostRef = useRef<HTMLDivElement>(null);
  const termRef = useRef<Terminal | null>(null);
  const fitRef = useRef<FitAddon | null>(null);
  const unlistenRef = useRef<UnlistenFn[]>([]);
  const clipboardRef = useRef<ClipboardAddon | null>(null);

  useEffect(() => {
    setStatus("Configure model + account in the Agent sidebar, then press Launch", {
      variant: "info",
    });
  }, [setStatus]);

  useEffect(() => {
    if (!hostRef.current) {
      return;
    }
    const term = new Terminal({
      fontFamily: cssVar("--nest-font-mono", "JetBrains Mono, Consolas, monospace"),
      fontSize: 12,
      cursorBlink: true,
      convertEol: false,
      theme: xtermTheme(),
    });
    const fit = new FitAddon();
    term.loadAddon(fit);
    const host = hostRef.current;
    term.open(host);

    term.onData((data) => {
      void sendAgentInput(data).catch(() => {});
    });
    clipboardRef.current = loadTerminalClipboard(term, (text) => {
      void sendAgentInput(text).catch(() => {});
    });

    termRef.current = term;
    fitRef.current = fit;

    let rafId = 0;
    let fitCalls = 0;
    const applyFit = () => {
      fitCalls += 1;
      if (fitCalls <= 20 || fitCalls % 50 === 0) {
        klog("agent-term", `applyFit #${fitCalls} (host ${host.clientWidth}x${host.clientHeight})`);
      }
      if (safeFit(term, host, fit)) {
        klog("agent-term", `resized to ${term.cols}x${term.rows}`);
        void resizeAgent(term.rows, term.cols).catch(() => {});
      }
    };

    const observer = new ResizeObserver(() => {
      if (rafId) {
        return;
      }
      rafId = window.requestAnimationFrame(() => {
        rafId = 0;
        applyFit();
      });
    });
    observer.observe(host);
    rafId = window.requestAnimationFrame(() => {
      rafId = 0;
      applyFit();
    });

    return () => {
      if (rafId) {
        window.cancelAnimationFrame(rafId);
      }
      observer.disconnect();
      unlistenRef.current.forEach((fn) => fn());
      unlistenRef.current = [];
      void stopAgent().catch(() => {});
      clipboardRef.current?.dispose();
      clipboardRef.current = null;
      term.dispose();
      termRef.current = null;
      fitRef.current = null;
    };
  }, []);

  const handleLaunch = useCallback(async () => {
    const term = termRef.current;
    const fit = fitRef.current;
    if (!term || !fit) {
      return;
    }
    safeFit(term, hostRef.current, fit);
    term.focus();

    unlistenRef.current.forEach((fn) => fn());
    unlistenRef.current = [];

    try {
      const offOutput = await onAgentOutput((bytes) => term.write(bytes));
      const offExit = await onAgentExit((message) => {
        setStatus(message, { variant: "warning" });
        setRunning(false);
      });
      unlistenRef.current = [offOutput, offExit];

      setStatus(
        direct
          ? `${runtime} · account${cwd ? ` · ${cwd}` : ""}`
          : `${runtime} @ ${ollamaHost} · ${model}${cwd ? ` · ${cwd}` : ""}`,
        { variant: "success" },
      );
      await launchAgent({
        runtime,
        model,
        ollamaHost,
        cwd,
        direct,
        rows: term.rows,
        cols: term.cols,
      });
      setRunning(true);
    } catch (error) {
      const message = formatIpcError(error);
      term.writeln(`\r\n\x1b[31mLaunch failed: ${message}\x1b[0m`);
      setStatus(`Agent launch failed: ${message}`, { variant: "error" });
      toast.error(message);
      setRunning(false);
    }
  }, [runtime, model, ollamaHost, cwd, direct, setStatus, toast]);

  const handleStop = useCallback(async () => {
    await stopAgent().catch(() => {});
    setRunning(false);
    setStatus("Agent stopped", { variant: "info" });
  }, [setStatus]);

  return (
    <div className="flex h-full min-h-0 flex-col bg-nest-background">
      <header className="flex shrink-0 items-center gap-2 border-b border-nest-border px-2 py-1.5">
        <div className="min-w-0 flex-1">
          <span className="text-xs font-semibold uppercase tracking-wide text-nest-muted">
            Agent
          </span>
          <p
            className="truncate font-mono text-[11px] text-nest-muted"
            title={direct ? `${runtime} · account` : `${runtime} · ${model} · ${ollamaHost}`}
          >
            {direct ? `${runtime} · account` : `${runtime} · ${model} · ${ollamaHost}`}
          </p>
        </div>
        <div className="flex shrink-0 items-center gap-1.5">
          {running ? (
            <button
              type="button"
              onClick={() => void handleStop()}
              title="Stop agent"
              className="inline-flex h-6 items-center gap-1 rounded-nest-sm border border-nest-border px-2 text-xs text-nest-error hover:bg-nest-error/10"
            >
              <Icon icon={faXmark} className="size-3" />
              Stop
            </button>
          ) : (
            <button
              type="button"
              onClick={() => void handleLaunch()}
              title="Launch agent"
              className="inline-flex h-6 items-center gap-1 rounded-nest-sm bg-nest-primary px-2 text-xs font-medium text-white hover:opacity-90"
            >
              <Icon icon={faPlay} className="size-3" />
              Launch
            </button>
          )}
        </div>
      </header>
      <div className="min-h-0 flex-1 overflow-hidden p-1">
        <div ref={hostRef} className="h-full w-full" />
      </div>
    </div>
  );
}
