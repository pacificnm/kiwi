# SPEC-010: Agent Service

## Purpose

Manage AI agent sessions in the Agent main tab.  Two execution paths are supported:

- **API mode** (`mode = "api"`) — direct LLM streaming over HTTP/SSE with a native egui chat panel. This is the primary path for new agents (Phase 4–6, issues #329–#334).
- **PTY mode** (`mode = "pty"`) — embedded PTY subprocess; legacy path retained for the TUI frontend and third-party CLI tools.

## Scope

### In scope

- Multi-session management via `AgentManager` (up to 3 concurrent agents)
- API-mode: streaming token delivery, tool-call widgets, chat history
- PTY-mode: spawn, I/O, resize, scrollback, scroll
- Agent status for status bar (both paths)
- Plugin discovery via `plugin.toml` manifest and Settings → Agents panel

### Out of scope

- Agent prompt templates UI
- Persistent chat history across restarts (future)
- Agent orchestration / task queuing (ADR-017)

## Functional Requirements

### Common (both modes)

1. Support multiple concurrent agent sessions; default max 3.
2. Persist `AgentMode` selection via `persist_user_agent_mode`; config survives restart.
3. Settings → Agents lists plugins that declare an `[agent]` section in `plugin.toml`.
4. Status bar reflects agent activity: Idle, Thinking, Executing, Success, Error, Warning.
5. `AgentNew` / `AgentSetActive` / `AgentCycle` manage sessions across both modes.

### API mode

6. On send (`AgentUserSend`): append user message to `ChatSession`, set `is_streaming = true`, emit `AgentEffect::StreamRequest`.
7. Stream token chunks arrive as `AgentTokenChunk`; accumulated in `ChatSession::streaming_text`.
8. On stream complete (`AgentStreamDone`): finalise assistant message, clear streaming state.
9. Tool calls arrive as `AgentToolCallStart` / `AgentToolCallComplete`; rendered as collapsible widgets.
10. Tool execution results returned via `AgentToolResult`.
11. Chat panel renders: message history with user/assistant bubbles, streaming cursor `▋`, tool widgets, multiline text input.
12. `follow_tail` auto-scrolls during streaming; user scroll disengages it.
13. `AgentClearChat` resets `ChatSession::messages` without closing the session.

### PTY mode

14. Lazy-spawn agent on first visit to Agent tab (`agent_spawn_effects_if_needed`).
15. Working directory: repository root; `KIWI_REPO_ROOT` set automatically.
16. Forward keyboard when main focus + Agent tab active.
17. Scrollback buffer 10 000 lines; viewport scroll with `PgUp`/`PgDn` and wheel.
18. Resize propagates to PTY (`TIOCSWINSZ` equivalent).
19. On agent process exit: show exit code; offer restart.

## Non-Functional Requirements

- API stream first token < 2 s on typical LLM endpoints
- PTY read latency < 50 ms batching
- No input lag on typing (either mode)
- Agent spawn (PTY) < 2 s

## Data Structures

```rust
// Per-session state — shared between API and PTY renderers
pub struct AgentState {
    pub command: Vec<String>,
    pub agent_name: String,
    pub status: AgentStatus,        // Idle, Thinking, Executing, Success, Error, Warning
    pub status_bar_label: String,   // cached; updated when status changes

    // PTY-mode fields (None in API mode)
    pub scrollback: ScrollbackBuffer,
    pub running: bool,
    pub spawned: bool,
    pub exit_code: Option<i32>,
    pub cols: u16,
    pub rows: u16,
    pub follow_tail: bool,          // PTY scroll-to-bottom
    pub viewport_offset: usize,
    pub status_check_accum: usize,  // bytes since last heuristic scan

    // API-mode field (None in PTY mode)
    pub chat: Option<ChatSession>,
}

pub struct ChatSession {
    pub messages: Vec<ChatMessage>,
    pub streaming_text: String,
    pub active_tool_call: Option<ToolUse>,
    pub input_draft: String,
    pub is_streaming: bool,
    pub follow_tail: bool,
    pub status: AgentStatus,
    pub model: String,
    pub provider: String,
}

pub enum ContentBlock {
    Text(String),
    ToolUse(ToolUse),
    ToolResult(ToolResult),
}

pub struct ChatMessage {
    pub role: MessageRole,      // User | Assistant
    pub content: Vec<ContentBlock>,
}

// Session manager
pub struct AgentManager {
    agents: Vec<AgentState>,
    active_idx: usize,
    cached_status_label: String,
}
```

## AgentMode

```toml
[agent]
mode = "api"            # "api" (native streaming) | "pty" (legacy subprocess)
provider = "claude"     # api mode only
api_key_env = "ANTHROPIC_API_KEY"
model = "claude-opus-4-8"
command = "agent"       # pty mode; ignored in api mode
```

`AgentMode` is also declared per-plugin in `plugin.toml`:

```toml
[agent]
mode = "api"
provider = "claude"
model = "claude-opus-4-8"
```

## Events / Commands

### API mode

```rust
// Commands (UI → reducer)
AppCommand::AgentUserSend { agent_id, text }
AppCommand::AgentClearChat(agent_id)

// Events (service → reducer)
AppEvent::AgentTokenChunk { agent_id, text }
AppEvent::AgentToolCallStart { agent_id, tool_use_id, tool_name, input_json }
AppEvent::AgentToolCallComplete { agent_id, tool_use_id }
AppEvent::AgentToolResult { agent_id, tool_use_id, output, is_error }
AppEvent::AgentStreamDone { agent_id }
AppEvent::AgentStreamError { agent_id, message }

// Side effects (reducer → service)
AgentEffect::StreamRequest(agent_id)
AgentEffect::CancelStream(agent_id)
AgentEffect::ExecuteTool { agent_id, tool_use_id, tool_name, input_json }
```

### PTY mode

```rust
// Commands
AppCommand::AgentWrite(Vec<u8>)
AppCommand::AgentScroll(i32)
AppCommand::AgentScrollLines(i32)
AppCommand::AgentRestart

// Events
AppEvent::AgentOutput { agent_id, data: Vec<u8> }
AppEvent::AgentExited { agent_id, code: i32 }

// Side effects
AgentEffect::Spawn(agent_id)
AgentEffect::Restart(agent_id)
AgentEffect::Write { agent_id, data }
```

### Shared

```rust
AppCommand::AgentNew
AppCommand::AgentSetActive(AgentId)
AppCommand::AgentCycle(i32)
AppCommand::SetAgent { command, args }
```

## Configuration Options

See `config.example.toml` for the full annotated reference.

## Error Handling

| Error | Behavior |
|-------|----------|
| API key missing | Error banner in chat panel; link to config docs |
| Stream network error | `AgentStreamError` → error badge in status bar |
| Tool execution failure | `ToolResult` marked `is_error = true`; shown inline |
| PTY command not found | Error panel with config hint |
| PTY spawn fail | Retry button in UI |
| Write to dead PTY | Ignore; prompt restart |

## Acceptance Criteria

### API mode

- [x] Chat panel renders when `AgentState.chat.is_some()` (Phase 4 #332)
- [x] User messages right-aligned; assistant messages left-aligned
- [x] Streaming cursor `▋` visible during active stream
- [x] Tool-call widgets collapsible with pretty-printed JSON
- [x] `→ Terminal` button in tool widget routes to shell panel
- [x] Ctrl+Enter sends; Escape clears input
- [x] `AgentMode::Api` initialises `ChatSession` at startup (Phase 6 #334)
- [ ] First-token latency < 2 s on Anthropic claude-opus-4-8
- [ ] Cancel mid-stream via `AgentCancelStream`

### PTY mode

- [x] Agent starts in Agent tab (lazy-spawn)
- [x] Interactive prompts work (keyboard forwarded)
- [x] Scrollback preserves history while scrolling
- [x] Restart recovers from crash
- [ ] Resize propagates to PTY (tracked separately)

### Config / plugin

- [x] `mode = "api"` in `plugin.toml` selects API path (Phase 5 #333)
- [x] `persist_user_agent_mode` writes config without clobbering other fields
- [x] Settings → Agents lists API-mode plugins correctly
