import { useState } from "react";
import { Icon } from "../shell";
import { faChevronDown } from "../lib/fontawesome";
import { PanelPlaceholder } from "./ActivityBar";
import { LogsPanel } from "./LogsPanel";
import { TerminalPanel } from "./TerminalPanel";
import { ToolCallLogPanel } from "./ToolCallLogPanel";
import { ToolOutputPanel } from "./ToolOutputPanel";
import { ProblemsPanel } from "./ProblemsPanel";

const BOTTOM_TABS = [
  "Terminal",
  "Problems",
  "Output",
  "Logs",
  "Tool Activity",
  "Debug",
] as const;

type BottomTab = (typeof BOTTOM_TABS)[number];

type BottomPanelProps = {
  /** Collapses the panel (toggle lives here so it flips with the panel). */
  onToggleCollapse?: () => void;
};

export function BottomPanel({ onToggleCollapse }: BottomPanelProps) {
  const [tab, setTab] = useState<BottomTab>("Terminal");

  return (
    <div className="flex h-full min-h-0 flex-col border-t border-nest-border bg-nest-background">
      <div className="flex h-8 shrink-0 items-stretch border-b border-nest-border bg-nest-surface text-[11px]">
        {BOTTOM_TABS.map((name) => (
          <button
            key={name}
            type="button"
            onClick={() => setTab(name)}
            className={[
              "border-r border-nest-border px-3 uppercase tracking-wide",
              tab === name
                ? "bg-nest-background font-medium text-nest-foreground"
                : "text-nest-muted hover:text-nest-foreground",
            ].join(" ")}
          >
            {name}
          </button>
        ))}
        {onToggleCollapse ? (
          <button
            type="button"
            onClick={onToggleCollapse}
            title="Hide panel"
            aria-label="Hide panel"
            className="ml-auto flex w-8 items-center justify-center text-nest-muted hover:bg-nest-muted/10 hover:text-nest-foreground"
          >
            <Icon icon={faChevronDown} className="size-3" />
          </button>
        ) : null}
      </div>
      <div className="min-h-0 flex-1">
        {/* Keep the terminal mounted across tab switches so sessions/scrollback
            survive; other tabs are lightweight placeholders. */}
        <div className={tab === "Terminal" ? "h-full" : "hidden"}>
          <TerminalPanel />
        </div>
        <div className={tab === "Problems" ? "h-full" : "hidden"}>
          <ProblemsPanel active={tab === "Problems"} />
        </div>
        <div className={tab === "Logs" ? "h-full" : "hidden"}>
          <LogsPanel active={tab === "Logs"} />
        </div>
        <div className={tab === "Output" ? "h-full" : "hidden"}>
          <ToolOutputPanel active={tab === "Output"} />
        </div>
        <div className={tab === "Tool Activity" ? "h-full" : "hidden"}>
          <ToolCallLogPanel active={tab === "Tool Activity"} />
        </div>
        {tab !== "Terminal" &&
        tab !== "Problems" &&
        tab !== "Logs" &&
        tab !== "Output" &&
        tab !== "Tool Activity" ? (
          <PanelPlaceholder title={tab} />
        ) : null}
      </div>
    </div>
  );
}
