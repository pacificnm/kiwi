import {
  Compass,
  Layers,
  LayoutGrid,
  MessageSquare,
  SlidersHorizontal,
  Table2,
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

export const CATEGORIES: { id: ComponentCategory; label: string }[] = [
  { id: "inputs", label: "Inputs" },
  { id: "feedback", label: "Feedback" },
  { id: "navigation", label: "Navigation" },
  { id: "surface", label: "Surface" },
  { id: "data-display", label: "Data Display" },
  { id: "layout", label: "Layout" },
];

const CATEGORY_ICON: Record<ComponentCategory, LucideIcon> = {
  inputs: SlidersHorizontal,
  feedback: MessageSquare,
  navigation: Compass,
  surface: Layers,
  "data-display": Table2,
  layout: LayoutGrid,
};

const CATEGORY_IDS = new Set<string>(CATEGORIES.map((category) => category.id));

// Every vendored component ships a `<Name>.docs.md`. Discover the full library
// from those docs — name + category from the path, description from the intro —
// so newly synced components appear here automatically with no manual registry.
const DOC_SOURCES = import.meta.glob("../nest-components/components/**/*.docs.md", {
  eager: true,
  query: "?raw",
  import: "default",
}) as Record<string, string>;

function toKebabCase(name: string): string {
  return name.replace(/([a-z0-9])([A-Z])/g, "$1-$2").toLowerCase();
}

function categoryFromPath(path: string): ComponentCategory | null {
  const parts = path.split("/");
  const segment = parts[parts.indexOf("components") + 1];
  return segment && CATEGORY_IDS.has(segment) ? (segment as ComponentCategory) : null;
}

function introFromMarkdown(markdown: string): string {
  for (const rawLine of markdown.split("\n")) {
    const line = rawLine.trim();
    if (!line || line.startsWith("#") || line.startsWith(">")) {
      continue;
    }
    // Strip Markdown link syntax, keeping the label text.
    return line.replace(/\[([^\]]+)\]\([^)]*\)/g, "$1");
  }
  return "";
}

export const COMPONENTS: ComponentDef[] = Object.entries(DOC_SOURCES)
  .map(([path, markdown]): ComponentDef | null => {
    const name = path.split("/").pop()!.replace(/\.docs\.md$/, "");
    const category = categoryFromPath(path);
    if (!category) {
      return null;
    }
    return {
      id: toKebabCase(name),
      name,
      category,
      icon: CATEGORY_ICON[category],
      description: introFromMarkdown(markdown),
    };
  })
  .filter((def): def is ComponentDef => def !== null)
  .sort((a, b) => a.name.localeCompare(b.name));

export function componentTabKey(id: string): string {
  return `nest-component:${id}`;
}

export function isComponentTab(relPath: string): boolean {
  return relPath.startsWith("nest-component:");
}
