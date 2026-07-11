import { useEffect, useId } from "react";
import { faXmark } from "../../lib/fontawesome";
import { Icon } from "../../components/Icon";

type MetadataItem = {
  id: string;
  title: string;
  subtitle?: string;
  color?: string;
};

type MetadataListModalProps = {
  open: boolean;
  title: string;
  loading: boolean;
  emptyMessage: string;
  items: MetadataItem[];
  onClose: () => void;
};

/** Read-only list modal for repository labels or milestones (v1). */
export function MetadataListModal({
  open,
  title,
  loading,
  emptyMessage,
  items,
  onClose,
}: MetadataListModalProps) {
  const titleId = useId();

  useEffect(() => {
    if (!open) {
      return;
    }
    const onKeyDown = (event: KeyboardEvent) => {
      if (event.key === "Escape") {
        onClose();
      }
    };
    window.addEventListener("keydown", onKeyDown);
    return () => window.removeEventListener("keydown", onKeyDown);
  }, [open, onClose]);

  if (!open) {
    return null;
  }

  return (
    <div
      className="fixed inset-0 z-[70] flex items-center justify-center bg-black/40 p-4"
      onClick={onClose}
      role="presentation"
    >
      <div
        role="dialog"
        aria-labelledby={titleId}
        className="flex max-h-[70vh] w-full max-w-md flex-col rounded-nest-md border border-nest-border bg-nest-surface shadow-xl"
        onClick={(event) => event.stopPropagation()}
      >
        <header className="flex items-center justify-between border-b border-nest-border px-4 py-3">
          <h2 id={titleId} className="text-sm font-semibold text-nest-foreground">
            {title}
          </h2>
          <button
            type="button"
            onClick={onClose}
            aria-label="Close"
            className="flex size-7 items-center justify-center rounded-nest-sm text-nest-muted hover:bg-nest-muted/10"
          >
            <Icon icon={faXmark} className="size-3.5" />
          </button>
        </header>
        <div className="min-h-0 flex-1 overflow-auto p-2">
          {loading ? (
            <p className="px-2 py-4 text-xs text-nest-muted">Loading…</p>
          ) : items.length === 0 ? (
            <p className="px-2 py-4 text-xs text-nest-muted">{emptyMessage}</p>
          ) : (
            <ul className="space-y-1">
              {items.map((item) => (
                <li
                  key={item.id}
                  className="rounded-nest-sm px-2 py-1.5 text-xs hover:bg-nest-muted/10"
                >
                  <div className="flex items-center gap-2">
                    {item.color ? (
                      <span
                        className="size-2.5 shrink-0 rounded-full border border-nest-border"
                        style={{ backgroundColor: `#${item.color}` }}
                      />
                    ) : null}
                    <span className="font-medium text-nest-foreground">{item.title}</span>
                  </div>
                  {item.subtitle ? (
                    <p className="mt-0.5 pl-4 text-[11px] text-nest-muted">{item.subtitle}</p>
                  ) : null}
                </li>
              ))}
            </ul>
          )}
        </div>
      </div>
    </div>
  );
}
