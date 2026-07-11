import * as monaco from "monaco-editor";
import { loader } from "@monaco-editor/react";
import editorWorker from "monaco-editor/esm/vs/editor/editor.worker?worker";
import jsonWorker from "monaco-editor/esm/vs/language/json/json.worker?worker";
import cssWorker from "monaco-editor/esm/vs/language/css/css.worker?worker";
import htmlWorker from "monaco-editor/esm/vs/language/html/html.worker?worker";
import tsWorker from "monaco-editor/esm/vs/language/typescript/ts.worker?worker";
import { cssVar } from "./xterm";

/** Monaco theme name defined from the active Nest theme. */
export const KIWI_MONACO_THEME = "kiwi-dark";

let configured = false;

/**
 * Wires Monaco to load entirely from the local bundle.
 *
 * `@monaco-editor/react` fetches Monaco from a CDN by default, which breaks in
 * the offline Tauri host. `loader.config({ monaco })` points it at the bundled
 * instance, and `self.MonacoEnvironment.getWorker` supplies the Vite-built web
 * workers so language services (TS/JSON/CSS/HTML) run off the main thread.
 *
 * Idempotent — safe to call from every editor mount.
 */
export function setupMonaco(): typeof monaco {
  if (configured) {
    return monaco;
  }
  configured = true;

  self.MonacoEnvironment = {
    getWorker(_workerId, label) {
      switch (label) {
        case "json":
          return new jsonWorker();
        case "css":
        case "scss":
        case "less":
          return new cssWorker();
        case "html":
        case "handlebars":
        case "razor":
          return new htmlWorker();
        case "typescript":
        case "javascript":
          return new tsWorker();
        default:
          return new editorWorker();
      }
    },
  };

  loader.config({ monaco });
  return monaco;
}

/**
 * Defines (and returns) a Monaco theme derived from the active Nest theme's CSS
 * variables so the editor matches the surrounding workbench chrome.
 */
export function defineKiwiTheme(instance: typeof monaco): string {
  instance.editor.defineTheme(KIWI_MONACO_THEME, {
    base: "vs-dark",
    inherit: true,
    rules: [],
    colors: {
      "editor.background": cssVar("--nest-color-background", "#1b1f23"),
      "editor.foreground": cssVar("--nest-color-foreground", "#cccccc"),
      "editorLineNumber.foreground": cssVar("--nest-color-muted", "#6b7280"),
      "editorCursor.foreground": cssVar("--nest-color-accent", "#4f8ef7"),
      "editor.lineHighlightBackground": cssVar("--nest-color-surface", "#22272e"),
      "editor.selectionBackground": cssVar("--nest-color-accent-muted", "#264f78"),
      "editorWidget.background": cssVar("--nest-color-surface", "#22272e"),
      "editorWidget.border": cssVar("--nest-color-border", "#30363d"),
    },
  });
  return KIWI_MONACO_THEME;
}

/** Maps a file name / relative path to a Monaco language id. */
export function languageForFilename(name: string): string {
  const base = name.split(/[\\/]/).pop() ?? name;
  const lower = base.toLowerCase();

  const byName: Record<string, string> = {
    dockerfile: "dockerfile",
    makefile: "makefile",
    "cargo.toml": "toml",
    "cargo.lock": "toml",
    ".gitignore": "plaintext",
  };
  if (byName[lower]) {
    return byName[lower];
  }

  const ext = lower.includes(".") ? lower.slice(lower.lastIndexOf(".") + 1) : "";
  const byExt: Record<string, string> = {
    rs: "rust",
    ts: "typescript",
    tsx: "typescript",
    js: "javascript",
    jsx: "javascript",
    mjs: "javascript",
    cjs: "javascript",
    json: "json",
    jsonc: "json",
    css: "css",
    scss: "scss",
    less: "less",
    html: "html",
    htm: "html",
    xml: "xml",
    svg: "xml",
    md: "markdown",
    markdown: "markdown",
    py: "python",
    go: "go",
    java: "java",
    c: "c",
    h: "c",
    cpp: "cpp",
    cc: "cpp",
    hpp: "cpp",
    cs: "csharp",
    sh: "shell",
    bash: "shell",
    zsh: "shell",
    toml: "toml",
    yaml: "yaml",
    yml: "yaml",
    sql: "sql",
    swift: "swift",
    rb: "ruby",
    php: "php",
    lua: "lua",
    dockerfile: "dockerfile",
  };
  return byExt[ext] ?? "plaintext";
}
