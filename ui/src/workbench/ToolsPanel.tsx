import { useCallback, useEffect, useMemo, useState } from "react";
import { formatIpcError } from "../lib/agent";
import {
  faChevronDown,
  faChevronLeft,
  faChevronRight,
  faRotateRight,
} from "../lib/fontawesome";
import { mcpOverview, type McpOverview, type McpServerInfo, type McpToolInfo } from "../lib/mcp";
import { Icon, isTauri, useToast } from "../shell";

type ToolsPanelProps = {
  onToggleCollapse?: () => void;
};

/** Tools activity sidebar — MCP servers, tools, and agent configuration. */
export function ToolsPanel({ onToggleCollapse }: ToolsPanelProps) {
  const toast = useToast();
  const [overview, setOverview] = useState<McpOverview | null>(null);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [selectedServer, setSelectedServer] = useState<string | null>(null);
  const [expandedTool, setExpandedTool] = useState<string | null>(null);

  const refresh = useCallback(async () => {
    if (!isTauri()) {
      return;
    }
    setBusy(true);
    try {
      const next = await mcpOverview();
      setOverview(next);
      setError(null);
      setSelectedServer((current) => {
        if (current && next.servers.some((server) => server.name === current)) {
          return current;
        }
        return next.servers[0]?.name ?? null;
      });
    } catch (caught) {
      const message = formatIpcError(caught);
      setError(message);
      toast.error(message);
    } finally {
      setBusy(false);
    }
  }, [toast]);

  useEffect(() => {
    if (isTauri()) {
      void refresh();
    }
  }, [refresh]);

  const server = useMemo(
    () => overview?.servers.find((entry) => entry.name === selectedServer) ?? null,
    [overview, selectedServer],
  );

  if (!isTauri()) {
    return (
      <PanelFrame onToggleCollapse={onToggleCollapse}>
        <p className="px-3 py-2 text-xs text-nest-muted">MCP tools are available in the desktop app.</p>
      </PanelFrame>
    );
  }

  return (
    <PanelFrame onRefresh={() => void refresh()} onToggleCollapse={onToggleCollapse} busy={busy}>
      {error ? <p className="px-3 py-2 text-xs text-nest-error">{error}</p> : null}

      {overview ? (
        <div className="space-y-0">
          <AgentOptionsSection options={overview.agent} configPath={overview.mcpConfigPath} />

          <Section title="MCP servers" count={overview.servers.length}>
            <ul>
              {overview.servers.map((entry) => (
                <li key={entry.name}>
                  <button
                    type="button"
                    onClick={() => {
                      setSelectedServer(entry.name);
                      setExpandedTool(null);
                    }}
                    className={[
                      "flex w-full items-center gap-2 px-3 py-2 text-left text-xs",
                      selectedServer === entry.name
                        ? "bg-nest-accent/15 text-nest-foreground"
                        : "hover:bg-nest-muted/10",
                    ].join(" ")}
                  >
                    <StatusDot server={entry} />
                    <span className="min-w-0 flex-1 truncate font-medium">{entry.name}</span>
                    <span className="shrink-0 text-[10px] text-nest-muted">{entry.tools.length}</span>
                  </button>
                </li>
              ))}
            </ul>
          </Section>

          {server ? (
            <ServerSection
              server={server}
              expandedTool={expandedTool}
              onToggleTool={(name) =>
                setExpandedTool((current) => (current === name ? null : name))
              }
            />
          ) : null}
        </div>
      ) : busy ? (
        <p className="px-3 py-4 text-xs text-nest-muted">Loading MCP tools…</p>
      ) : (
        <p className="px-3 py-4 text-xs text-nest-muted">No MCP data loaded.</p>
      )}
    </PanelFrame>
  );
}

function PanelFrame({
  children,
  onRefresh,
  onToggleCollapse,
  busy,
}: {
  children: React.ReactNode;
  onRefresh?: () => void;
  onToggleCollapse?: () => void;
  busy?: boolean;
}) {
  return (
    <div className="flex h-full min-h-0 flex-col bg-nest-background">
      <header className="flex h-9 shrink-0 items-center gap-1 border-b border-nest-border px-3">
        <span className="truncate text-xs font-semibold uppercase tracking-wide text-nest-muted">
          Tools
        </span>
        <div className="ml-auto flex items-center gap-0.5">
          {onRefresh ? (
            <button
              type="button"
              onClick={onRefresh}
              title="Refresh"
              aria-label="Refresh"
              className="flex size-6 items-center justify-center rounded-nest-sm text-nest-muted hover:bg-nest-muted/10 hover:text-nest-foreground"
            >
              <Icon
                icon={faRotateRight}
                className={["size-3", busy ? "animate-spin" : ""].join(" ")}
              />
            </button>
          ) : null}
          {onToggleCollapse ? (
            <button
              type="button"
              onClick={onToggleCollapse}
              title="Hide sidebar"
              aria-label="Hide sidebar"
              className="flex size-6 items-center justify-center rounded-nest-sm text-nest-muted hover:bg-nest-muted/10 hover:text-nest-foreground"
            >
              <Icon icon={faChevronLeft} className="size-3" />
            </button>
          ) : null}
        </div>
      </header>
      <div className="min-h-0 flex-1 overflow-auto">{children}</div>
    </div>
  );
}

function AgentOptionsSection({
  options,
  configPath,
}: {
  options: McpOverview["agent"];
  configPath: string;
}) {
  return (
    <div className="border-b border-nest-border px-3 py-2 text-[10px] text-nest-muted">
      <p className="mb-1 font-semibold uppercase tracking-wide">Configuration</p>
      <p className="truncate font-mono text-nest-foreground/80" title={configPath}>
        {configPath}
      </p>
      <p className="mt-1 font-mono text-nest-foreground/80">
        servers: {options.mcpServers.join(", ")}
      </p>
      <p className="mt-2 text-[10px] leading-relaxed text-nest-muted">
        Probes Kiwi&apos;s MCP client (Tools sidebar). External agents use runtime-specific
        config: Claude reads <span className="font-mono">.mcp.json</span>; OpenCode uses{" "}
        <span className="font-mono">opencode.json</span>
        instead.
      </p>
      <div className="mt-1 flex flex-wrap gap-1">
        <Chip label={`max_steps=${options.maxSteps}`} />
        {options.agentMode ? <Chip label="agent_mode" /> : null}
        {options.allowSaveContext ? <Chip label="allow_save_context" /> : null}
        {options.allowFileWrites ? <Chip label="allow_file_writes" /> : null}
        {options.disabledMcpServers.map((name) => (
          <Chip key={name} label={`disabled: ${name}`} />
        ))}
      </div>
    </div>
  );
}

function Chip({ label }: { label: string }) {
  return (
    <span className="rounded-full bg-nest-muted/15 px-2 py-0.5 font-mono text-[10px] text-nest-foreground/80">
      {label}
    </span>
  );
}

function Section({
  title,
  count,
  children,
}: {
  title: string;
  count: number;
  children: React.ReactNode;
}) {
  return (
    <div className="border-b border-nest-border">
      <div className="flex items-center gap-1.5 px-3 py-1.5 text-[11px] font-semibold uppercase tracking-wide text-nest-muted">
        <span>{title}</span>
        <span className="ml-auto rounded-full bg-nest-muted/20 px-1.5 text-[10px] font-medium">
          {count}
        </span>
      </div>
      {children}
    </div>
  );
}

function ServerSection({
  server,
  expandedTool,
  onToggleTool,
}: {
  server: McpServerInfo;
  expandedTool: string | null;
  onToggleTool: (name: string) => void;
}) {
  return (
    <div className="border-b border-nest-border">
      <div className="px-3 py-2">
        <div className="flex items-center gap-2">
          <StatusDot server={server} />
          <h3 className="text-xs font-semibold text-nest-foreground">{server.name}</h3>
          <span className="text-[10px] text-nest-muted">
            {server.enabled ? "enabled" : server.configured ? "disabled" : "not configured"}
          </span>
        </div>
        {server.command ? (
          <p className="mt-1 break-all font-mono text-[10px] text-nest-muted">{server.command}</p>
        ) : null}
        {server.args.length > 0 ? (
          <p className="mt-0.5 break-all font-mono text-[10px] text-nest-muted">
            {server.args.join(" ")}
          </p>
        ) : null}
        {server.cwd ? (
          <p className="mt-0.5 break-all font-mono text-[10px] text-nest-muted">cwd: {server.cwd}</p>
        ) : null}
        {server.error ? <p className="mt-1 text-[11px] text-nest-error">{server.error}</p> : null}
      </div>

      <div className="pb-2">
        <p className="px-3 py-1 text-[10px] font-semibold uppercase tracking-wide text-nest-muted">
          Tools ({server.tools.length})
        </p>
        {server.tools.length === 0 ? (
          <p className="px-3 py-1 text-[11px] text-nest-muted">No tools loaded.</p>
        ) : (
          <ul>
            {server.tools.map((tool) => (
              <ToolRow
                key={tool.qualifiedName}
                tool={tool}
                expanded={expandedTool === tool.qualifiedName}
                onToggle={() => onToggleTool(tool.qualifiedName)}
              />
            ))}
          </ul>
        )}
      </div>
    </div>
  );
}

function StatusDot({ server }: { server: McpServerInfo }) {
  const color = server.error
    ? "bg-nest-error"
    : server.enabled
      ? "bg-nest-success"
      : server.configured
        ? "bg-nest-warning"
        : "bg-nest-muted";
  return <span className={["size-2 shrink-0 rounded-full", color].join(" ")} />;
}

function ToolRow({
  tool,
  expanded,
  onToggle,
}: {
  tool: McpToolInfo;
  expanded: boolean;
  onToggle: () => void;
}) {
  return (
    <li className="border-t border-nest-border/50">
      <button
        type="button"
        onClick={onToggle}
        className="flex w-full items-start gap-2 px-3 py-1.5 text-left hover:bg-nest-muted/5"
      >
        <Icon
          icon={expanded ? faChevronDown : faChevronRight}
          className="mt-0.5 size-3 shrink-0 text-nest-muted"
        />
        <span className="min-w-0 flex-1">
          <span className="block font-mono text-xs text-nest-foreground">{tool.name}</span>
          {tool.description ? (
            <span className="mt-0.5 block text-[10px] leading-snug text-nest-muted">
              {tool.description}
            </span>
          ) : null}
        </span>
      </button>
      {expanded ? (
        <pre className="overflow-x-auto border-t border-nest-border/50 bg-nest-surface/40 px-3 py-2 font-mono text-[10px] text-nest-foreground/90">
          {JSON.stringify(tool.inputSchema, null, 2)}
        </pre>
      ) : null}
    </li>
  );
}
