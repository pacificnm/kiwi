/** Bottom panel — streamed output from the active tool invocation. */
export function ToolOutputPanel({ active }: { active: boolean }) {
  if (!active) {
    return null;
  }
  return (
    <div className="flex h-full flex-col px-3 py-4 text-xs text-nest-muted">
      <p>Tool output from agent runs will appear here.</p>
      <p className="mt-1 opacity-70">Run an agent query with MCP tools to populate this panel.</p>
    </div>
  );
}
