import { kiwiInvoke } from "./ipc";

export type McpAgentOptions = {
  mcpServers: string[];
  disabledMcpServers: string[];
  maxSteps: number;
  agentMode: boolean;
  allowSaveContext: boolean;
  allowFileWrites: boolean;
};

export type McpToolInfo = {
  name: string;
  qualifiedName: string;
  description: string;
  inputSchema: Record<string, unknown>;
};

export type McpServerInfo = {
  name: string;
  configured: boolean;
  enabled: boolean;
  command: string;
  args: string[];
  cwd: string | null;
  env: Record<string, string>;
  tools: McpToolInfo[];
  error: string | null;
};

export type McpOverview = {
  mcpConfigPath: string;
  agent: McpAgentOptions;
  servers: McpServerInfo[];
};

/** Loads MCP servers, tools, and agent configuration. */
export async function mcpOverview(): Promise<McpOverview> {
  return kiwiInvoke<McpOverview>("mcp_overview");
}
