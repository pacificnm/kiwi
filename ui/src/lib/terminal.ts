import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { kiwiInvoke } from "./ipc";

const OUTPUT_EVENT = "kiwi://terminal-output";
const EXIT_EVENT = "kiwi://terminal-exit";

type TerminalOutput = { id: string; base64: string };
type TerminalExit = { id: string; message: string };

export type OpenTerminalArgs = {
  id: string;
  cwd?: string | null;
  shell?: string | null;
  rows: number;
  cols: number;
};

/** Opens an interactive shell terminal keyed by `id`. */
export async function openTerminal(args: OpenTerminalArgs): Promise<void> {
  const cwd = args.cwd?.trim();
  return kiwiInvoke("terminal_open", {
    id: args.id,
    cwd: cwd && cwd.length > 0 ? cwd : null,
    shell: args.shell ?? null,
    rows: args.rows,
    cols: args.cols,
  });
}

/** Keystrokes — high-frequency; backend logs at debug only. */
export async function sendTerminalInput(id: string, data: string): Promise<void> {
  return invoke("plugin:kiwi|terminal_input", { id, data });
}

/** PTY resize — high-frequency; backend logs at debug only. */
export async function resizeTerminal(id: string, rows: number, cols: number): Promise<void> {
  return invoke("plugin:kiwi|terminal_resize", { id, rows, cols });
}

/** Closes a terminal session (hangs up its shell). */
export async function closeTerminal(id: string): Promise<void> {
  return kiwiInvoke("terminal_close", { id });
}

/** Ids of all live terminal sessions. */
export async function listTerminals(): Promise<string[]> {
  return kiwiInvoke<string[]>("terminal_list");
}

/** Subscribes to terminal output; decodes base64 to raw bytes per session id. */
export async function onTerminalOutput(
  handler: (id: string, bytes: Uint8Array) => void,
): Promise<UnlistenFn> {
  return listen<TerminalOutput>(OUTPUT_EVENT, (event) => {
    handler(event.payload.id, decodeBase64(event.payload.base64));
  });
}

/** Subscribes to terminal exit events (per session id). */
export async function onTerminalExit(
  handler: (id: string, message: string) => void,
): Promise<UnlistenFn> {
  return listen<TerminalExit>(EXIT_EVENT, (event) => {
    handler(event.payload.id, event.payload.message);
  });
}

function decodeBase64(value: string): Uint8Array {
  const binary = atob(value);
  const bytes = new Uint8Array(binary.length);
  for (let i = 0; i < binary.length; i += 1) {
    bytes[i] = binary.charCodeAt(i);
  }
  return bytes;
}
