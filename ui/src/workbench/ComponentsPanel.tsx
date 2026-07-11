import { CATEGORIES, COMPONENTS, componentTabKey, type ComponentCategory } from "../lib/componentsLibrary";
import { faChevronLeft } from "../lib/fontawesome";
import { Icon } from "../shell";
import { useWorkbench } from "./state";

type ComponentsPanelProps = {
  onToggleCollapse?: () => void;
};

/** Components Activity sidebar — browses the vendored @nest/components library. */
export function ComponentsPanel({ onToggleCollapse }: ComponentsPanelProps) {
  const { activePath, openComponent } = useWorkbench();

  return (
    <div className="flex h-full min-h-0 flex-col bg-nest-background">
      <header className="flex h-9 shrink-0 items-center border-b border-nest-border px-3">
        <span className="text-xs font-semibold uppercase tracking-wide text-nest-muted">Components</span>
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
      <div className="min-h-0 flex-1 overflow-auto">
        {CATEGORIES.map((category) => (
          <CategorySection
            key={category.id}
            category={category.id}
            label={category.label}
            activePath={activePath}
            onSelect={openComponent}
          />
        ))}
      </div>
    </div>
  );
}

function CategorySection({
  category,
  label,
  activePath,
  onSelect,
}: {
  category: ComponentCategory;
  label: string;
  activePath: string | null;
  onSelect: (def: (typeof COMPONENTS)[number]) => void;
}) {
  const items = COMPONENTS.filter((component) => component.category === category);

  return (
    <div className="border-b border-nest-border">
      <div className="px-3 py-1.5 text-[11px] font-semibold uppercase tracking-wide text-nest-muted">
        {label}
      </div>
      {items.length === 0 ? (
        <p className="px-3 pb-2 text-[11px] italic text-nest-muted/70">Coming soon</p>
      ) : (
        <ul className="pb-1">
          {items.map((component) => {
            const selected = activePath === componentTabKey(component.id);
            return (
              <li key={component.id}>
                <button
                  type="button"
                  onClick={() => onSelect(component)}
                  title={component.description}
                  className={[
                    "flex w-full items-center gap-2 px-3 py-1.5 text-left text-xs transition-colors",
                    selected
                      ? "bg-nest-accent/15 text-nest-foreground"
                      : "text-nest-muted hover:bg-nest-muted/10 hover:text-nest-foreground",
                  ].join(" ")}
                >
                  <component.icon className="size-3.5 shrink-0" />
                  <span className="min-w-0 flex-1 truncate">{component.name}</span>
                </button>
              </li>
            );
          })}
        </ul>
      )}
    </div>
  );
}
