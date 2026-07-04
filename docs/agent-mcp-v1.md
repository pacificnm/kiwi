# Kiwi agent + MCP v1 Implementation Plan

## Status: In progress

Canonical copy also lives in the Nest monorepo at
[`docs/plan/nest-agent-mcp-v1.md`](../../../docs/plan/nest-agent-mcp-v1.md).

Enable Kiwi to run a **tool-using agent loop** against local Ollama models, with tools
supplied by **MCP servers** — starting with the Nest memory servers used by Cursor.

## Kiwi paths

| Item | Path |
| --- | --- |
| Desktop app | `apps/kiwi/desktop/` |
| Config | `apps/kiwi/desktop/config.toml` |
| MCP config (reuse Cursor) | Nest repo `.cursor/mcp.json` |
| MCP setup | Nest repo [`tools/MCP-SETUP.md`](../../../tools/MCP-SETUP.md) |
| This plan | `apps/kiwi/docs/agent-mcp-v1.md` |

Suggested `[agent]` config (Phase 4):

```toml
[agent]
host = "192.168.88.10"
port = 11434
model = "qwen2.5:7b"
mcp_config = "../../../.cursor/mcp.json"  # relative to config.toml
mcp_servers = ["nest-memory", "nest-knowledge", "nest-context-memory"]
max_steps = 10
```

## Architecture

```text
Kiwi workbench / CLI
        ↓
  nest-agent (loop)  ←→  nest-ai / nest-ai-ollama
        ↓
    nest-mcp (stdio client)
        ↓
  Python MCP servers (nest-memory, nest-knowledge, nest-context-memory)
```

**Transport:** FastMCP uses **newline-delimited JSON** on stdio (not Content-Length).

## Phases

| Phase | Scope | Status |
| --- | --- | --- |
| 0 | Ollama tool-calling spike | **Done** |
| 1 | `nest-mcp` stdio client | **Done (v0.1)** |
| 2 | `nest-ai` tool types + Ollama tools API | **Done (v0.1)** |
| 3 | `nest-agent` loop | **Done (v0.1)** |
| 4 | Kiwi CLI `kiwi agent` → GUI tool UI | **Done (v0.1)** |
| 5 | Hardening | **Done (v1)** |

## Phase 5 (v1)

- Agent replies **stream** via Ollama `stream_complete`
- **Parallel** MCP tool calls when the model returns multiple tools
- MCP **reconnect** on server crash / stdout close
- Agent sidebar: **enable/disable** MCP servers (`disabled_mcp_servers`)
- Optional **`save_context_memory`** auto-run (`allow_save_context`)
- Attached-file summarize; markdown fenced tool-call parsing

## Phase 1 deliverable

```bash
cargo test -p nest-mcp
```

Spawn MCP servers from `.cursor/mcp.json`, `tools/list`, `tools/call`.

## Model guidance

| Model | Tool calling | Notes |
| --- | --- | --- |
| `qwen2.5-coder:3b` | Weak | OK for chat, not agent QA |
| `qwen2.5:7b` / `llama3.1:8b` | Good | Agent MVP minimum |

## Related

- [Nest architecture](../../../docs/architecture.md)
- [nest-ai](../../../docs/nest-ai/README.md)
- [MCP-SETUP.md](../../../tools/MCP-SETUP.md)
