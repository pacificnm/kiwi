import { useEffect, useState } from "react";
import { formatIpcError } from "../lib/agent";
import { docsList, docTabKey, type DocEntry } from "../lib/docs";
import { faChevronLeft } from "../lib/fontawesome";
import { Icon, isTauri } from "../shell";
import { useWorkbench } from "./state";

type HelpPanelProps = {
  onToggleCollapse?: () => void;
};

/** Help Activity sidebar — lists Kiwi's own `docs/`; opens each as an editor tab. */
export function HelpPanel({ onToggleCollapse }: HelpPanelProps) {
  const { activePath, openDoc } = useWorkbench();
  const [entries, setEntries] = useState<DocEntry[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    if (!isTauri()) {
      setLoading(false);
      return;
    }

    let cancelled = false;

    async function loadIndex() {
      try {
        const list = await docsList();
        if (!cancelled) {
          setEntries(list);
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

    void loadIndex();
    return () => {
      cancelled = true;
    };
  }, []);

  if (!isTauri()) {
    return (
      <PanelFrame onToggleCollapse={onToggleCollapse}>
        <p className="px-3 py-2 text-xs text-nest-muted">Help is available in the desktop app.</p>
      </PanelFrame>
    );
  }

  return (
    <PanelFrame onToggleCollapse={onToggleCollapse}>
      {error ? <p className="px-3 py-2 text-xs text-nest-error">{error}</p> : null}
      {loading ? (
        <p className="px-3 py-4 text-xs text-nest-muted">Loading…</p>
      ) : entries.length === 0 ? (
        <p className="px-3 py-4 text-xs text-nest-muted">No documentation found.</p>
      ) : (
        <ul>
          {entries.map((entry) => {
            const selected = activePath === docTabKey(entry.path);
            return (
              <li key={entry.path}>
                <button
                  type="button"
                  onClick={() => openDoc(entry)}
                  title={entry.path}
                  style={{ paddingLeft: `${12 + entry.depth * 14}px` }}
                  className={[
                    "flex w-full items-center gap-1.5 py-1.5 pr-3 text-left text-xs transition-colors",
                    selected
                      ? "bg-nest-accent/15 text-nest-foreground"
                      : "text-nest-muted hover:bg-nest-muted/10 hover:text-nest-foreground",
                  ].join(" ")}
                >
                  <span className="min-w-0 flex-1 truncate">{entry.name}</span>
                </button>
              </li>
            );
          })}
        </ul>
      )}
    </PanelFrame>
  );
}

function PanelFrame({
  children,
  onToggleCollapse,
}: {
  children: React.ReactNode;
  onToggleCollapse?: () => void;
}) {
  return (
    <div className="flex h-full min-h-0 flex-col bg-nest-background">
      <header className="flex h-9 shrink-0 items-center border-b border-nest-border px-3">
        <span className="text-xs font-semibold uppercase tracking-wide text-nest-muted">Help</span>
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
      <div className="min-h-0 flex-1 overflow-auto">{children}</div>
    </div>
  );
}
