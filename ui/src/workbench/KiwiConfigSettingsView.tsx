import { useCallback, useEffect, useRef, useState } from "react";
import Editor, { type BeforeMount, type OnMount } from "@monaco-editor/react";
import { formatIpcError } from "../lib/agent";
import { kiwiConfigPath, kiwiConfigRead, kiwiConfigWrite } from "../lib/kiwiConfig";
import { defineKiwiTheme, KIWI_MONACO_THEME, setupMonaco } from "../lib/monaco";
import { isTauri } from "../shell";

setupMonaco();

/** Settings detail view for the "Kiwi Config" item — edits `config.toml` directly. */
export function KiwiConfigSettingsView() {
  const [path, setPath] = useState<string | null>(null);
  const [content, setContent] = useState("");
  const [savedContent, setSavedContent] = useState("");
  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const contentRef = useRef(content);
  contentRef.current = content;

  const load = useCallback(async () => {
    setLoading(true);
    try {
      const [resolvedPath, text] = await Promise.all([kiwiConfigPath(), kiwiConfigRead()]);
      setPath(resolvedPath);
      setContent(text);
      setSavedContent(text);
      setError(null);
    } catch (loadError) {
      setError(formatIpcError(loadError));
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    if (!isTauri()) {
      setLoading(false);
      return;
    }
    void load();
  }, [load]);

  const dirty = content !== savedContent;

  const save = useCallback(async () => {
    setSaving(true);
    setError(null);
    try {
      await kiwiConfigWrite(contentRef.current);
      setSavedContent(contentRef.current);
    } catch (saveError) {
      setError(formatIpcError(saveError));
    } finally {
      setSaving(false);
    }
  }, []);

  const saveRef = useRef(save);
  saveRef.current = save;

  const beforeMount: BeforeMount = useCallback((monaco) => {
    defineKiwiTheme(monaco);
  }, []);

  const onMount: OnMount = useCallback((editorInstance, monaco) => {
    editorInstance.addCommand(monaco.KeyMod.CtrlCmd | monaco.KeyCode.KeyS, () => {
      void saveRef.current();
    });
  }, []);

  if (!isTauri()) {
    return (
      <div className="flex h-full items-center justify-center text-sm text-nest-muted">
        Kiwi Config is available in the desktop app.
      </div>
    );
  }

  return (
    <div className="flex h-full min-h-0 flex-col bg-nest-background">
      <header className="flex items-start justify-between gap-4 border-b border-nest-border bg-nest-surface/60 px-6 py-5">
        <div className="min-w-0">
          <h1 className="text-xl font-semibold text-nest-foreground">Kiwi Config</h1>
          <p className="mt-0.5 text-sm text-nest-muted">
            Kiwi&rsquo;s default settings — <code className="font-mono text-[12px]">agent</code>,{" "}
            <code className="font-mono text-[12px]">ai</code>,{" "}
            <code className="font-mono text-[12px]">project</code>,{" "}
            <code className="font-mono text-[12px]">swift</code>, and more.
          </p>
          {path ? (
            <p className="mt-1 truncate font-mono text-[11px] text-nest-muted">{path}</p>
          ) : null}
        </div>
        <div className="flex shrink-0 items-center gap-2">
          {error ? <span className="text-xs text-nest-error">{error}</span> : null}
          <button
            type="button"
            onClick={() => void load()}
            disabled={loading || saving}
            className="h-7 rounded-nest-sm border border-nest-border px-3 text-xs hover:bg-nest-muted/10 disabled:opacity-50"
          >
            Reload
          </button>
          <button
            type="button"
            onClick={() => void save()}
            disabled={!dirty || saving}
            className="h-7 rounded-nest-sm border border-nest-border px-3 text-xs hover:bg-nest-muted/10 disabled:opacity-50"
          >
            {saving ? "Saving…" : dirty ? "Save (Ctrl/Cmd+S)" : "Saved"}
          </button>
        </div>
      </header>
      <div className="min-h-0 flex-1">
        {loading ? (
          <p className="p-3 text-xs text-nest-muted">Loading…</p>
        ) : (
          <Editor
            className="h-full w-full"
            theme={KIWI_MONACO_THEME}
            path={path ?? "config.toml"}
            language="toml"
            value={content}
            beforeMount={beforeMount}
            onMount={onMount}
            onChange={(value) => setContent(value ?? "")}
            loading={<div className="p-3 text-xs text-nest-muted">Loading editor…</div>}
            options={{
              automaticLayout: true,
              fontSize: 13,
              fontFamily: "ui-monospace, SFMono-Regular, Menlo, Consolas, monospace",
              minimap: { enabled: false },
              scrollBeyondLastLine: false,
              smoothScrolling: true,
              tabSize: 2,
            }}
          />
        )}
      </div>
    </div>
  );
}
