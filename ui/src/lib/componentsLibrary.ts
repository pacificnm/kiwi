import {
  AlertCircle,
  ListTree,
  MessageSquare,
  PanelTop,
  Square,
  Type,
  type LucideIcon,
} from "lucide-react";

export type ComponentCategory =
  | "inputs"
  | "feedback"
  | "navigation"
  | "surface"
  | "data-display"
  | "layout";

export type ComponentDef = {
  id: string;
  name: string;
  category: ComponentCategory;
  icon: LucideIcon;
  description: string;
};

export const COMPONENTS: ComponentDef[] = [
  { id: "button", name: "Button", category: "inputs", icon: Square, description: "Action buttons with variants" },
  { id: "icon-button", name: "IconButton", category: "inputs", icon: Square, description: "Icon-only buttons" },
  { id: "text-field", name: "TextField", category: "inputs", icon: Type, description: "Text input with label" },
  { id: "dialog", name: "Dialog", category: "feedback", icon: MessageSquare, description: "Modal overlays" },
  { id: "alert", name: "Alert", category: "feedback", icon: AlertCircle, description: "Inline messages" },
  { id: "snackbar", name: "Snackbar", category: "feedback", icon: MessageSquare, description: "Toast notifications" },
  { id: "app-bar", name: "AppBar", category: "navigation", icon: PanelTop, description: "Top application bar + toolbar" },
  { id: "menu", name: "Menu", category: "navigation", icon: ListTree, description: "Dropdown menus and File-menu bars" },
];

export const CATEGORIES: { id: ComponentCategory; label: string }[] = [
  { id: "inputs", label: "Inputs" },
  { id: "feedback", label: "Feedback" },
  { id: "navigation", label: "Navigation" },
  { id: "surface", label: "Surface" },
  { id: "data-display", label: "Data Display" },
  { id: "layout", label: "Layout" },
];

export function componentTabKey(id: string): string {
  return `nest-component:${id}`;
}

export function isComponentTab(relPath: string): boolean {
  return relPath.startsWith("nest-component:");
}
