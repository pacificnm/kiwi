import { useEffect, useId, useRef, useState } from "react";
import { faXmark } from "../lib/fontawesome";
import { Icon } from "./Icon";

type PromptDialogProps = {
  open: boolean;
  title: string;
  /** Helper text shown above the input (e.g. "Create a new file in src"). */
  description?: string;
  /** Field label. */
  label: string;
  /** Initial input value (e.g. current name for Rename). */
  initialValue?: string;
  placeholder?: string;
  confirmLabel?: string;
  /**
   * Called on submit. Return a rejected promise (or throw) with a message to
   * surface a validation / I/O error and keep the dialog open.
   */
  onSubmit: (value: string) => Promise<void>;
  onCancel: () => void;
};

/** Single-line text prompt modal (New File / New Folder / Rename). */
export function PromptDialog({
  open,
  title,
  description,
  label,
  initialValue = "",
  placeholder,
  confirmLabel = "Create",
  onSubmit,
  onCancel,
}: PromptDialogProps) {
  const titleId = useId();
  const inputId = useId();
  const inputRef = useRef<HTMLInputElement>(null);
  const [value, setValue] = useState(initialValue);
  const [error, setError] = useState<string | null>(null);
  const [submitting, setSubmitting] = useState(false);

  useEffect(() => {
    if (open) {
      setValue(initialValue);
      setError(null);
      setSubmitting(false);
    }
  }, [open, initialValue]);

  useEffect(() => {
    if (!open) {
      return;
    }
    const timer = window.setTimeout(() => {
      const input = inputRef.current;
      if (!input) {
        return;
      }
      input.focus();
      const dot = input.value.lastIndexOf(".");
      input.setSelectionRange(0, dot > 0 ? dot : input.value.length);
    }, 0);
    const onKeyDown = (event: KeyboardEvent) => {
      if (event.key === "Escape") {
        onCancel();
      }
    };
    window.addEventListener("keydown", onKeyDown);
    return () => {
      window.clearTimeout(timer);
      window.removeEventListener("keydown", onKeyDown);
    };
  }, [open, onCancel]);

  if (!open) {
    return null;
  }

  const submit = () => {
    const trimmed = value.trim();
    if (!trimmed || submitting) {
      return;
    }
    setSubmitting(true);
    setError(null);
    onSubmit(trimmed)
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
        aria-modal="true"
        aria-labelledby={titleId}
        className="w-full max-w-md overflow-hidden rounded-nest-lg border border-nest-border bg-nest-surface shadow-xl"
        onClick={(event) => event.stopPropagation()}
      >
        <header className="flex items-center justify-between border-b border-nest-border px-5 py-3">
          <h2 id={titleId} className="text-sm font-semibold">
            {title}
          </h2>
          <button
            type="button"
            onClick={onCancel}
            className="rounded-nest-sm p-1 text-nest-muted hover:bg-nest-muted/10 hover:text-nest-foreground"
            aria-label="Close"
          >
            <Icon icon={faXmark} className="size-3.5" />
          </button>
        </header>

        <div className="px-5 py-4">
          {description ? (
            <p className="mb-3 text-xs text-nest-muted">{description}</p>
          ) : null}
          <label htmlFor={inputId} className="mb-1 block text-xs font-medium text-nest-foreground">
            {label}
          </label>
          <input
            id={inputId}
            ref={inputRef}
            type="text"
            value={value}
            placeholder={placeholder}
            spellCheck={false}
            autoComplete="off"
            onChange={(event) => setValue(event.target.value)}
            onKeyDown={(event) => {
              if (event.key === "Enter") {
                event.preventDefault();
                submit();
              }
            }}
            className="w-full rounded-nest-sm border border-nest-border bg-nest-background px-2 py-1.5 text-sm text-nest-foreground outline-none focus:border-nest-accent"
          />
          {error ? <p className="mt-2 text-xs text-nest-error">{error}</p> : null}
        </div>

        <footer className="flex justify-end gap-2 border-t border-nest-border px-5 py-3">
          <button
            type="button"
            onClick={onCancel}
            className="rounded-nest-md border border-nest-border px-4 py-2 text-sm hover:bg-nest-muted/10"
          >
            Cancel
          </button>
          <button
            type="button"
            onClick={submit}
            disabled={!value.trim() || submitting}
            className="rounded-nest-md bg-nest-primary px-4 py-2 text-sm font-medium text-white hover:bg-nest-secondary disabled:cursor-default disabled:opacity-50"
          >
            {submitting ? "Working…" : confirmLabel}
          </button>
        </footer>
      </div>
    </div>
  );
}
