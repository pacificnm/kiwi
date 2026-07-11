import { useMemo, useState } from "react";
import { changeBadge, type GitChangeKind, type GitCommitChanges } from "../lib/git";
import { languageForFilename } from "../lib/monaco";

const STATUS_KIND: Record<string, GitChangeKind> = {
  A: "added",
  M: "modified",
  D: "deleted",
  R: "renamed",
  C: "copied",
};

const KIND_COLOR: Record<GitChangeKind, string> = {
  modified: "text-nest-warning",
  added: "text-nest-success",
  deleted: "text-nest-error",
  renamed: "text-nest-accent",
  copied: "text-nest-accent",
  untracked: "text-nest-success",
  other: "text-nest-muted",
};

type CommitChangesViewProps = {
  changes: GitCommitChanges;
};

export function CommitChangesView({ changes }: CommitChangesViewProps) {
  const [selectedPath, setSelectedPath] = useState(changes.files[0]?.path ?? null);
  const selected = useMemo(
    () => changes.files.find((file) => file.path === selectedPath) ?? null,
    [changes.files, selectedPath],
  );

  return (
    <div className="flex h-full min-h-0 flex-col bg-nest-background">
      <header className="shrink-0 border-b border-nest-border px-4 py-3">
        <h1 className="text-sm font-semibold text-nest-foreground">{changes.subject}</h1>
        <p className="mt-1 text-xs text-nest-muted">
          <span className="font-mono text-nest-foreground/80">{changes.shortHash}</span>
          {" · "}
          {changes.files.length} file{changes.files.length === 1 ? "" : "s"} changed
        </p>
      </header>

      <div className="flex min-h-0 flex-1">
        <aside className="w-64 shrink-0 overflow-auto border-r border-nest-border bg-nest-surface/40">
          <p className="px-3 py-2 text-[10px] font-semibold uppercase tracking-wide text-nest-muted">
            Changed files
          </p>
          <ul>
            {changes.files.map((file) => {
              const kind = STATUS_KIND[file.status] ?? "other";
              const active = file.path === selectedPath;
              return (
                <li key={file.path}>
                  <button
                    type="button"
                    onClick={() => setSelectedPath(file.path)}
                    title={file.path}
                    className={[
                      "flex w-full items-center gap-2 px-3 py-1.5 text-left text-xs",
                      active
                        ? "bg-nest-accent/15 text-nest-foreground"
                        : "text-nest-foreground hover:bg-nest-muted/10",
                    ].join(" ")}
                  >
                    <span
                      className={[
                        "w-3 shrink-0 text-center text-[10px] font-semibold",
                        KIND_COLOR[kind],
                      ].join(" ")}
                    >
                      {changeBadge(kind)}
                    </span>
                    <span className="min-w-0 flex-1 truncate">{baseName(file.path)}</span>
                  </button>
                  {file.oldPath ? (
                    <p className="truncate px-3 pb-1 pl-8 text-[10px] text-nest-muted">
                      {file.oldPath}
                    </p>
                  ) : null}
                </li>
              );
            })}
          </ul>
        </aside>

        <section className="min-w-0 flex-1 overflow-auto">
          {selected ? (
            <div className="min-h-full">
              <div className="sticky top-0 z-10 border-b border-nest-border bg-nest-background/95 px-4 py-2 backdrop-blur">
                <p className="truncate font-mono text-xs text-nest-foreground">{selected.path}</p>
                <p className="text-[10px] text-nest-muted">
                  {languageForFilename(selected.path)} · {statusLabel(selected.status)}
                </p>
              </div>
              {selected.diff.trim() ? (
                <DiffBody diff={selected.diff} />
              ) : (
                <p className="px-4 py-6 text-sm text-nest-muted">No diff available for this file.</p>
              )}
            </div>
          ) : (
            <p className="px-4 py-6 text-sm text-nest-muted">Select a file to view its changes.</p>
          )}
        </section>
      </div>
    </div>
  );
}

function DiffBody({ diff }: { diff: string }) {
  return (
    <pre className="font-mono text-[12px] leading-5">
      {diff.split("\n").map((line, index) => (
        <DiffLine key={`${index}-${line}`} line={line} />
      ))}
    </pre>
  );
}

function DiffLine({ line }: { line: string }) {
  let className = "block whitespace-pre px-4";
  if (line.startsWith("+++") || line.startsWith("---")) {
    className += " bg-nest-muted/10 text-nest-muted";
  } else if (line.startsWith("@@")) {
    className += " bg-nest-accent/10 text-nest-accent";
  } else if (line.startsWith("+")) {
    className += " bg-green-500/15 text-green-300";
  } else if (line.startsWith("-")) {
    className += " bg-red-500/15 text-red-300";
  } else if (line.startsWith("diff --git")) {
    className += " bg-nest-surface text-nest-muted";
  } else {
    className += " text-nest-foreground/90";
  }
  return <code className={className}>{line || " "}</code>;
}

function statusLabel(status: string): string {
  switch (status) {
    case "A":
      return "Added";
    case "M":
      return "Modified";
    case "D":
      return "Deleted";
    case "R":
      return "Renamed";
    case "C":
      return "Copied";
    default:
      return "Changed";
  }
}

function baseName(path: string): string {
  const parts = path.split("/");
  return parts[parts.length - 1] || path;
}
