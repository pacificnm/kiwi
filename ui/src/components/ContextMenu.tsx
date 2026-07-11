import { useEffect, useLayoutEffect, useRef, useState } from "react";
import { createPortal } from "react-dom";

/** One entry in a {@link ContextMenu}. A `separator` renders a divider. */
export type ContextMenuItem =
  | {
      kind?: "item";
      /** Stable id for React keys. */
      id: string;
      /** Menu label. */
      label: string;
      /** Disable (grey out, non-clickable) the entry. */
      disabled?: boolean;
      /** Render in the danger color (e.g. Delete). */
      danger?: boolean;
      /** Invoked when the entry is chosen. */
      onSelect: () => void;
    }
  | { kind: "separator"; id: string };

type ContextMenuProps = {
  /** Viewport coordinates where the menu was opened. */
  x: number;
  y: number;
  items: ContextMenuItem[];
  onClose: () => void;
};

/** Floating right-click menu, positioned at (x, y) and clamped to the viewport. */
export function ContextMenu({ x, y, items, onClose }: ContextMenuProps) {
  const ref = useRef<HTMLDivElement>(null);
  const [pos, setPos] = useState({ left: x, top: y });

  useLayoutEffect(() => {
    const node = ref.current;
    if (!node) {
      return;
    }
    const rect = node.getBoundingClientRect();
    const margin = 4;
    const left = Math.min(x, window.innerWidth - rect.width - margin);
    const top = Math.min(y, window.innerHeight - rect.height - margin);
    setPos({ left: Math.max(margin, left), top: Math.max(margin, top) });
  }, [x, y]);

  useEffect(() => {
    const onPointerDown = (event: MouseEvent) => {
      if (ref.current && !ref.current.contains(event.target as Node)) {
        onClose();
      }
    };
    const onKeyDown = (event: KeyboardEvent) => {
      if (event.key === "Escape") {
        onClose();
      }
    };
    window.addEventListener("mousedown", onPointerDown, true);
    window.addEventListener("contextmenu", onPointerDown, true);
    window.addEventListener("keydown", onKeyDown);
    window.addEventListener("blur", onClose);
    return () => {
      window.removeEventListener("mousedown", onPointerDown, true);
      window.removeEventListener("contextmenu", onPointerDown, true);
      window.removeEventListener("keydown", onKeyDown);
      window.removeEventListener("blur", onClose);
    };
  }, [onClose]);

  return createPortal(
    <div
      ref={ref}
      role="menu"
      className="fixed z-[80] min-w-[200px] overflow-hidden rounded-nest-md border border-nest-border bg-nest-surface py-1 text-[13px] shadow-xl"
      style={{ left: pos.left, top: pos.top }}
      onContextMenu={(event) => event.preventDefault()}
    >
      {items.map((item) =>
        item.kind === "separator" ? (
          <div key={item.id} className="my-1 h-px bg-nest-border" role="separator" />
        ) : (
          <button
            key={item.id}
            type="button"
            role="menuitem"
            disabled={item.disabled}
            onClick={() => {
              if (item.disabled) {
                return;
              }
              onClose();
              item.onSelect();
            }}
            className={[
              "flex w-full items-center px-3 py-1 text-left",
              item.disabled
                ? "cursor-default text-nest-muted/50"
                : item.danger
                  ? "text-nest-error hover:bg-nest-error/10"
                  : "text-nest-foreground hover:bg-nest-accent/15",
            ].join(" ")}
          >
            {item.label}
          </button>
        ),
      )}
    </div>,
    document.body,
  );
}
