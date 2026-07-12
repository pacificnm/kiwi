import { useMemo, useState, type ComponentType } from "react";
import { BookOpen, Code as CodeIcon, Eye, type LucideIcon } from "lucide-react";
import ReactMarkdown from "react-markdown";
import remarkGfm from "remark-gfm";
import { COMPONENTS } from "../lib/componentsLibrary";

type DetailTab = "preview" | "docs" | "code";
type DemoModule = Record<string, unknown>;

// Data-driven from the vendored library: live demos, doc markdown, and demo
// source are discovered by glob and keyed by component name, so every synced
// component gets Preview / Documentation / Code with no per-component wiring.
const DEMO_MODULES = import.meta.glob("../nest-components/components/**/*.demo.tsx", {
  eager: true,
}) as Record<string, DemoModule>;
const DEMO_SOURCES = import.meta.glob("../nest-components/components/**/*.demo.tsx", {
  eager: true,
  query: "?raw",
  import: "default",
}) as Record<string, string>;
const DOC_SOURCES = import.meta.glob("../nest-components/components/**/*.docs.md", {
  eager: true,
  query: "?raw",
  import: "default",
}) as Record<string, string>;

function indexByName<T>(glob: Record<string, T>, suffix: string): Record<string, T> {
  const byName: Record<string, T> = {};
  for (const [path, value] of Object.entries(glob)) {
    const name = path.split("/").pop()!.replace(suffix, "");
    byName[name] = value;
  }
  return byName;
}

const DEMOS = indexByName(DEMO_MODULES, ".demo.tsx");
const CODE = indexByName(DEMO_SOURCES, ".demo.tsx");
const DOCS = indexByName(DOC_SOURCES, ".docs.md");

/** Demos export either `default` or a named `<Name>Demos` function. */
function resolveDemoComponent(mod: DemoModule | undefined): ComponentType | null {
  if (!mod) {
    return null;
  }
  if (typeof mod.default === "function") {
    return mod.default as ComponentType;
  }
  const named = Object.values(mod).find((value) => typeof value === "function");
  return (named as ComponentType | undefined) ?? null;
}

const MARKDOWN_CLASSES = [
  "max-w-[820px] text-sm leading-relaxed text-nest-foreground",
  "[&_h1]:mb-1 [&_h1]:text-xl [&_h1]:font-semibold",
  "[&_h2]:mb-2 [&_h2]:mt-6 [&_h2]:text-[15px] [&_h2]:font-semibold",
  "[&_h3]:mb-1 [&_h3]:mt-4 [&_h3]:font-semibold",
  "[&_p]:my-2 [&_p]:text-nest-foreground/90",
  "[&_ul]:my-2 [&_ul]:list-disc [&_ul]:pl-5 [&_li]:my-1",
  "[&_a]:text-nest-primary [&_a]:underline",
  "[&_code]:rounded [&_code]:bg-nest-muted/15 [&_code]:px-1 [&_code]:py-0.5 [&_code]:font-mono [&_code]:text-[12.5px]",
  "[&_pre]:my-3 [&_pre]:overflow-x-auto [&_pre]:rounded-nest-md [&_pre]:border [&_pre]:border-nest-border [&_pre]:bg-nest-background [&_pre]:p-3",
  "[&_pre_code]:bg-transparent [&_pre_code]:p-0",
  "[&_table]:my-3 [&_table]:w-full [&_table]:border-collapse [&_table]:text-left",
  "[&_th]:border-b [&_th]:border-nest-border [&_th]:py-1.5 [&_th]:pr-4 [&_th]:font-semibold",
  "[&_td]:border-b [&_td]:border-nest-border/50 [&_td]:py-1.5 [&_td]:pr-4 [&_td]:align-top",
].join(" ");

/** Read-only Preview/Docs/Code viewer for one @nest/components component. */
export function ComponentDetailView({ componentId }: { componentId: string }) {
  const [tab, setTab] = useState<DetailTab>("preview");
  const def = COMPONENTS.find((component) => component.id === componentId);

  if (!def) {
    return (
      <div className="flex h-full items-center justify-center text-sm text-nest-muted">
        Unknown component &ldquo;{componentId}&rdquo;.
      </div>
    );
  }

  return (
    <div className="flex h-full min-h-0 flex-col bg-nest-background">
      <header className="border-b border-nest-border bg-nest-surface/60 px-6 py-5">
        <div className="flex items-center gap-3">
          <def.icon className="size-6 text-nest-primary" />
          <div>
            <h1 className="text-xl font-semibold text-nest-foreground">{def.name}</h1>
            <p className="mt-0.5 text-sm text-nest-muted">{def.description}</p>
          </div>
        </div>
        <div className="mt-4 flex gap-1">
          <DetailTabButton icon={Eye} label="Preview" active={tab === "preview"} onClick={() => setTab("preview")} />
          <DetailTabButton
            icon={BookOpen}
            label="Documentation"
            active={tab === "docs"}
            onClick={() => setTab("docs")}
          />
          <DetailTabButton icon={CodeIcon} label="Code" active={tab === "code"} onClick={() => setTab("code")} />
        </div>
      </header>
      <div className="min-h-0 flex-1 overflow-y-auto px-6 py-6">
        {tab === "preview" ? <ComponentPreview name={def.name} /> : null}
        {tab === "docs" ? <ComponentDocs name={def.name} /> : null}
        {tab === "code" ? <ComponentCode name={def.name} /> : null}
      </div>
    </div>
  );
}

function DetailTabButton({
  icon: Icon,
  label,
  active,
  onClick,
}: {
  icon: LucideIcon;
  label: string;
  active: boolean;
  onClick: () => void;
}) {
  return (
    <button
      type="button"
      onClick={onClick}
      className={[
        "flex items-center gap-1.5 rounded-t-nest-md px-3 py-2 text-[13px] font-medium transition-colors",
        active
          ? "bg-nest-background text-nest-primary"
          : "text-nest-muted hover:bg-nest-primary/5 hover:text-nest-foreground",
      ].join(" ")}
    >
      <Icon className="size-3.5" />
      {label}
    </button>
  );
}

function EmptyState({ title, hint }: { title: string; hint?: string }) {
  return (
    <div className="flex min-h-[200px] flex-col items-center justify-center gap-1 text-center text-nest-muted">
      <p className="text-sm">{title}</p>
      {hint ? <p className="text-xs text-nest-muted/70">{hint}</p> : null}
    </div>
  );
}

function ComponentPreview({ name }: { name: string }) {
  const Demo = useMemo(() => resolveDemoComponent(DEMOS[name]), [name]);
  if (!Demo) {
    return <EmptyState title="No live demo yet" hint="See the Documentation and Code tabs." />;
  }
  return (
    <div className="overflow-hidden rounded-nest-lg border border-nest-border bg-nest-surface/40">
      <Demo />
    </div>
  );
}

function ComponentDocs({ name }: { name: string }) {
  const markdown = DOCS[name];
  if (!markdown) {
    return <EmptyState title="Documentation coming soon" />;
  }
  return (
    <div className={MARKDOWN_CLASSES}>
      <ReactMarkdown remarkPlugins={[remarkGfm]}>{markdown}</ReactMarkdown>
    </div>
  );
}

function ComponentCode({ name }: { name: string }) {
  const code = CODE[name];
  if (!code) {
    return <EmptyState title="Example coming soon" hint="This component has no demo file yet." />;
  }
  return (
    <div className="max-w-[900px]">
      <pre className="overflow-x-auto whitespace-pre rounded-nest-md border border-nest-border bg-nest-background p-4 font-mono text-[13px] text-nest-foreground">
        <code>{code}</code>
      </pre>
      <p className="mt-3 text-xs text-nest-muted">
        Demo source from <code>@nest/components</code> — import components from <code>@nest/components</code>.
      </p>
    </div>
  );
}
