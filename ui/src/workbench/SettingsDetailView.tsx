import { SETTINGS_ITEMS } from "../lib/settings";

type SettingsDetailViewProps = {
  itemId: string;
};

/** Detail pane for one settings item, opened as an editor tab from SettingsPanel. */
export function SettingsDetailView({ itemId }: SettingsDetailViewProps) {
  const item = SETTINGS_ITEMS.find((candidate) => candidate.id === itemId);

  if (!item) {
    return (
      <div className="flex h-full items-center justify-center text-sm text-nest-muted">
        Unknown setting &ldquo;{itemId}&rdquo;.
      </div>
    );
  }

  return (
    <div className="flex h-full min-h-0 flex-col bg-nest-background">
      <header className="border-b border-nest-border bg-nest-surface/60 px-6 py-5">
        <h1 className="text-xl font-semibold text-nest-foreground">{item.label}</h1>
        <p className="mt-0.5 text-sm text-nest-muted">{item.description}</p>
      </header>
      <div className="min-h-0 flex-1 overflow-y-auto px-6 py-6 text-sm text-nest-muted">
        Coming soon.
      </div>
    </div>
  );
}
