# Native Chat Agent Migration Plan

**Status:** Approved for implementation  
**Replaces:** PTY-based agent pipeline (ADR-006, ADR-017, `docs/agents/pty-pipeline.md`)  
**Date:** 2026-06-28

---

## Background and Motivation

The current Agent tab spawns a CLI process (e.g. `claude`, `cursor`) in a pseudo-terminal (PTY),
captures raw ANSI bytes, feeds them through a scrollback buffer, and re-renders the output in
egui using a custom ANSI parser. This approach has produced a growing list of problems:

- **Alt-screen conflicts** — Agent CLIs use `?1049h`/`?1049l` (alternate screen) to render their
  own TUI, which collides with kiwi's rendering (fixed in PR #327 but fragile by design).
- **True-color and SGR edge cases** — ANSI parsing must cover the full terminal emulator spec to
  correctly render every agent's output. Each new agent adds new failure modes.
- **No structured data** — Agent actions (file reads, writes, shell commands) are embedded in
  styled text; kiwi cannot inspect, approve, cancel, or display them as first-class UI elements.
- **Keyboard routing complexity** — Raw PTY key encoding (`pty_input.rs`) must cover every
  modifier combination an agent TUI might rely on.
- **Fragile status heuristics** — `infer_status_from_text` reads scrollback for keywords like
  "Running tool:". This will always lag and misclassify.

VS Code / Cursor solve this by calling the LLM API directly and rendering responses as structured
chat. They do not capture a TUI process. This plan migrates kiwi to the same model.

---

## Architecture: Before and After

### Current (PTY)

```text
Agent CLI process (PTY slave)
        │  raw bytes (ANSI, cursor codes, alt-screen)
        ▼
AgentOutputReader (background thread)
        │  AppEvent::AgentOutput { data: Vec<u8> }
        ▼
EventChannel (coalesces chunks)
        │
        ▼
ScrollbackBuffer  ──► ANSI parser ──► egui LayoutJob ──► Agent dock panel
                                                              (scrollback renderer)
        ▲
AgentWrite(Vec<u8>) ◄── pty_input.rs (raw key encoding) ◄── egui keyboard events
```

Problems: opaque bytes, terminal emulator complexity, no tool visibility, ANSI edge cases.

### Target (Native Chat)

```text
User types message in chat input box
        │  AppCommand::AgentUserSend(text)
        ▼
API streaming task (tokio, reqwest SSE)
        │  AppEvent::AgentTokenChunk / AgentToolCallStart / AgentTurnComplete
        ▼
Reducer ──► ChatSession { messages: Vec<ChatMessage> }
        │
        ▼
Chat panel (egui)
  ├── message history (scrollable, virtualized)
  │     ├── User message bubbles (plain text)
  │     ├── Assistant message (markdown via egui_commonmark)
  │     └── Tool-use widgets (collapsible: ReadFile, RunBash, WriteFile …)
  └── text input box (egui TextEdit, multiline)
        ▲
Tool executor (kiwi owns execution)
  ├── ReadFile  ──► std::fs
  ├── WriteFile ──► std::fs
  ├── RunBash   ──► existing Terminal PTY panel (output shown there)
  ├── GitStatus ──► kiwi_core::git
  └── ListDir   ──► kiwi_core::file_tree
```

Benefits: structured data, rich rendering, tool visibility, no ANSI parsing, clean status.

---

## Scope

### In scope

- Replace `AgentState` PTY fields with `ChatSession` message history
- New Claude API streaming client (Anthropic SDK via `reqwest` + SSE)
- Tool execution system with file, git, and bash tools
- New egui chat panel with message list, tool widgets, text input
- Config migration: `[agent]` gains `provider`, `api_key`, `model`
- Plugin manifest: `mode = "api"` for API-based agents
- Remove `AgentSession`, `AgentOutputReader`, `AgentRuntime`, PTY keyboard routing
- Update SPEC-010 and related docs

### Out of scope (this migration)

- TUI (`crates/kiwi`) agent panel — keeps PTY approach until a follow-up
- Ollama / OpenAI provider implementations — Claude first, then extend
- Agent conversation history persistence — future work (SPEC-017 extension)
- Approval flows / human-in-the-loop tool confirmation — future work

### Preserved unchanged

- Shell / Terminal panel (PTY stays correct for the shell)
- `AgentManager` multi-session concept (session state changes, not the manager structure)
- Navigation, tab routing, `AgentId`
- All git, GitHub, diff, file panels
- Plugin discovery via `plugin.toml` (manifest extended, not replaced)

---

## New Data Model

### `kiwi_core::agent::chat` (new module)

```rust
/// A single turn in the conversation.
pub struct ChatMessage {
    pub role: MessageRole,
    pub blocks: Vec<ContentBlock>,
    pub timestamp: std::time::SystemTime,
}

pub enum MessageRole {
    User,
    Assistant,
}

pub enum ContentBlock {
    Text(String),
    ToolUse(ToolUse),
    ToolResult(ToolResult),
}

pub struct ToolUse {
    pub id: String,            // Claude tool_use_id for result correlation
    pub name: String,          // "read_file", "run_bash", etc.
    pub input: serde_json::Value,
    pub collapsed: bool,       // UI state: expand/collapse tool widget
}

pub struct ToolResult {
    pub tool_use_id: String,
    pub content: String,
    pub is_error: bool,
}
```

### `ChatSession` (replaces `AgentState` PTY fields)

```rust
pub struct ChatSession {
    // Conversation
    pub messages: Vec<ChatMessage>,
    pub input_draft: String,         // text box content

    // Streaming state
    pub is_streaming: bool,
    pub streaming_text: String,      // partial assistant token accumulator
    pub active_tool_call: Option<ToolUse>,

    // Scroll
    pub scroll_offset: usize,
    pub follow_tail: bool,

    // Status
    pub status: AgentStatus,
    pub error: Option<String>,
    pub status_bar_label: String,

    // API config (resolved from config at spawn time)
    pub model: String,
    pub provider: AgentProvider,
}

pub enum AgentProvider {
    Claude { api_key: String },
    // Future: OpenAI, Ollama
}
```

`AgentManager` holds `ChatSession` per session instead of `AgentState`. The `AgentId` type,
session count, active tracking, and status bar label logic are unchanged.

### New events

```rust
// Background → reducer
AppEvent::AgentTokenChunk { agent_id: AgentId, text: String }
AppEvent::AgentToolCallStart { agent_id: AgentId, tool: ToolUse }
AppEvent::AgentToolResult { agent_id: AgentId, result: ToolResult }
AppEvent::AgentTurnComplete { agent_id: AgentId }
AppEvent::AgentError { agent_id: AgentId, message: String }

// UI → reducer
AppCommand::AgentUserSend { agent_id: AgentId, text: String }
AppCommand::AgentToggleToolExpand { agent_id: AgentId, tool_use_id: String }
AppCommand::AgentClearHistory { agent_id: AgentId }
```

Old events removed: `AgentOutput { data: Vec<u8> }`, `AgentExited { code }`.  
Old commands removed: `AgentWrite(Vec<u8>)`, `AgentScroll`, `AgentScrollLines`.

---

## Tool Execution System

Tools are defined as a kiwi-side enum. When the API returns a `tool_use` block, the reducer emits
a `SideEffect::Agent(AgentEffect::ExecuteTool(...))`, the service executes it, and feeds
`AppEvent::AgentToolResult` back into the event loop.

### Tool definitions (sent to API as JSON schema)

| Tool name | Description | Inputs |
|-----------|-------------|--------|
| `read_file` | Read a file from the repo | `path: String` |
| `write_file` | Write or overwrite a file | `path: String, content: String` |
| `list_directory` | List a directory tree | `path: String, depth: u8` |
| `run_bash` | Run a shell command (routes to Terminal panel) | `command: String` |
| `git_status` | Current git status | _(none)_ |
| `git_diff` | Diff for a file or all changes | `path: Option<String>` |
| `search_files` | Fuzzy-find files by name | `query: String` |
| `search_content` | ripgrep content search | `query: String, path: Option<String>` |

### `run_bash` special handling

Rather than capturing bash output into the chat panel, `run_bash` routes the command to the
existing Terminal PTY panel, which is already a correctly implemented PTY renderer. The chat panel
shows a tool widget: `▶ Running: cargo test` with a link to switch to the Terminal tab. Output
appears in the Terminal as it would normally.

---

## Config Schema Changes

### Current

```toml
[agent]
command = "claude"
args = []
[agent.env]
ANTHROPIC_API_KEY = "..."
```

### New

```toml
[agent]
provider = "claude"          # "claude" | "pty" (legacy) | future: "openai", "ollama"
model = "claude-opus-4-8"    # default
api_key = "..."              # or set ANTHROPIC_API_KEY env var

# Legacy PTY mode (for non-API agents)
# provider = "pty"
# command = "aider"
# args = ["--model", "gpt-4"]
```

`provider = "pty"` keeps the existing PTY pipeline alive for agents that don't have an API.
This is the hybrid fallback — the GUI renders a PTY scrollback panel for those agents, exactly
as today.

### Plugin manifest extension

```toml
# plugins/kiwi_agent_claude/plugin.toml
[plugin]
name = "kiwi-agent-claude"
display_name = "Claude (Anthropic)"

[agent]
mode = "api"                 # new field: "api" | "pty"
provider = "claude"
model = "claude-opus-4-8"
```

Settings → Agents continues to discover plugins via manifest. For `mode = "api"` plugins,
applying the plugin writes the `provider`/`model` fields; for `mode = "pty"`, it writes
`command`/`args` as before.

---

## Implementation Phases

Each phase is independently testable and shippable as its own PR.

---

### Phase 1 — New Data Model in `kiwi_core`

**Goal:** Replace PTY-specific `AgentState` fields with `ChatSession`. No UI changes yet;
both old and new fields can coexist temporarily behind a feature flag or by keeping the
reducer wired to stubs.

**Files changed:**

| File | Change |
|------|--------|
| `crates/kiwi_core/src/agent/chat.rs` | New — `ChatMessage`, `ContentBlock`, `ToolUse`, `ToolResult`, `MessageRole` |
| `crates/kiwi_core/src/agent/mod.rs` | Export new chat types |
| `crates/kiwi_core/src/state/domains.rs` | Replace `AgentState` PTY fields with `ChatSession` fields; keep `AgentStatus`, `status_bar_label` |
| `crates/kiwi_core/src/events/mod.rs` | Add new `AppEvent` variants; keep old ones until Phase 5 |
| `crates/kiwi_core/src/reducer/agent.rs` | Add stubs for new event handlers; old handlers still compile |
| `crates/kiwi_core/src/agent/status.rs` | Remove scrollback heuristic; replace with direct status from API events |

**GitHub issue title:** `[Agent] Phase 1: Replace AgentState PTY fields with ChatSession data model`

---

### Phase 2 — Claude API Streaming Client

**Goal:** Implement the HTTP streaming client that calls the Claude Messages API, streams
tokens via SSE, and feeds events back through `EventSender`.

**New dependency:** `reqwest` with `stream` feature (already in workspace? verify),
`eventsource-stream` or manual SSE parsing, `serde_json`.

**Files changed / created:**

| File | Change |
|------|--------|
| `crates/kiwi_core/src/agent/api_client.rs` | New — `ClaudeClient`, `stream_message()` returns `impl Stream<Item=StreamEvent>` |
| `crates/kiwi_core/src/agent/stream_event.rs` | New — `StreamEvent` enum matching Anthropic SSE types |
| `crates/kiwi_core/src/agent/provider.rs` | New — `AgentProvider` enum, `create_client()` factory |
| `crates/kiwi_gui/src/services.rs` | Replace `spawn_agent` PTY side-effect with `spawn_api_task` tokio task |
| `crates/kiwi_core/src/reducer/agent.rs` | Wire `AgentTokenChunk`, `AgentTurnComplete`, `AgentError` reducers |

**Streaming task lifecycle:**

```text
AppCommand::AgentUserSend
    ├── reducer appends User message to ChatSession
    ├── sets is_streaming = true
    └── SideEffect::Agent(AgentEffect::StreamRequest { id, messages, tools })
            │
            ▼ (kiwi_gui service)
        tokio::spawn {
            ClaudeClient::stream_message(...)
                .for_each(|event| sender.send(AppEvent::AgentTokenChunk / ToolCallStart / TurnComplete))
        }
```

**GitHub issue title:** `[Agent] Phase 2: Claude API streaming client (reqwest SSE → EventSender)`

---

### Phase 3 — Tool Execution System

**Goal:** Implement the server-side tool executor. When the API returns a `tool_use` block,
kiwi executes the tool locally and feeds the result back.

**Files changed / created:**

| File | Change |
|------|--------|
| `crates/kiwi_core/src/agent/tools.rs` | New — `KiwiTool` enum, JSON schema definitions for API |
| `crates/kiwi_core/src/agent/tool_executor.rs` | New — `execute_tool(tool, repo_root) -> ToolResult` |
| `crates/kiwi_core/src/events/mod.rs` | `AgentEffect::ExecuteTool { id, agent_id, tool: KiwiTool }` |
| `crates/kiwi_gui/src/services.rs` | Handle `ExecuteTool` side-effect: run executor, send `AgentToolResult` |
| `crates/kiwi_core/src/reducer/agent.rs` | `reduce_agent_tool_result` — appends `ToolResult` block, feeds next API turn |

**Tool execution loop:**

```text
AgentToolCallStart event
    ├── reducer appends ToolUse block to current message
    └── SideEffect::Agent(AgentEffect::ExecuteTool { tool })
            │
            ▼ (service)
        execute_tool(tool, repo_root)
            │  ToolResult { content, is_error }
            ▼
        AppEvent::AgentToolResult
            │
            ▼ (reducer)
        append ToolResult block
        SideEffect::Agent(AgentEffect::StreamRequest) // continue conversation
```

**GitHub issue title:** `[Agent] Phase 3: Tool execution system (file, git, bash tools)`

---

### Phase 4 — Chat Panel UI (`kiwi_gui`)

**Goal:** Replace the ANSI scrollback panel with a proper chat UI. This is the most user-visible
change.

**Dependencies:** `egui_commonmark` for markdown rendering (add to `Cargo.toml`).

**Files changed / created:**

| File | Change |
|------|--------|
| `crates/kiwi_gui/src/dock/panels/chat.rs` | New — top-level `render(ui, ctx)` for the Agent tab |
| `crates/kiwi_gui/src/dock/panels/chat_message.rs` | New — per-message renderer (user/assistant/tool) |
| `crates/kiwi_gui/src/dock/panels/chat_input.rs` | New — text input box, send button, keyboard handling |
| `crates/kiwi_gui/src/dock/panels/chat_tool_widget.rs` | New — collapsible tool-use widget |
| `crates/kiwi_gui/src/dock/panels/agent.rs` | Replace PTY render call with `chat::render(ui, ctx)` |
| `crates/kiwi_gui/src/dock/panels/scrollback.rs` | Remove `render_agent_panel`; keep `render_shell_panel` |
| `crates/kiwi_gui/src/dock/panels/pty_input.rs` | Remove agent keyboard path; shell path unchanged |

**Chat panel layout:**

```
┌─────────────────────────────────────────┐
│ Agent · claude-opus-4-8  [● streaming]  │  ← chrome (existing pattern)
├─────────────────────────────────────────┤
│                                         │
│  [You]  Fix the scrollback bug          │  ← user message
│                                         │
│  [Claude]  I'll start by reading the    │  ← assistant text (markdown)
│  scrollback module...                   │
│  ▶ read_file  src/shell/scrollback.rs   │  ← tool widget (collapsible)
│  The issue is on line 47. I'll fix it.  │
│  ▶ write_file  src/shell/scrollback.rs  │
│                                         │
│  [You]  Run the tests                   │
│  [Claude]  ▋                            │  ← streaming cursor
│                                         │
├─────────────────────────────────────────┤
│ > Type a message...           [Ctrl+↵]  │  ← text input
└─────────────────────────────────────────┘
```

**Keyboard behaviour:**

- `Enter` — send message
- `Shift+Enter` — newline in input
- `Ctrl+C` — copy last assistant message to clipboard
- `Escape` — clear input draft
- `PageUp`/`PageDown` — scroll message history

**GitHub issue title:** `[Agent] Phase 4: Native chat panel UI (message list, markdown, tool widgets, text input)`

---

### Phase 5 — Config and Plugin Migration

**Goal:** Update `ResolvedConfig`, the config loader/writer, and plugin manifests to support
the new `[agent]` schema. Maintain backward compatibility for `mode = "pty"` users.

**Files changed:**

| File | Change |
|------|--------|
| `crates/kiwi_core/src/config/types.rs` | `AgentSettings` gains `provider`, `api_key`, `model`; keep `command`/`args` for PTY |
| `crates/kiwi_core/src/config/loader.rs` | Parse new fields; detect old-style config and emit deprecation warning |
| `crates/kiwi_core/src/config/writer.rs` | Write new schema |
| `crates/kiwi_plugin_api/src/manifest.rs` | `AgentManifest` gains `mode: AgentMode` (`Api` | `Pty`) |
| `plugins/kiwi_agent_claude/plugin.toml` | Update to `mode = "api"`, `provider = "claude"` |
| `config.example.toml` | Document new schema with comments |
| `docs/agents/pty-pipeline.md` | Add deprecation notice; link to new doc |

**GitHub issue title:** `[Agent] Phase 5: Config and plugin manifest migration (api vs pty mode)`

---

### Phase 6 — Remove Legacy PTY Agent Code

**Goal:** Delete all PTY-agent source files that are no longer needed after Phases 1–5 ship and
tests are green. Do not delete until Phase 4 is merged and manually verified.

**Files deleted:**

| File | Reason |
|------|--------|
| `crates/kiwi_core/src/agent/session.rs` | `AgentSession` (PTY spawn) no longer used for API agents |
| `crates/kiwi_core/src/agent/io.rs` | `AgentOutputReader` thread gone |
| `crates/kiwi_core/src/agent/command.rs` | `AgentLaunchSpec` replaced by `AgentProvider` |
| `crates/kiwi_core/src/agent/status.rs` | `infer_status_from_scrollback` removed; status comes from API events |
| `crates/kiwi_gui/src/pty/agent_runtime.rs` | `AgentRuntime` gone |
| `crates/kiwi_core/src/shell/scrollback.rs` `PrimaryScreenSnapshot` | Alt-screen workaround no longer needed (unless Shell still needs it) |

**Note:** `crates/kiwi_core/src/shell/scrollback.rs` itself stays — the Shell tab still uses it.
Only the `PrimaryScreenSnapshot` / alt-screen handling added in PR #327 can be removed if no
other panel needs it.

**Files updated:**

| File | Change |
|------|--------|
| `crates/kiwi_core/src/agent/mod.rs` | Remove PTY exports |
| `crates/kiwi_core/src/events/mod.rs` | Remove `AgentOutput`, `AgentExited` old variants |
| `crates/kiwi_core/src/reducer/agent.rs` | Remove old PTY reducer branches |
| All `AppCommand::AgentWrite` call sites | Remove (none should exist after Phase 4) |

**GitHub issue title:** `[Agent] Phase 6: Remove legacy PTY agent code and clean up`

---

### Phase 7 — Update SPEC-010 and Documentation

**Goal:** Rewrite the agent-facing documentation to reflect the new architecture. Deprecate or
archive the PTY pipeline doc.

**Documents to update / create:**

| Document | Action |
|----------|--------|
| `docs/specs/SPEC-010-agent-service.md` | Rewrite: new data model, API client, tool contracts |
| `docs/agents/pty-pipeline.md` | Add top-of-file deprecation notice; link to new doc |
| `docs/agents/native-chat-pipeline.md` | New — mirrors pty-pipeline.md for the new architecture |
| `docs/agents/tool-reference.md` | New — each tool's schema, inputs, outputs, examples |
| `docs/architecture/adr/ADR-024-native-chat-agent.md` | New ADR superseding PTY approach for API agents |
| `docs/roadmap/milestones.md` | Update M7 to reflect chat migration as the agent goal |
| `AGENTS.md` (repo root) | Update agent configuration instructions |

**GitHub issue title:** `[Agent] Phase 7: Update SPEC-010, ADR-024, and agent documentation`

---

## Dependency Summary

```
Phase 1 (data model)
    └── Phase 2 (API client)           depends on Phase 1 types
            └── Phase 3 (tools)        depends on Phase 2 streaming loop
                    └── Phase 4 (UI)   depends on Phase 1 types + Phase 3 tool widgets
                            └── Phase 5 (config)   depends on Phase 4 working
                                    └── Phase 6 (remove legacy)   depends on Phase 4+5 verified
                                            └── Phase 7 (docs)    depends on Phase 6 merged
```

Phases 2 and 3 can be developed in parallel once Phase 1 merges. Phase 5 can start alongside
Phase 4 since config types are independent of the UI.

---

## Risk Register

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| API key management UX — where do users store the key? | High | Medium | Support env var (`ANTHROPIC_API_KEY`) first; settings panel field later |
| `run_bash` tool routing to Terminal tab requires UI feedback | Medium | Low | Show tool widget with "output in Terminal tab" link; dispatch `NavCommand::SwitchToTerminal` |
| `egui_commonmark` rendering edge cases (tables, images) | Medium | Low | Fall back to plain text for unknown blocks |
| Token costs — long conversation histories | Medium | Medium | Add `AgentClearHistory` command; expose token count in chrome |
| TUI (`crates/kiwi`) left behind — diverges from GUI | High | Low | TUI keeps PTY agent for now; converge in a follow-up milestone |
| Other agent CLIs (Cursor, aider) lose support | Medium | Medium | `provider = "pty"` hybrid path preserves them; document migration path |
| Streaming task cancellation on panel close / restart | Medium | High | Track task handle in `AgentRuntime`; cancel on `AgentRestart` / window close |
| Rate limit and auth error handling | Low | Medium | Surface `AgentError` event → error message in chat panel |

---

## TUI Compatibility Note

`crates/kiwi` (the terminal UI) also uses `AgentState` via `kiwi_core`. During this migration:

- The TUI **keeps the PTY agent path** unchanged. It will use `provider = "pty"` implicitly.
- `kiwi_core` types will need to remain compatible with both modes until a TUI chat panel is
  built (future milestone).
- A `ChatSession`-based state must be conditionally used based on provider mode, or a wrapper
  type must unify both.

The cleanest approach: make `AgentState` a sum type:

```rust
pub enum AgentState {
    Chat(ChatSession),
    Pty(PtyAgentState),    // existing fields renamed
}
```

The reducers branch on the variant. The GUI always creates `Chat`; the TUI creates `Pty`.

---

## Success Criteria

The migration is complete when:

1. Opening the Agent tab with `provider = "claude"` shows a chat UI with a text input box.
2. Typing a message and pressing Enter sends it to the Claude API and streams the response as
   rendered markdown in the message list.
3. A tool call (e.g. `read_file`) is visible in the chat as a collapsible widget showing the
   file path and result size.
4. A `run_bash` tool call routes the command to the Terminal panel; the chat widget shows a link.
5. No ANSI escape codes appear in the chat panel under any circumstances.
6. Status bar correctly shows Thinking / Executing / Success / Error from API event state — no
   keyword heuristics.
7. `provider = "pty"` still works for non-API agents (Cursor, aider) using the existing PTY
   scrollback renderer.
8. All `cargo test --workspace` tests pass.
9. SPEC-010 acceptance criteria rewritten and met by the new implementation.

---

## Related Documents

- `docs/agents/pty-pipeline.md` — current architecture being superseded
- `docs/specs/SPEC-010-agent-service.md` — behavioral spec (to be rewritten in Phase 7)
- `docs/architecture/adr/ADR-006-pty-architecture.md` — original PTY decision
- `docs/architecture/adr/ADR-017-multi-agent-future-design.md` — multi-session model (preserved)
- `docs/architecture/adr/ADR-018-plugin-architecture.md` — plugin manifest (extended in Phase 5)
- `docs/roadmap/milestones.md` — M7 Advanced Features
