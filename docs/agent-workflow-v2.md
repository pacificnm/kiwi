# Kiwi agent workflow v2

Improve agent reliability on weak local models (3B–7B) while waiting for
stronger GPU hardware. Cursor parity is the long-term target; this plan covers
what we can build now.

## Status

| Feature | Status | Notes |
| --- | --- | --- |
| Auto context: open editor tabs | **Done** | Injected as `<open-files>` on Send |
| Auto context: workspace rules | **Done** | `AGENTS.md` + `.cursor/rules/*.{mdc,md}` |
| Auto context: output path hint | **Done** | Doc/save tasks → `docs/agent/agent.md` hint |
| `search_code` tool | **Done** | Line grep across source extensions |
| `cargo_check` tool | **Done** | Linter/compile feedback for Rust |
| `search_files` (path) | Done (v1) | |
| Semantic / embedding search | Planned | After P40 + 30B; needs index + embed model |
| `cargo clippy` | Planned | Stricter lint pass |
| Post-edit auto-check | Planned | Optional host hook after write tools |

## Auto context injection

On **Agent Send**, Kiwi prepends workspace context to the user message:

```xml
<open-files>
<file path="src/agent/mod.rs" active="true">
…
</file>
</open-files>

<workspace-rules>
<rule file="AGENTS.md">…</rule>
</workspace-rules>

<task-hint>… docs/agent/agent.md …</task-hint>

---

(user prompt)
```

Limits: 6 open files, 8KB per file, 24KB total context.

Implementation: [`src/agent/context.rs`](../desktop/crates/kiwi/src/agent/context.rs)

## Agent tools (native `nest-file`)

| Tool | Purpose |
| --- | --- |
| `search_files` | Find paths by filename |
| `search_code` | Find lines by content (grep) |
| `read_file` | Read file |
| `write_file` / `update_file` | Create or edit |
| `cargo_check` | Compiler errors after Rust edits |

`cargo_check` is the v1 **linter feedback** loop for Rust. The model should call
it after editing `.rs` files.

## Model guidance

| Hardware | Suggested model | Agent use |
| --- | --- | --- |
| Current (CPU / weak GPU) | `qwen2.5-coder:7b` | Minimum for tool calling |
| Tesla P40 (24GB) | `qwen2.5:32b` or similar | Target after install |

## Next steps (v2.1)

1. **Semantic search** — embed workspace chunks (reuse nest-memory patterns or
   local Ollama embed model); expose as `search_semantic` tool.
2. **`cargo clippy`** — optional stricter lint tool.
3. **Auto-check after write** — Kiwi host optionally runs `cargo check` when
   agent writes `.rs` files and feeds result into the next step.
4. **Diagnostics panel** — surface last `cargo_check` output in bottom panel.

## Related

- [agent-mcp-v1.md](./agent-mcp-v1.md) — MCP loop v1
- [nest-agent README](../../../docs/nest-agent/README.md)
