import { CheckCircle, Palette } from "lucide-react";
import { themeLabel, type ThemeDefinition, type TypographyStyle } from "../lib/themes";

type ThemeDetailViewProps = {
  theme: ThemeDefinition;
  isActive: boolean;
  onApply: () => void;
};

/** Read-only token viewer for one theme, with a button to apply it live. */
export function ThemeDetailView({ theme, isActive, onApply }: ThemeDetailViewProps) {
  const colorEntries: { label: string; value?: string }[] = [
    { label: "Background", value: theme.colors.background },
    { label: "Foreground", value: theme.colors.foreground },
    { label: "Primary", value: theme.colors.primary },
    { label: "Secondary", value: theme.colors.secondary },
    { label: "Surface", value: theme.colors.surface },
    { label: "Border", value: theme.colors.border },
    { label: "Accent", value: theme.colors.accent },
    { label: "Muted", value: theme.colors.muted },
  ];

  const statusEntries: { label: string; value: string }[] = [
    { label: "Success", value: theme.status.success },
    { label: "Warning", value: theme.status.warning },
    { label: "Error", value: theme.status.error },
    { label: "Info", value: theme.status.info },
  ];

  const typographyEntries: { label: string; style?: TypographyStyle }[] = [
    { label: "Body", style: theme.typography.body },
    { label: "Heading", style: theme.typography.heading },
    { label: "Caption", style: theme.typography.caption },
    { label: "Mono", style: theme.typography.mono },
  ];

  const spacingEntries = Object.entries(theme.spacing).filter(
    (entry): entry is [string, number] => entry[1] !== undefined,
  );
  const maxSpacing = Math.max(...spacingEntries.map(([, value]) => value));

  const radiusEntries = Object.entries(theme.radius).filter(
    (entry): entry is [string, number] => entry[1] !== undefined,
  );

  return (
    <div className="h-full min-h-0 overflow-auto bg-nest-background">
      <div className="mx-auto max-w-3xl space-y-8 px-6 py-6">
        <header className="flex items-center gap-3">
          <Palette className="size-6 text-nest-primary" />
          <div className="min-w-0 flex-1">
            <h1 className="text-lg font-semibold text-nest-foreground">{themeLabel(theme.id)}</h1>
            <p className="text-xs text-nest-muted">
              <code className="text-nest-foreground">{theme.id}</code> ·{" "}
              <span className="capitalize">{theme.mode}</span> mode
            </p>
          </div>
          {isActive ? (
            <span className="flex shrink-0 items-center gap-1.5 rounded-full bg-nest-success/15 px-2.5 py-1 text-xs font-medium text-nest-success">
              <CheckCircle className="size-3.5" />
              Active
            </span>
          ) : (
            <button
              type="button"
              onClick={onApply}
              className="shrink-0 rounded-nest-md bg-nest-primary px-3 py-1.5 text-xs font-medium text-white transition-colors hover:bg-nest-primary/90"
            >
              Apply Theme
            </button>
          )}
        </header>

        <section>
          <h2 className="mb-3 text-xs font-semibold uppercase tracking-wide text-nest-muted">Colors</h2>
          <div className="grid grid-cols-2 gap-3 sm:grid-cols-4">
            {colorEntries
              .filter((entry): entry is { label: string; value: string } => Boolean(entry.value))
              .map((entry) => (
                <ColorSwatch key={entry.label} label={entry.label} value={entry.value} />
              ))}
          </div>
        </section>

        <section>
          <h2 className="mb-3 text-xs font-semibold uppercase tracking-wide text-nest-muted">Status Colors</h2>
          <div className="grid grid-cols-2 gap-3 sm:grid-cols-4">
            {statusEntries.map((entry) => (
              <ColorSwatch key={entry.label} label={entry.label} value={entry.value} />
            ))}
          </div>
        </section>

        <section>
          <h2 className="mb-3 text-xs font-semibold uppercase tracking-wide text-nest-muted">Typography</h2>
          <div className="space-y-3">
            {typographyEntries
              .filter((entry): entry is { label: string; style: TypographyStyle } => Boolean(entry.style))
              .map((entry) => (
                <div key={entry.label} className="rounded-nest-md border border-nest-border p-3">
                  <div className="mb-1 flex items-baseline justify-between gap-2">
                    <span className="text-xs font-semibold text-nest-foreground">{entry.label}</span>
                    <span className="truncate font-mono text-[11px] text-nest-muted">
                      {entry.style.font_family} · {entry.style.size}px / {entry.style.line_height}px ·{" "}
                      {entry.style.weight}
                    </span>
                  </div>
                  <p
                    className="truncate text-nest-foreground"
                    style={{
                      fontFamily: entry.style.font_family,
                      fontSize: `${Math.min(entry.style.size, 28)}px`,
                      lineHeight: `${entry.style.line_height}px`,
                      fontWeight: entry.style.weight,
                    }}
                  >
                    The quick brown fox jumps over the lazy dog
                  </p>
                </div>
              ))}
          </div>
        </section>

        <section>
          <h2 className="mb-3 text-xs font-semibold uppercase tracking-wide text-nest-muted">Spacing</h2>
          <div className="space-y-1.5">
            {spacingEntries.map(([key, value]) => (
              <div key={key} className="flex items-center gap-3">
                <span className="w-10 shrink-0 font-mono text-xs text-nest-muted">{key}</span>
                <div
                  className="h-3 rounded-nest-sm bg-nest-primary/60"
                  style={{ width: `${Math.max((value / maxSpacing) * 200, 4)}px` }}
                />
                <span className="font-mono text-xs text-nest-muted">{value}px</span>
              </div>
            ))}
          </div>
        </section>

        <section>
          <h2 className="mb-3 text-xs font-semibold uppercase tracking-wide text-nest-muted">Radius</h2>
          <div className="flex flex-wrap items-end gap-4">
            {radiusEntries.map(([key, value]) => (
              <div key={key} className="flex flex-col items-center gap-1.5">
                <div
                  className="size-12 border border-nest-border bg-nest-surface"
                  style={{ borderRadius: `${Math.min(value, 24)}px` }}
                />
                <span className="font-mono text-[11px] text-nest-muted">
                  {key}: {value}px
                </span>
              </div>
            ))}
          </div>
        </section>
      </div>
    </div>
  );
}

function ColorSwatch({ label, value }: { label: string; value: string }) {
  return (
    <div className="overflow-hidden rounded-nest-md border border-nest-border">
      <div className="h-12 w-full" style={{ backgroundColor: value }} />
      <div className="px-2 py-1.5">
        <p className="text-xs font-medium text-nest-foreground">{label}</p>
        <p className="font-mono text-[11px] text-nest-muted">{value}</p>
      </div>
    </div>
  );
}
