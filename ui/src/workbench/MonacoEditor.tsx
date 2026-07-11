import { useCallback, useEffect, useMemo, useRef } from "react";
import Editor, { type BeforeMount, type OnMount } from "@monaco-editor/react";
import type { editor } from "monaco-editor";
import {
  defineKiwiTheme,
  KIWI_MONACO_THEME,
  languageForFilename,
  setupMonaco,
} from "../lib/monaco";
import { useEditorCommands } from "./editorCommands";
import { useEditorNavigation } from "./editorNavigation";
import type { EditorTab } from "./state";
import { isIssueTab } from "../lib/github";

// Point @monaco-editor/react at the bundled Monaco + local workers (no CDN).
setupMonaco();

type Props = {
  /** The tab whose contents this editor is showing. */
  tab: EditorTab;
  /** Called on every edit with the tab's relative path and new content. */
  onChange: (relPath: string, content: string) => void;
  /** Called when the user presses Ctrl/Cmd+S. */
  onSave: (relPath: string) => void;
};

/**
 * Monaco-backed code editor for the active editor tab.
 *
 * A single Monaco instance is reused across tabs; switching `path` swaps the
 * underlying model, so per-file content, undo history, and scroll/cursor state
 * are preserved without mounting one editor per tab.
 */
export function MonacoEditor({ tab, onChange, onSave }: Props) {
  const { registerEditor } = useEditorCommands();
  const { location, clear } = useEditorNavigation();
  // Keep the save keybinding pointed at the current tab without re-registering.
  const relRef = useRef(tab.relPath);
  relRef.current = tab.relPath;
  const saveRef = useRef(onSave);
  saveRef.current = onSave;
  const editorRef = useRef<editor.IStandaloneCodeEditor | null>(null);
  const unregisterRef = useRef<(() => void) | null>(null);
  const readOnly = tab.loading || isIssueTab(tab.relPath);
  const readOnlyRef = useRef(readOnly);
  readOnlyRef.current = readOnly;

  const beforeMount: BeforeMount = useCallback((monaco) => {
    defineKiwiTheme(monaco);
  }, []);

  const onMount: OnMount = useCallback((editorInstance, monaco) => {
    editorRef.current = editorInstance;
    editorInstance.updateOptions({ readOnly: readOnlyRef.current });
    editorInstance.addCommand(monaco.KeyMod.CtrlCmd | monaco.KeyCode.KeyS, () => {
      saveRef.current(relRef.current);
    });
    unregisterRef.current?.();
    unregisterRef.current = registerEditor(editorInstance);
  }, [registerEditor]);

  // @monaco-editor/react does not always re-apply `options` after mount; sync
  // readOnly when a tab finishes loading or when switching issue vs file tabs.
  useEffect(() => {
    editorRef.current?.updateOptions({ readOnly });
    if (editorRef.current) {
      unregisterRef.current?.();
      unregisterRef.current = registerEditor(editorRef.current);
    }
  }, [readOnly, registerEditor]);

  useEffect(
    () => () => {
      unregisterRef.current?.();
      unregisterRef.current = null;
      registerEditor(null);
    },
    [registerEditor],
  );

  useEffect(() => {
    if (!location || location.relPath !== tab.relPath) {
      return;
    }
    const editorInstance = editorRef.current;
    if (!editorInstance) {
      return;
    }
    editorInstance.revealLineInCenter(location.line);
    editorInstance.setPosition({ lineNumber: location.line, column: location.col });
    editorInstance.focus();
    clear();
  }, [clear, location, tab.relPath]);

  const editorOptions = useMemo(
    () => ({
      readOnly,
      automaticLayout: true,
      fontSize: 13,
      fontFamily: "ui-monospace, SFMono-Regular, Menlo, Consolas, monospace",
      minimap: { enabled: true },
      scrollBeyondLastLine: false,
      smoothScrolling: true,
      tabSize: 2,
      renderWhitespace: "selection" as const,
      fixedOverflowWidgets: true,
    }),
    [readOnly],
  );

  return (
    <Editor
      key={tab.relPath}
      className="h-full w-full"
      theme={KIWI_MONACO_THEME}
      path={tab.relPath}
      language={isIssueTab(tab.relPath) ? "markdown" : languageForFilename(tab.name)}
      defaultValue={tab.content}
      beforeMount={beforeMount}
      onMount={onMount}
      onChange={(value) => onChange(relRef.current, value ?? "")}
      loading={<div className="p-3 text-xs text-nest-muted">Loading editor…</div>}
      options={editorOptions}
    />
  );
}
