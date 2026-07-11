import { useCallback, useEffect, useMemo, useRef, useState, type FormEvent } from "react";
import { formatIpcError } from "../lib/agent";
import { faChevronLeft } from "../lib/fontawesome";
import { baseName } from "../lib/workspace";
import {
  workspaceReplaceAll,
  workspaceSearch,
  type WorkspaceSearchFile,
  type WorkspaceSearchQuery,
  type WorkspaceSearchResponse,
} from "../lib/workspace";
import { Icon, isTauri, useToast } from "../shell";
import { useSearchPanelFocus } from "./searchPanelFocus";
import { useWorkbench } from "./state";

type SearchPanelProps = {
  onToggleCollapse?: () => void;
};

export function SearchPanel({ onToggleCollapse }: SearchPanelProps) {
  const { openFile } = useWorkbench();
  const { focusTarget, clearFocus } = useSearchPanelFocus();
  const toast = useToast();
  const findRef = useRef<HTMLInputElement>(null);
  const replaceRef = useRef<HTMLInputElement>(null);
  const [query, setQuery] = useState("");
  const [replace, setReplace] = useState("");
  const [includes, setIncludes] = useState("");
  const [excludes, setExcludes] = useState("");
  const [matchCase, setMatchCase] = useState(false);
  const [wholeWord, setWholeWord] = useState(false);
  const [useRegex, setUseRegex] = useState(false);
  const [busy, setBusy] = useState(false);
  const [results, setResults] = useState<WorkspaceSearchResponse | null>(null);

  const searchQuery: WorkspaceSearchQuery = useMemo(
    () => ({
      query,
      includes: includes.trim() ? includes : undefined,
      excludes: excludes.trim() ? excludes : undefined,
      matchCase,
      wholeWord,
      useRegex,
      maxMatches: 2000,
    }),
    [excludes, includes, matchCase, query, useRegex, wholeWord],
  );

  const runSearch = useCallback(async () => {
    if (!isTauri()) {
      toast.info("Search requires the desktop app");
      return;
    }
    setBusy(true);
    try {
      const next = await workspaceSearch(searchQuery);
      setResults(next);
    } catch (error) {
      toast.error(formatIpcError(error));
    } finally {
      setBusy(false);
    }
  }, [searchQuery, toast]);

  const runReplaceAll = useCallback(async () => {
    if (!isTauri()) {
      toast.info("Replace requires the desktop app");
      return;
    }
    if (!searchQuery.query.trim()) {
      return;
    }
    setBusy(true);
    try {
      const replaced = await workspaceReplaceAll({ search: searchQuery, replace });
      toast.success(`Replaced ${replaced.matchCount} matches in ${replaced.fileCount} files`);
      const next = await workspaceSearch(searchQuery);
      setResults(next);
    } catch (error) {
      toast.error(formatIpcError(error));
    } finally {
      setBusy(false);
    }
  }, [replace, searchQuery, toast]);

  const onSubmit = useCallback(
    (event: FormEvent) => {
      event.preventDefault();
      void runSearch();
    },
    [runSearch],
  );

  useEffect(() => {
    if (focusTarget === "find") {
      findRef.current?.focus();
      findRef.current?.select();
      clearFocus();
      return;
    }
    if (focusTarget === "replace") {
      replaceRef.current?.focus();
      replaceRef.current?.select();
      clearFocus();
    }
  }, [clearFocus, focusTarget]);

  return (
    <div className="flex h-full min-h-0 flex-col bg-nest-background">
      <header className="flex h-9 shrink-0 items-center gap-2 border-b border-nest-border px-3">
        <span className="truncate text-xs font-semibold uppercase tracking-wide text-nest-muted">
          Search
        </span>
        <div className="ml-auto flex items-center gap-0.5">
          {onToggleCollapse ? (
            <button
              type="button"
              onClick={onToggleCollapse}
              title="Hide sidebar"
              aria-label="Hide sidebar"
              className="flex size-6 items-center justify-center rounded-nest-sm text-nest-muted hover:bg-nest-muted/10 hover:text-nest-foreground"
            >
              <Icon icon={faChevronLeft} className="size-3" />
            </button>
          ) : null}
        </div>
      </header>

      <div className="min-h-0 flex-1 overflow-auto p-3">
        <form onSubmit={onSubmit} className="flex flex-col gap-2">
          <div className="flex items-center gap-1">
            <input
              ref={findRef}
              value={query}
              onChange={(e) => setQuery(e.target.value)}
              placeholder="Search"
              spellCheck={false}
              className="h-7 flex-1 rounded-nest-sm border border-nest-border bg-nest-surface px-2 text-sm text-nest-foreground placeholder:text-nest-muted focus:outline-none focus:ring-2 focus:ring-nest-accent/50"
            />
            <ToggleButton
              active={matchCase}
              label="Aa"
              title="Match Case"
              onClick={() => setMatchCase((v) => !v)}
            />
            <ToggleButton
              active={wholeWord}
              label="ab"
              title="Match Whole Word"
              onClick={() => setWholeWord((v) => !v)}
            />
            <ToggleButton
              active={useRegex}
              label=".*"
              title="Use Regular Expression"
              onClick={() => setUseRegex((v) => !v)}
            />
          </div>

          <div className="flex items-center gap-1">
            <input
              ref={replaceRef}
              value={replace}
              onChange={(e) => setReplace(e.target.value)}
              placeholder="Replace"
              spellCheck={false}
              className="h-7 flex-1 rounded-nest-sm border border-nest-border bg-nest-surface px-2 text-sm text-nest-foreground placeholder:text-nest-muted focus:outline-none focus:ring-2 focus:ring-nest-accent/50"
            />
            <button
              type="button"
              onClick={() => void runReplaceAll()}
              disabled={busy || !query.trim()}
              className="h-7 shrink-0 rounded-nest-sm bg-nest-muted/10 px-2 text-xs font-medium text-nest-foreground hover:bg-nest-muted/15 disabled:opacity-50"
              title="Replace All"
            >
              Replace All
            </button>
          </div>

          <input
            value={includes}
            onChange={(e) => setIncludes(e.target.value)}
            placeholder="Files to include"
            spellCheck={false}
            className="h-7 rounded-nest-sm border border-nest-border bg-nest-surface px-2 text-sm text-nest-foreground placeholder:text-nest-muted focus:outline-none focus:ring-2 focus:ring-nest-accent/50"
          />
          <input
            value={excludes}
            onChange={(e) => setExcludes(e.target.value)}
            placeholder="Files to exclude"
            spellCheck={false}
            className="h-7 rounded-nest-sm border border-nest-border bg-nest-surface px-2 text-sm text-nest-foreground placeholder:text-nest-muted focus:outline-none focus:ring-2 focus:ring-nest-accent/50"
          />

          <div className="flex items-center gap-2 pt-1">
            <button
              type="submit"
              disabled={busy}
              className="h-7 rounded-nest-sm bg-nest-accent px-3 text-xs font-semibold text-nest-background hover:brightness-110 disabled:opacity-50"
            >
              Search
            </button>
            {results ? (
              <span className="text-xs text-nest-muted">
                {results.matchCount} results in {results.fileCount} files
                {results.truncated ? " (truncated)" : ""}
              </span>
            ) : (
              <span className="text-xs text-nest-muted">Enter a search term to find matches</span>
            )}
          </div>
        </form>

        <div className="pt-3">
          {results?.files?.length ? (
            <div className="flex flex-col gap-2">
              {results.files.map((file) => (
                <FileResult
                  key={file.relPath}
                  file={file}
                  onOpen={(relPath) => openFile(relPath, baseName(relPath))}
                />
              ))}
            </div>
          ) : results ? (
            <div className="pt-3 text-xs text-nest-muted">No results found</div>
          ) : null}
        </div>
      </div>
    </div>
  );
}

function ToggleButton({
  active,
  label,
  title,
  onClick,
}: {
  active: boolean;
  label: string;
  title: string;
  onClick: () => void;
}) {
  return (
    <button
      type="button"
      onClick={onClick}
      title={title}
      aria-pressed={active}
      className={[
        "flex h-7 w-8 items-center justify-center rounded-nest-sm border text-xs font-semibold",
        active
          ? "border-nest-accent bg-nest-accent/15 text-nest-foreground"
          : "border-nest-border bg-nest-surface text-nest-muted hover:bg-nest-muted/10 hover:text-nest-foreground",
      ].join(" ")}
    >
      {label}
    </button>
  );
}

function FileResult({
  file,
  onOpen,
}: {
  file: WorkspaceSearchFile;
  onOpen: (relPath: string) => void;
}) {
  return (
    <div className="rounded-nest-sm border border-nest-border bg-nest-surface">
      <button
        type="button"
        onClick={() => onOpen(file.relPath)}
        className="flex w-full items-center justify-between gap-2 px-2 py-1 text-left text-xs font-medium text-nest-foreground hover:bg-nest-muted/10"
      >
        <span className="truncate">{file.relPath}</span>
        <span className="shrink-0 text-nest-muted">{file.matches.length}</span>
      </button>
      <div className="border-t border-nest-border">
        {file.matches.slice(0, 20).map((m) => (
          <button
            key={`${m.line}:${m.col}:${m.matchText}`}
            type="button"
            onClick={() => onOpen(file.relPath)}
            className="flex w-full gap-2 px-2 py-1 text-left text-xs text-nest-foreground hover:bg-nest-muted/10"
            title={`${file.relPath}:${m.line}:${m.col}`}
          >
            <span className="shrink-0 tabular-nums text-nest-muted">
              {m.line}:{m.col}
            </span>
            <span className="min-w-0 flex-1 truncate text-nest-foreground/90">{m.lineText}</span>
          </button>
        ))}
        {file.matches.length > 20 ? (
          <div className="px-2 py-1 text-[11px] text-nest-muted">
            Showing first 20 matches
          </div>
        ) : null}
      </div>
    </div>
  );
}

