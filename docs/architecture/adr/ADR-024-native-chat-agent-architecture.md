# ADR-024: Native Chat Agent Architecture

## Status

Accepted (implemented — issues #329–#334)

## Context

Kiwi's original agent integration ran any configured executable as a PTY subprocess inside the
Agent tab. While this works for CLI tools (Cursor Agent, Ollama wrapper, custom scripts), it has
fundamental limitations for first-class LLM integrations:

- **No structured message history** — the agent's entire conversational state lives in the PTY
  scrollback; Kiwi cannot read, search, or persist individual turns.
- **No structured tool calls** — tool invocations are interleaved as ANSI terminal output; Kiwi
  cannot present them as interactive widgets or route results back to the LLM.
- **No streaming control** — Kiwi cannot cancel mid-generation or inject tool results without
  writing raw bytes to the PTY slave fd.
- **Input coupling** — keyboard focus must be forwarded to the PTY; the agent panel cannot be
  controlled by a purpose-built egui text input with Ctrl+Enter / Escape shortcuts.
- **Status inference is fragile** — `infer_status_from_scrollback` pattern-matches keywords in
  rendered PTY output; it fails on ANSI escape sequences and is not reliable across providers.

The PTY approach works for opaque agent binaries, but cannot deliver a first-class chat UX for
API-native LLMs (Claude, GPT, Gemini).

## Decision

Introduce a second agent execution path — **API mode** — alongside the existing PTY path:

- Kiwi makes direct HTTP/SSE streaming calls to the LLM provider API from within `kiwi_gui` (or
  a future `kiwi_agent_*` service crate).
- Agent state is modelled as a **`ChatSession`** (`Vec<ChatMessage>`, streaming accumulator,
  tool-call list) rather than a PTY scrollback buffer.
- A new **native chat panel** (`dock/panels/chat.rs`) renders the message list, tool widgets, and
  a text input. It is shown when `AgentState.chat.is_some()`; the existing PTY panel is shown
  otherwise — no flag day for existing users.
- **Tool calls** arrive as structured `AgentToolCallStart` / `AgentToolCallComplete` events;
  results are returned via `AgentToolResult`. The reducer wires them into `ChatSession::messages`
  as `ContentBlock::ToolUse` / `ContentBlock::ToolResult`.
- **Streaming** is managed via `StreamCancelHandle`; cancel is a first-class operation.
- **AgentMode** (`"api"` | `"pty"`) is declared in both `config.toml` and `plugin.toml`.

### Execution path selection

```text
config.agent.mode == AgentMode::Api
    → AppState::from_startup initialises AgentState { chat: Some(ChatSession { .. }) }
    → agent.rs panel checks chat.is_some() → renders chat.rs
    → AgentEffect::StreamRequest → services.rs → HTTP streaming task

config.agent.mode == AgentMode::Pty  (default)
    → AgentState { chat: None }
    → agent.rs panel renders PTY scrollback (unchanged)
    → AgentEffect::Spawn → services.rs → PtyRuntime::spawn_agent
```

### What was NOT changed

- `kiwi_core` PTY files (`agent/session.rs`, `agent/io.rs`, `agent/command.rs`) are retained —
  the TUI frontend (`crates/kiwi`) still uses the PTY path.
- `AgentManager`, multi-session tabs, and `AgentStatus` enum are shared between paths.
- `infer_status_from_scrollback` is retained in `kiwi_core::agent::status` for the TUI; the GUI
  no longer calls it (`apply_status_heuristic` returns `false` unconditionally in kiwi_gui).

### Implementation phases

| Issue | Phase | Scope |
|-------|-------|-------|
| #329 | 1 | `ChatSession` and `ContentBlock` types in kiwi_core |
| #330 | 2 | Streaming events and reducer wiring |
| #331 | 3 | Tool execution plumbing (ExecuteTool, ToolResult) |
| #332 | 4 | Native egui chat panel UI |
| #333 | 5 | Config and plugin manifest `AgentMode` fields |
| #334 | 6 | Remove PTY agent code from kiwi_gui |

## Consequences

### Positive

- Structured message history enables future features: search, export, context injection.
- Tool calls are interactive widgets, not ANSI noise; results are routed correctly.
- Cancel mid-generation is reliable (`StreamCancelHandle`).
- Status updates come from structured events, not fragile keyword matching.
- Backward compatible: existing PTY-mode agents continue to work without config changes.
- Plugin authors can declare `mode = "api"` to opt into the native panel.

### Negative

- Two rendering code paths to maintain until PTY mode is fully sunset.
- API-mode agents require a Kiwi-specific integration (HTTP client + event mapping); opaque CLI
  tools cannot use the native chat panel without a wrapper.
- `apply_status_heuristic` is now a no-op in kiwi_gui, leaving the PTY status inference dead for
  GUI users even when they use PTY mode. Acceptable for now; will be removed when PTY path is
  sunsetted.
- Chat history is in-process memory; it does not survive kiwi-gui restarts. Persistence is
  deferred (tracked separately).

## Alternatives Considered

| Alternative | Rejection Rationale |
|-------------|---------------------|
| Keep PTY-only, add structured parsing layer | Fragile; ANSI stripping loses fidelity; still no cancel/tool-result injection |
| WebView / Electron-style chat embed | Introduces a browser engine dependency; conflicts with Kiwi's egui-only GUI policy |
| Separate chat process communicating over IPC | High complexity; splits state that the reducer already manages cleanly |
| Full PTY removal in one PR | Would break TUI and all existing PTY-mode users simultaneously; phased approach preferred |

## Follow-up Work

- HTTP/SSE streaming client implementation (service layer, separate issue)
- Persist `ChatSession::messages` to disk between sessions
- Sunset PTY mode from kiwi_gui once API-mode parity is confirmed (remove `apply_status_heuristic`, PTY panel code)
- Extend plugin API to allow plugins to supply their own stream handler without forking the service layer
- Multi-provider support (OpenAI, Gemini) via provider-agnostic event mapping
