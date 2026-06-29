# Agents

Kiwi's agent system spawns any configured binary as a PTY process and renders its output in the **Agent tab**. The binary receives stdin from the keyboard and writes to stdout; Kiwi infers live status from keyword patterns in the output.

See [SPEC-010 Agent Service](../specs/SPEC-010-agent-service.md) for the full PTY contract, [PTY pipeline](./pty-pipeline.md) for the end-to-end streaming architecture (required reading for tool authors), and [ADR-017](../architecture/adr/ADR-017-multi-agent-future-design.md) for multi-agent plans.

## Available agents

| Agent | Description | Guide |
|---|---|---|
| `agent` | Cursor Agent CLI | — |
| `claude` | Anthropic Claude Code CLI | — |
| `kiwi-ollama` | Ollama/qwen2.5-coder local LLM with RAG codebase indexing | [ollama-qwen-agent.md](./ollama-qwen-agent.md) |

## MCP memory servers

Two MCP servers provide persistent memory over PostgreSQL + pgvector:

| Server | Binary | Purpose |
|---|---|---|
| Project memory | `kiwi-mcp-memory` | Index and search project docs | 
| Context memory | `kiwi-mcp-context` | Save and retrieve agent session notes |

See [mcp-memory-servers.md](./mcp-memory-servers.md) for setup and configuration.

## Adding a custom agent

Any executable can be used. Minimal `.kiwi.toml`:

```toml
[agent]
command = "my-agent"
args = ["--flag"]
[agent.env]
MY_KEY = "value"
```

Kiwi infers agent status from these output keywords (case-insensitive):

| Status | Trigger words |
|---|---|
| Thinking | `thinking`, `planning`, `reasoning` |
| Executing | `running tool`, `tool call`, `grep ` |
| Success | `completed`, `finished`, `success` |
| Error | `error:`, `failed`, `panic:` |
| Warning | `warning:`, `deprecated` |
