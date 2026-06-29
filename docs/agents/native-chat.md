# Native Chat Agent Integration

How Kiwi streams directly to an LLM API and renders results in the native egui chat panel.
This is the primary integration path for new agents (see ADR-024).

For the legacy PTY subprocess path, see [pty-pipeline.md](./pty-pipeline.md).

Related contracts:

- [SPEC-010 Agent Service](../specs/SPEC-010-agent-service.md) — full requirements
- [ADR-024 Native Chat Architecture](../architecture/adr/ADR-024-native-chat-agent-architecture.md) — design rationale
- [ADR-017 Multi-agent](../architecture/adr/ADR-017-multi-agent-future-design.md) — session model

## Mental model

In API mode Kiwi owns the LLM conversation directly — no subprocess, no PTY:

```text
┌─────────────────────┐   HTTP/SSE stream    ┌──────────────────────┐
│ LLM Provider API    │ ──────────────────── │ streaming task       │ (background)
│ (Claude, etc.)      │                      └──────────┬───────────┘
└─────────────────────┘                                 │ AppEvent::AgentTokenChunk
                                                        │ AppEvent::AgentToolCallStart
                                                        │ AppEvent::AgentStreamDone
                                                        ▼
                                               ┌──────────────────────┐
                                               │ reducer/agent.rs     │
                                               │ → ChatSession state  │
                                               └──────────┬───────────┘
                                                          │
                                                          ▼
                                               ┌──────────────────────┐
                                               │ chat.rs panel        │ egui (GUI only)
                                               │ message list         │
                                               │ tool widgets         │
                                               │ text input           │
                                               └──────────────────────┘
```

The TUI frontend (`crates/kiwi`) still uses the PTY path and is unaffected by this architecture.

## Activation

Set `mode = "api"` in your config or plugin manifest:

```toml
# ~/.config/kiwi/config.toml  (user) or .kiwi.toml (repo)
[agent]
mode = "api"
provider = "claude"
api_key_env = "ANTHROPIC_API_KEY"   # env var holding the key; default
model = "claude-opus-4-8"
```

`AppState::from_startup` checks `config.agent.mode` and initialises
`AgentState { chat: Some(ChatSession { model, .. }) }` when `AgentMode::Api`.

The agent panel (`agent.rs`) checks `chat.is_some()` and delegates to `chat.rs` — no other code
path change required.

## Plugin manifest

```toml
# plugins/my_agent/plugin.toml
[plugin]
name = "my-agent"
display_name = "My Agent"
description = "Claude-powered assistant"

[agent]
mode = "api"
provider = "claude"
model = "claude-opus-4-8"
```

Settings → Agents lists plugins that declare an `[agent]` section. Applying a plugin persists
`mode`, `provider`, and `model` via `persist_user_agent_mode` without touching other config.

## Lifecycle

| Phase | Trigger | Core action |
|-------|---------|-------------|
| Initialise | `AppState::from_startup` with `AgentMode::Api` | `AgentState { chat: Some(ChatSession) }` |
| User sends | `AgentUserSend { agent_id, text }` | Append `ChatMessage(User)`, set `is_streaming`, emit `AgentEffect::StreamRequest` |
| Token arrives | `AgentTokenChunk { agent_id, text }` | Append to `ChatSession::streaming_text` |
| Tool call | `AgentToolCallStart { tool_use_id, tool_name, input_json }` | Append `ContentBlock::ToolUse`, emit `AgentEffect::ExecuteTool` |
| Tool result | `AgentToolResult { tool_use_id, output, is_error }` | Append `ContentBlock::ToolResult` |
| Stream done | `AgentStreamDone { agent_id }` | Finalise assistant message, clear streaming state |
| Error | `AgentStreamError { agent_id, message }` | Set `AgentStatus::Error`, show error banner |
| Cancel | `AgentCancelStream` command | `StreamCancelHandle::cancel()` via `PtyRuntime::cancel_stream` |
| Clear | `AgentClearChat(agent_id)` | Reset `ChatSession::messages` |

## State slices

### `ChatSession`

| Field | Role |
|-------|------|
| `messages: Vec<ChatMessage>` | Full conversation history |
| `streaming_text: String` | Accumulates the in-flight assistant turn |
| `active_tool_call: Option<ToolUse>` | Tool being executed |
| `input_draft: String` | Current text box content |
| `is_streaming: bool` | Locks input, shows cursor |
| `follow_tail: bool` | Auto-scroll during streaming |
| `status: AgentStatus` | Drives status bar |
| `model: String` | Shown in panel chrome |
| `provider: String` | Used by streaming task |

### `ContentBlock`

```rust
pub enum ContentBlock {
    Text(String),
    ToolUse(ToolUse),     // { id, name, input_json }
    ToolResult(ToolResult), // { tool_use_id, output, is_error }
}
```

### `AgentManager`

Same as PTY mode — tracks up to 3 sessions, active index, cached status label.

## Chat panel UI (`dock/panels/chat.rs`)

| Section | Behaviour |
|---------|-----------|
| Chrome | Status badge (Thinking/Executing/…), model label, session tabs, Clear button |
| Error banner | Shown when `AgentStatus::Error`; red-filled Frame |
| Message list | `ScrollArea` with `stick_to_bottom(is_streaming && follow_tail)` |
| User message | Right-aligned Frame bubble |
| Assistant message | Plain `ui.label()` rows |
| Streaming cursor | `▋` appended to partial turn |
| Tool widget | Collapsible section with arrow; pretty-printed JSON; `→ Terminal` button |
| Input box | Multiline `TextEdit` disabled while streaming; Ctrl+Enter sends; Escape clears |

The `→ Terminal` button dispatches `NavCommand::SetFocus(FocusTarget::Shell)` so the user can
paste tool output back.

## Stream cancellation

`PtyRuntime` holds a `StreamCancelHandle` per agent (in `AgentRuntime::stream_cancels`).
`AgentEffect::StreamRequest` registers a new handle, automatically cancelling any prior stream
for the same agent. `AgentEffect::CancelStream` explicitly cancels without starting a new one.

On shutdown, `AgentRuntime::cancel_all_streams` fires all handles.

## Building an API-mode plugin

1. Add `mode = "api"`, `provider`, and `model` to `plugin.toml`.
2. Ensure `ANTHROPIC_API_KEY` (or equivalent) is set in the environment or via `api_key_env`.
3. Register in Settings → Agents — the Apply button calls `persist_user_agent_mode`.
4. No agent binary required; Kiwi owns the HTTP call.

See `plugins/kiwi_agent_claude/` for the reference implementation.

## What the PTY path still does

The PTY path handles `AgentState { chat: None }` agents:

- `agent.rs` falls through to `render_agent_chrome` / `render_agent_panel` (PTY scrollback)
- `AgentEffect::Spawn` / `Restart` / `Write` are handled in `services.rs`
- The TUI (`crates/kiwi`) uses this path exclusively

PTY mode is not deprecated for the TUI or for opaque CLI agents. GUI-only PTY support will be
sunsetted once API-mode parity is confirmed (tracked in follow-up issues).

## Key source files

| Area | Path |
|------|------|
| Chat state types | `crates/kiwi_core/src/agent/chat.rs` |
| Stream events | `crates/kiwi_core/src/agent/stream_event.rs` |
| Reducer — chat | `crates/kiwi_core/src/reducer/agent.rs` (lines 200+) |
| Chat panel | `crates/kiwi_gui/src/dock/panels/chat.rs` |
| Agent panel (dispatch) | `crates/kiwi_gui/src/dock/panels/agent.rs` |
| Stream cancel registry | `crates/kiwi_gui/src/pty/agent_runtime.rs` |
| Config types | `crates/kiwi_core/src/config/types.rs` (`AgentMode`, `AgentSettings`) |
| Config writer | `crates/kiwi_core/src/config/writer.rs` (`persist_user_agent_mode`) |
| Plugin manifest | `crates/kiwi_plugin_api/src/manifest.rs` |
| Reference plugin | `plugins/kiwi_agent_claude/` |

## Verification

```bash
cargo check --workspace
cargo test --workspace
# Manual: open Agent tab with mode="api" configured; send a message; confirm streaming cursor
# and message history; trigger a tool call; confirm widget renders.
```
