import {
  ClipboardAddon,
  type IClipboardProvider,
} from "@xterm/addon-clipboard";
import type { Terminal } from "@xterm/xterm";
import { readText, writeText } from "@tauri-apps/plugin-clipboard-manager";
import { isTauri } from "./tauri";

/** Bridges xterm OSC 52 + keyboard shortcuts to the OS clipboard. */
class KiwiClipboardProvider implements IClipboardProvider {
  readText(_selection: string): Promise<string> {
    if (isTauri()) {
      return readText();
    }
    return navigator.clipboard.readText();
  }

  writeText(_selection: string, text: string): Promise<void> {
    if (isTauri()) {
      return writeText(text);
    }
    return navigator.clipboard.writeText(text);
  }
}

function wantsCopy(event: KeyboardEvent): boolean {
  if (event.type !== "keydown") {
    return false;
  }
  if (event.ctrlKey && event.key === "Insert" && !event.shiftKey) {
    return true;
  }
  return (event.ctrlKey || event.metaKey) && event.shiftKey && event.key.toLowerCase() === "c";
}

function wantsPaste(event: KeyboardEvent): boolean {
  if (event.type !== "keydown") {
    return false;
  }
  if (event.shiftKey && event.key === "Insert" && !event.ctrlKey) {
    return true;
  }
  return (event.ctrlKey || event.metaKey) && event.shiftKey && event.key.toLowerCase() === "v";
}

/**
 * Enables clipboard for an embedded PTY terminal (Agent + shell tabs).
 *
 * OpenCode and other TUIs copy via OSC 52; without this addon the UI can show
 * "copied to clipboard" while nothing reaches the OS. Uses Tauri's clipboard
 * plugin on desktop for reliable Linux Wayland/X11 access.
 */
export function loadTerminalClipboard(
  term: Terminal,
  onPaste: (text: string) => void,
): ClipboardAddon {
  const provider = new KiwiClipboardProvider();
  const addon = new ClipboardAddon(undefined, provider);
  term.loadAddon(addon);

  term.attachCustomKeyEventHandler((event) => {
    if (wantsCopy(event)) {
      const selection = term.getSelection();
      if (selection) {
        void provider.writeText("c", selection);
      }
      return false;
    }
    if (wantsPaste(event)) {
      void provider.readText("c").then((text) => {
        if (text) {
          onPaste(text);
        }
      });
      return false;
    }
    return true;
  });

  return addon;
}
