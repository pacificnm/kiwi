import { useEffect, useState } from "react";
import { formatIpcError } from "../lib/agent";
import { faChevronLeft } from "../lib/fontawesome";
import { Icon, isTauri } from "../shell";
import { themeLabel, themesList, themeTabKey, type ThemeDefinition } from "../lib/themes";
import { useWorkbench } from "./state";

type ThemesPanelProps = {
  onToggleCollapse?: () => void;
};

/** Theme Activity sidebar — lists registered themes; clicking one applies it live. */
export function ThemesPanel({ onToggleCollapse }: ThemesPanelProps) {
  const { activePath, activeThemeId, openTheme } = useWorkbench();
  const [themes, setThemes] = useState<ThemeDefinition[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    if (!isTauri()) {
      setLoading(false);
      return;
    }

    let cancelled = false;

    async function loadThemes() {
      try {
        const list = await themesList();
        if (!cancelled) {
          setThemes(list);
        }
      } catch (loadError) {
        if (!cancelled) {
          setError(formatIpcError(loadError));
        }
      } finally {
        if (!cancelled) {
          setLoading(false);
        }
      }
    }

    void loadThemes();
    return () => {
      cancelled = true;
    };
  }, []);

  return (
    <div className="flex h-full min-h-0 flex-col bg-nest-background">
      <header className="flex h-9 shrink-0 items-center border-b border-nest-border px-3">
        <span className="text-xs font-semibold uppercase tracking-wide text-nest-muted">Theme</span>
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
        {!isTauri() ? (
          <p className="px-3 py-2 text-xs text-nest-muted">Theme is available in the desktop app.</p>
        ) : error ? (
          <p className="px-3 py-2 text-xs text-nest-error">{error}</p>
        ) : loading ? (
          <p className="px-3 py-4 text-xs text-nest-muted">Loading…</p>
        ) : themes.length === 0 ? (
          <p className="px-3 py-4 text-xs text-nest-muted">No themes registered.</p>
        ) : (
          <ul>
            {themes.map((theme) => {
              const selected = activePath === themeTabKey(theme.id);
              const isActive = activeThemeId === theme.id;
              return (
                <li key={theme.id}>
                  <button
                    type="button"
                    onClick={() => openTheme(theme)}
                    title={theme.id}
                    className={[
                      "flex w-full items-center gap-2 px-3 py-1.5 text-left text-xs transition-colors",
                      selected
                        ? "bg-nest-accent/15 text-nest-foreground"
                        : "text-nest-muted hover:bg-nest-muted/10 hover:text-nest-foreground",
                    ].join(" ")}
                  >
                    <span
                      className="size-3 shrink-0 rounded-full border border-nest-border"
                      style={{ backgroundColor: theme.colors.primary }}
                      aria-hidden
                    />
                    <span className="min-w-0 flex-1 truncate">{themeLabel(theme.id)}</span>
                    {isActive ? <span className="shrink-0 text-[10px] text-nest-success">●</span> : null}
                  </button>
                </li>
              );
            })}
          </ul>
        )}
      </div>
    </div>
  );
}
