import type { IconDefinition } from "@fortawesome/fontawesome-svg-core";
import {
  faBars,
  faCheck,
  faCircleExclamation,
  faCircleQuestion,
  faFolder,
  faLink,
  faMagnifyingGlass,
  faPalette,
  faPlus,
  faShapes,
  faUser,
} from "../lib/fontawesome";

/** Activity bar selection — controls left sidebar content. */
export type ActivityId =
  | "explorer"
  | "search"
  | "source-control"
  | "issues"
  | "tasks"
  | "agent"
  | "tools"
  | "components"
  | "theme"
  | "help"
  | "extensions";

export type ActivityDef = {
  id: ActivityId;
  label: string;
  icon: IconDefinition;
};

/** All activity bar items in display order. */
export const ACTIVITIES: ActivityDef[] = [
  { id: "explorer", label: "Explorer", icon: faFolder },
  { id: "search", label: "Search", icon: faMagnifyingGlass },
  { id: "source-control", label: "Source Control", icon: faLink },
  { id: "issues", label: "Issues", icon: faCircleExclamation },
  { id: "tasks", label: "Tasks", icon: faCheck },
  { id: "agent", label: "Agent", icon: faUser },
  { id: "tools", label: "Tools", icon: faBars },
  { id: "components", label: "Components", icon: faShapes },
  { id: "theme", label: "Theme", icon: faPalette },
  { id: "help", label: "Help", icon: faCircleQuestion },
  { id: "extensions", label: "Extensions", icon: faPlus },
];

/** Workbench layout constants (px). */
export const LAYOUT = {
  menuBarHeight: 32,
  activityBarWidth: 48,
  sidebarDefault: 260,
  sidebarMin: 200,
  sidebarMax: 400,
  aiPanelDefault: 360,
  aiPanelMin: 240,
  bottomPanelDefault: 200,
  bottomPanelMin: 80,
} as const;
