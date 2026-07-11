import { SETTINGS_GROUPS, SETTINGS_ITEMS, settingsTabKey, type SettingsGroupId } from "../lib/settings";
import { faChevronLeft } from "../lib/fontawesome";
import { Icon } from "../shell";
import { useWorkbench } from "./state";

type SettingsPanelProps = {
  onToggleCollapse?: () => void;
};

/** Settings Activity sidebar — lists settings grouped by category; opens each as an editor tab. */
export function SettingsPanel({ onToggleCollapse }: SettingsPanelProps) {
  const { activePath, openSetting } = useWorkbench();

  return (
    <div className="flex h-full min-h-0 flex-col bg-nest-background">
      <header className="flex h-9 shrink-0 items-center border-b border-nest-border px-3">
        <span className="text-xs font-semibold uppercase tracking-wide text-nest-muted">Settings</span>
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
        {SETTINGS_GROUPS.length === 0 ? (
          <p className="px-3 py-4 text-xs text-nest-muted">No settings yet.</p>
        ) : (
          SETTINGS_GROUPS.map((group) => (
            <GroupSection
              key={group.id}
              groupId={group.id}
              label={group.label}
              activePath={activePath}
              onSelect={openSetting}
            />
          ))
        )}
      </div>
    </div>
  );
}

function GroupSection({
  groupId,
  label,
  activePath,
  onSelect,
}: {
  groupId: SettingsGroupId;
  label: string;
  activePath: string | null;
  onSelect: (item: (typeof SETTINGS_ITEMS)[number]) => void;
}) {
  const items = SETTINGS_ITEMS.filter((item) => item.groupId === groupId);

  return (
    <div className="border-b border-nest-border">
      <div className="px-3 py-1.5 text-[11px] font-semibold uppercase tracking-wide text-nest-muted">
        {label}
      </div>
      {items.length === 0 ? (
        <p className="px-3 pb-2 text-[11px] italic text-nest-muted/70">Coming soon</p>
      ) : (
        <ul className="pb-1">
          {items.map((item) => {
            const selected = activePath === settingsTabKey(item.id);
            return (
              <li key={item.id}>
                <button
                  type="button"
                  onClick={() => onSelect(item)}
                  title={item.description}
                  className={[
                    "flex w-full items-center gap-2 px-3 py-1.5 text-left text-xs transition-colors",
                    selected
                      ? "bg-nest-accent/15 text-nest-foreground"
                      : "text-nest-muted hover:bg-nest-muted/10 hover:text-nest-foreground",
                  ].join(" ")}
                >
                  <span className="min-w-0 flex-1 truncate">{item.label}</span>
                </button>
              </li>
            );
          })}
        </ul>
      )}
    </div>
  );
}
