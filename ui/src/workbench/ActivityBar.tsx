import type { ReactNode } from "react";
import { ACTIVITIES, type ActivityId } from "./activity";
import { Icon } from "../components/Icon";
import { faChevronLeft, faGear } from "../lib/fontawesome";

type ActivityBarProps = {
  /** Highlighted activity, or `null` when the sidebar is collapsed. */
  active: ActivityId | null;
  onSelect: (id: ActivityId) => void;
};

/** Full-height activity icon column (48px). */
export function ActivityBar({ active, onSelect }: ActivityBarProps) {
  return (
    <nav
      className="flex w-12 shrink-0 flex-col border-r border-nest-border bg-nest-surface"
      aria-label="Activity bar"
    >
      <div className="flex flex-1 flex-col gap-0.5 pt-2">
        {ACTIVITIES.map((item) => {
          const selected = item.id === active;
          return (
            <button
              key={item.id}
              type="button"
              title={item.label}
              aria-label={item.label}
              aria-current={selected ? "page" : undefined}
              onClick={() => onSelect(item.id)}
              className={[
                "relative flex h-9 w-full items-center justify-center transition-colors",
                selected
                  ? "text-nest-foreground before:absolute before:left-0 before:top-1 before:h-7 before:w-0.5 before:rounded-r before:bg-nest-accent"
                  : "text-nest-muted hover:text-nest-foreground",
              ].join(" ")}
            >
              <Icon icon={item.icon} className="size-4" />
            </button>
          );
        })}
      </div>
      <div className="border-t border-nest-border p-2">
        <button
          type="button"
          title="Settings"
          aria-label="Settings"
          aria-current={active === "settings" ? "page" : undefined}
          onClick={() => onSelect("settings")}
          className={[
            "flex h-9 w-full items-center justify-center transition-colors",
            active === "settings" ? "text-nest-foreground" : "text-nest-muted hover:text-nest-foreground",
          ].join(" ")}
        >
          <Icon icon={faGear} className="size-4" />
        </button>
      </div>
    </nav>
  );
}

type PanelPlaceholderProps = {
  title: string;
  children?: ReactNode;
  /** Collapses the sidebar panel. */
  onToggleCollapse?: () => void;
};

/** P0 placeholder chrome for sidebar / editor / bottom / AI regions. */
export function PanelPlaceholder({ title, children, onToggleCollapse }: PanelPlaceholderProps) {
  return (
    <div className="flex h-full min-h-0 flex-col bg-nest-background">
      <header className="flex h-9 shrink-0 items-center border-b border-nest-border px-3">
        <span className="text-xs font-semibold uppercase tracking-wide text-nest-muted">{title}</span>
        {onToggleCollapse ? (
          <button
            type="button"
            onClick={onToggleCollapse}
            title="Hide sidebar"
            aria-label="Hide sidebar"
            className="ml-auto flex size-6 items-center justify-center rounded-nest-sm text-nest-muted hover:bg-nest-muted/10 hover:text-nest-foreground"
          >
            <Icon icon={faChevronLeft} className="size-3" />
          </button>
        ) : null}
      </header>
      <div className="min-h-0 flex-1 overflow-auto p-3 text-sm text-nest-muted">
        {children ?? <p className="text-xs">Coming in a later migration phase.</p>}
      </div>
    </div>
  );
}
