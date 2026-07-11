export type SettingsGroupId = string;

export type SettingsGroup = {
  id: SettingsGroupId;
  label: string;
};

export type SettingsItem = {
  id: string;
  groupId: SettingsGroupId;
  label: string;
  description: string;
};

/** Settings groups, in display order. */
export const SETTINGS_GROUPS: SettingsGroup[] = [
  { id: "general", label: "General" },
  { id: "help", label: "Help" },
];

/** Settings items, grouped via `groupId`. */
export const SETTINGS_ITEMS: SettingsItem[] = [
  {
    id: "kiwi-config",
    groupId: "general",
    label: "Kiwi Config",
    description: "Kiwi's default settings, from ~/.config/kiwi/config.toml.",
  },
  {
    id: "doc-sources",
    groupId: "help",
    label: "Doc Sources",
    description: "Manage the projects whose documentation is synced into Help.",
  },
];

export function settingsTabKey(id: string): string {
  return `settings:${id}`;
}

export function isSettingsTab(relPath: string): boolean {
  return relPath.startsWith("settings:");
}
