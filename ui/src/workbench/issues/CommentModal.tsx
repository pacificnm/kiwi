import { useEffect, useId, useRef, useState } from "react";
import { faXmark } from "../../lib/fontawesome";
import { Icon } from "../../components/Icon";

type CommentModalProps = {
  open: boolean;
  issueNumber?: number;
  onSubmit: (issueNumber: number, body: string) => Promise<void>;
  onCancel: () => void;
};

/** Posts a new comment on a GitHub issue (Git menu + issue row context menu). */
export function CommentModal({ open, issueNumber, onSubmit, onCancel }: CommentModalProps) {
  const titleId = useId();
  const bodyId = useId();
  const numberRef = useRef<HTMLInputElement>(null);
  const bodyRef = useRef<HTMLTextAreaElement>(null);
  const [number, setNumber] = useState("");
  const [body, setBody] = useState("");
  const [error, setError] = useState<string | null>(null);
  const [submitting, setSubmitting] = useState(false);

  useEffect(() => {
    if (!open) {
      return;
    }
    setNumber(issueNumber ? String(issueNumber) : "");
    setBody("");
    setError(null);
    setSubmitting(false);
    const timer = window.setTimeout(() => {
      if (issueNumber) {
        bodyRef.current?.focus();
      } else {
        numberRef.current?.focus();
      }
    }, 0);
    return () => window.clearTimeout(timer);
  }, [open, issueNumber]);

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
    const parsed = Number.parseInt(number.trim(), 10);
    const trimmedBody = body.trim();
    if (!Number.isFinite(parsed) || parsed <= 0) {
      setError("Enter a valid issue number");
      return;
    }
    if (!trimmedBody) {
      setError("Comment cannot be empty");
      return;
    }
    setSubmitting(true);
    setError(null);
    onSubmit(parsed, trimmedBody)
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
            New Comment
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
          <label className="block text-xs text-nest-muted" htmlFor={bodyId.replace("body", "number")}>
            Issue number
            <input
              ref={numberRef}
              id={`${bodyId}-number`}
              value={number}
              onChange={(event) => setNumber(event.target.value)}
              inputMode="numeric"
              className="mt-1 h-8 w-full rounded-nest-sm border border-nest-border bg-nest-background px-2 text-sm text-nest-foreground focus:outline-none focus:ring-2 focus:ring-nest-accent/50"
            />
          </label>
          <label className="block text-xs text-nest-muted" htmlFor={bodyId}>
            Comment
            <textarea
              ref={bodyRef}
              id={bodyId}
              value={body}
              onChange={(event) => setBody(event.target.value)}
              rows={6}
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
            Post Comment
          </button>
        </footer>
      </div>
    </div>
  );
}
