import { useEffect, useId, useRef, useState } from "react";
import { faXmark } from "../../lib/fontawesome";
import { Icon } from "../../components/Icon";

type NewIssueModalProps = {
  open: boolean;
  onSubmit: (title: string, body: string) => Promise<void>;
  onCancel: () => void;
};

/** Creates a new GitHub issue (Git → New Issue). */
export function NewIssueModal({ open, onSubmit, onCancel }: NewIssueModalProps) {
  const titleId = useId();
  const titleFieldId = useId();
  const bodyFieldId = useId();
  const titleRef = useRef<HTMLInputElement>(null);
  const [title, setTitle] = useState("");
  const [body, setBody] = useState("");
  const [error, setError] = useState<string | null>(null);
  const [submitting, setSubmitting] = useState(false);

  useEffect(() => {
    if (!open) {
      return;
    }
    setTitle("");
    setBody("");
    setError(null);
    setSubmitting(false);
    const timer = window.setTimeout(() => titleRef.current?.focus(), 0);
    return () => window.clearTimeout(timer);
  }, [open]);

  useEffect(() => {
    if (!open) {
      return;
    }
    const onKeyDown = (event: KeyboardEvent) => {
      if (event.key === "Escape") {
        onCancel();
      }
    };
    window.addEventListener("keydown", onKeyDown);
    return () => window.removeEventListener("keydown", onKeyDown);
  }, [open, onCancel]);

  if (!open) {
    return null;
  }

  const submit = () => {
    const trimmedTitle = title.trim();
    if (!trimmedTitle) {
      setError("Title is required");
      return;
    }
    setSubmitting(true);
    setError(null);
    onSubmit(trimmedTitle, body)
      .catch((err: unknown) => {
        setError(err instanceof Error ? err.message : String(err));
      })
      .finally(() => setSubmitting(false));
  };

  return (
    <div
      className="fixed inset-0 z-[70] flex items-center justify-center bg-black/40 p-4"
      onClick={onCancel}
      role="presentation"
    >
      <div
        role="dialog"
        aria-labelledby={titleId}
        className="w-full max-w-lg rounded-nest-md border border-nest-border bg-nest-surface shadow-xl"
        onClick={(event) => event.stopPropagation()}
      >
        <header className="flex items-center justify-between border-b border-nest-border px-4 py-3">
          <h2 id={titleId} className="text-sm font-semibold text-nest-foreground">
            New Issue
          </h2>
          <button
            type="button"
            onClick={onCancel}
            aria-label="Close"
            className="flex size-7 items-center justify-center rounded-nest-sm text-nest-muted hover:bg-nest-muted/10"
          >
            <Icon icon={faXmark} className="size-3.5" />
          </button>
        </header>
        <div className="space-y-3 p-4">
          <label className="block text-xs text-nest-muted" htmlFor={titleFieldId}>
            Title
            <input
              ref={titleRef}
              id={titleFieldId}
              value={title}
              onChange={(event) => setTitle(event.target.value)}
              className="mt-1 h-8 w-full rounded-nest-sm border border-nest-border bg-nest-background px-2 text-sm text-nest-foreground focus:outline-none focus:ring-2 focus:ring-nest-accent/50"
            />
          </label>
          <label className="block text-xs text-nest-muted" htmlFor={bodyFieldId}>
            Description
            <textarea
              id={bodyFieldId}
              value={body}
              onChange={(event) => setBody(event.target.value)}
              rows={8}
              className="mt-1 w-full resize-y rounded-nest-sm border border-nest-border bg-nest-background px-2 py-1.5 text-sm text-nest-foreground focus:outline-none focus:ring-2 focus:ring-nest-accent/50"
            />
          </label>
          {error ? <p className="text-xs text-nest-error">{error}</p> : null}
        </div>
        <footer className="flex justify-end gap-2 border-t border-nest-border px-4 py-3">
          <button
            type="button"
            onClick={onCancel}
            className="h-8 rounded-nest-sm px-3 text-xs text-nest-muted hover:bg-nest-muted/10"
          >
            Cancel
          </button>
          <button
            type="button"
            disabled={submitting}
            onClick={submit}
            className="h-8 rounded-nest-sm bg-nest-accent px-3 text-xs font-semibold text-nest-background disabled:opacity-50"
          >
            Create Issue
          </button>
        </footer>
      </div>
    </div>
  );
}
