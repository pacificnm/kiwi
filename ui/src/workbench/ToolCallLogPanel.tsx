/** Bottom panel — log of MCP tool calls during agent runs. */
export function ToolCallLogPanel({ active }: { active: boolean }) {
  if (!active) {
    return null;
  }
  return (
    <div className="flex h-full flex-col px-3 py-4 text-xs text-nest-muted">
      <p>MCP tool activity will be logged here during agent runs.</p>
      <p className="mt-1 opacity-70">
        Each tool call, arguments, and result summary will show in this stream.
      </p>
    </div>
  );
}
