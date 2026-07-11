import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { ClipboardAddon } from "@xterm/addon-clipboard";
import { Terminal } from "@xterm/xterm";
import { FitAddon } from "@xterm/addon-fit";
import "@xterm/xterm/css/xterm.css";
import type { UnlistenFn } from "@tauri-apps/api/event";
import { Icon, isTauri } from "../shell";
import { loadTerminalClipboard } from "../lib/terminalClipboard";
import { klog } from "../lib/log";
import { cssVar, safeFit, xtermTheme } from "../lib/xterm";
import { faPlus, faTerminal, faTrash, faXmark } from "../lib/fontawesome";
import {
  closeTerminal,
  onTerminalExit,
  onTerminalOutput,
  openTerminal,
  resizeTerminal,
  sendTerminalInput,
} from "../lib/terminal";
import { useWorkbench } from "./state";

type TerminalSession = { id: string; title: string };

/** One live shell terminal: owns an xterm instance bound to a backend PTY. */
function TerminalInstance({
  id,
  cwd,
  active,
  onExit,
}: {
  id: string;
  cwd: string;
  active: boolean;
  onExit: (id: string) => void;
}) {
  const hostRef = useRef<HTMLDivElement>(null);
  const termRef = useRef<Terminal | null>(null);
  const fitRef = useRef<FitAddon | null>(null);
  const clipboardRef = useRef<ClipboardAddon | null>(null);
  const exitedRef = useRef(false);

  useEffect(() => {
    if (!hostRef.current) {
      return;
    }
    const term = new Terminal({
      fontFamily: cssVar("--nest-font-mono", "JetBrains Mono, Consolas, monospace"),
      fontSize: 12,
      cursorBlink: true,
      theme: xtermTheme(),
    });
    const fit = new FitAddon();
    term.loadAddon(fit);
    const host = hostRef.current;
    term.open(host);
    termRef.current = term;
    fitRef.current = fit;

    term.onData((data) => {
      void sendTerminalInput(id, data).catch(() => {});
    });
    clipboardRef.current = loadTerminalClipboard(term, (text) => {
      void sendTerminalInput(id, text).catch(() => {});
    });

    const unlisten: UnlistenFn[] = [];
    let disposed = false;

    void onTerminalOutput((outId, bytes) => {
      if (outId === id) {
        term.write(bytes);
      }
    }).then((off) => (disposed ? off() : unlisten.push(off)));

    void onTerminalExit((exitId, message) => {
      if (exitId === id) {
        exitedRef.current = true;
        term.writeln(`\r\n\x1b[90m${message}\x1b[0m`);
        onExit(id);
      }
    }).then((off) => (disposed ? off() : unlisten.push(off)));

    let rafId = 0;
    let opened = false;
    const openPty = () => {
      if (opened) {
        return;
      }
      opened = true;
      klog("terminal", `open id=${id} ${term.cols}x${term.rows} cwd=${cwd || "(default)"}`);
      void openTerminal({ id, cwd, rows: term.rows, cols: term.cols }).catch((error) => {
        term.writeln(`\r\n\x1b[31mFailed to start shell: ${String(error)}\x1b[0m`);
      });
    };
    const applyFit = () => {
      const resized = safeFit(term, host, fit);
      if (!opened) {
        // Open the PTY at the freshly-measured grid so the shell starts at the
        // right width. Opening at 80x24 and then resizing makes bash redraw its
        // prompt on SIGWINCH (the "prompt printed 3 times" artifact).
        openPty();
      } else if (resized) {
        void resizeTerminal(id, term.rows, term.cols).catch(() => {});
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

    // First fit happens on the next frame (once the panel has laid out), then
    // the PTY opens at that size — no post-launch resize, so no prompt redraw.
    rafId = window.requestAnimationFrame(() => {
      rafId = 0;
      applyFit();
    });

    return () => {
      disposed = true;
      if (rafId) {
        window.cancelAnimationFrame(rafId);
      }
      observer.disconnect();
      unlisten.forEach((off) => off());
      if (!exitedRef.current) {
        void closeTerminal(id).catch(() => {});
      }
      clipboardRef.current?.dispose();
      clipboardRef.current = null;
      term.dispose();
      termRef.current = null;
      fitRef.current = null;
    };
  }, [id, cwd, onExit]);

  // On becoming visible, refit to the now-measurable container and focus.
  useEffect(() => {
    if (!active) {
      return;
    }
    const raf = window.requestAnimationFrame(() => {
      const term = termRef.current;
      if (term && safeFit(term, hostRef.current, fitRef.current)) {
        void resizeTerminal(id, term.rows, term.cols).catch(() => {});
      }
      term?.focus();
    });
    return () => window.cancelAnimationFrame(raf);
  }, [active, id]);

  return (
    <div
      className={active ? "absolute inset-0 p-1" : "absolute inset-0 hidden"}
      onClick={() => termRef.current?.focus()}
      role="presentation"
    >
      <div ref={hostRef} className="h-full w-full" />
    </div>
  );
}

/** Bottom-panel Terminal tab: manages one or more shell sessions. */
export function TerminalPanel() {
  const { workspace } = useWorkbench();
  const cwd = workspace?.root ?? "";
  const [sessions, setSessions] = useState<TerminalSession[]>([]);
  const [activeId, setActiveId] = useState<string | null>(null);
  const counterRef = useRef(0);
  const autoSpawnedRef = useRef(false);

  const newTerminal = useCallback(() => {
    counterRef.current += 1;
    const id =
      typeof crypto !== "undefined" && "randomUUID" in crypto
        ? crypto.randomUUID()
        : `term-${Date.now()}-${counterRef.current}`;
    const title = `Terminal ${counterRef.current}`;
    setSessions((current) => [...current, { id, title }]);
    setActiveId(id);
  }, []);

  const removeTerminal = useCallback((id: string) => {
    setSessions((current) => {
      const next = current.filter((session) => session.id !== id);
      setActiveId((active) => {
        if (active !== id) {
          return active;
        }
        return next.length > 0 ? next[next.length - 1].id : null;
      });
      return next;
    });
  }, []);

  const killActive = useCallback(() => {
    if (activeId) {
      removeTerminal(activeId);
    }
  }, [activeId, removeTerminal]);

  // Spawn the first terminal automatically inside the desktop host.
  // The ref guard makes this fire exactly once even under React StrictMode,
  // which double-invokes mount effects (the cause of two terminals on launch).
  useEffect(() => {
    if (autoSpawnedRef.current) {
      return;
    }
    autoSpawnedRef.current = true;
    if (isTauri()) {
      newTerminal();
    }
    // Only on mount; subsequent empties are handled by the empty state.
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  const showList = sessions.length > 1;

  const content = useMemo(() => {
    if (!isTauri()) {
      return (
        <p className="px-3 py-2 text-xs text-nest-muted">
          The integrated terminal is available in the desktop app.
        </p>
      );
    }
    if (sessions.length === 0) {
      return (
        <div className="flex h-full flex-col items-center justify-center gap-2 text-xs text-nest-muted">
          <Icon icon={faTerminal} className="size-5" />
          <p>No open terminals.</p>
          <button
            type="button"
            onClick={newTerminal}
            className="inline-flex items-center gap-1 rounded-nest-sm bg-nest-primary px-2 py-1 text-xs font-medium text-white hover:opacity-90"
          >
            <Icon icon={faPlus} className="size-3" />
            New Terminal
          </button>
        </div>
      );
    }
    return sessions.map((session) => (
      <TerminalInstance
        key={session.id}
        id={session.id}
        cwd={cwd}
        active={session.id === activeId}
        onExit={removeTerminal}
      />
    ));
  }, [sessions, activeId, cwd, newTerminal, removeTerminal]);

  return (
    <div className="flex h-full min-h-0 bg-nest-background">
      <div className="relative min-h-0 flex-1">{content}</div>

      <div className="flex w-8 shrink-0 flex-col items-center gap-0.5 border-l border-nest-border bg-nest-surface py-1">
        <button
          type="button"
          onClick={newTerminal}
          title="New Terminal"
          aria-label="New Terminal"
          className="flex size-6 items-center justify-center rounded-nest-sm text-nest-muted hover:bg-nest-muted/10 hover:text-nest-foreground"
        >
          <Icon icon={faPlus} className="size-3" />
        </button>
        <button
          type="button"
          onClick={killActive}
          disabled={!activeId}
          title="Kill Terminal"
          aria-label="Kill Terminal"
          className="flex size-6 items-center justify-center rounded-nest-sm text-nest-muted hover:bg-nest-error/10 hover:text-nest-error disabled:opacity-40"
        >
          <Icon icon={faTrash} className="size-3" />
        </button>
      </div>

      {showList ? (
        <ul className="flex w-40 shrink-0 flex-col border-l border-nest-border bg-nest-surface py-1 text-xs">
          {sessions.map((session) => (
            <li key={session.id}>
              <div
                className={[
                  "group flex items-center gap-1.5 px-2 py-1",
                  session.id === activeId
                    ? "bg-nest-accent/15 text-nest-foreground"
                    : "text-nest-muted hover:bg-nest-muted/10",
                ].join(" ")}
              >
                <button
                  type="button"
                  onClick={() => setActiveId(session.id)}
                  className="flex min-w-0 flex-1 items-center gap-1.5 text-left"
                >
                  <Icon icon={faTerminal} className="size-3 shrink-0" />
                  <span className="truncate">{session.title}</span>
                </button>
                <button
                  type="button"
                  onClick={() => removeTerminal(session.id)}
                  title="Close"
                  aria-label={`Close ${session.title}`}
                  className="shrink-0 rounded-nest-sm p-0.5 text-nest-muted opacity-0 hover:bg-nest-muted/20 hover:text-nest-foreground group-hover:opacity-100"
                >
                  <Icon icon={faXmark} className="size-2.5" />
                </button>
              </div>
            </li>
          ))}
        </ul>
      ) : null}
    </div>
  );
}
