# Agent PTY Pipeline

How Kiwi runs AI agents, streams their output into the Agent tab, and where tool/plugin
authors should hook in. This document reflects the performance-oriented pipeline as of the
Agent tab review (issue #321 branch).

Related contracts:

- [SPEC-010 Agent Service](../specs/SPEC-010-agent-service.md) — behavioral requirements
- [ADR-017 Multi-agent](../architecture/adr/ADR-017-multi-agent-future-design.md) — session model
- [PTY panes](../development/pty-panes.md) — focus, scrollback, keyboard routing

## Mental model

Kiwi does **not** parse agent transcripts into a custom chat model. The Agent tab is a **PTY
viewer**: any configured binary (Cursor CLI, Ollama wrapper, custom tool) runs in a
pseudo-terminal, writes ANSI text to stdout, and reads stdin like a normal TUI.

```text
┌─────────────────┐     read 4 KiB      ┌──────────────────┐
│ Agent process   │ ──────────────────► │ AgentOutputReader│ (background thread)
│ (PTY slave)     │                     └────────┬─────────┘
└────────▲────────┘                              │ AppEvent::AgentOutput
         │ write stdin                           ▼
         │                              ┌──────────────────┐
┌────────┴────────┐                     │ EventChannel     │ coalesce per agent
│ AgentSession    │ ◄── AgentWrite ──── │ drain_coalesced  │
│ (portable-pty)  │                     └────────┬─────────┘
└─────────────────┘                              │ reduce (≤256/frame)
                                                 ▼
                                        ┌──────────────────┐
                                        │ ScrollbackBuffer │ 10_000 lines
                                        └────────┬─────────┘
                                                 │ virtualized rows
                                                 ▼
                                        ┌──────────────────┐
                                        │ Agent dock panel │ egui (GUI) / ratatui (TUI)
                                        └──────────────────┘
```

**Implication for tool builders:** if your agent speaks terminal protocols (ANSI, `\r`, `\n`,
cursor motion), Kiwi will render it correctly. You do not need a Kiwi-specific streaming API
for basic integration—configure `[agent] command` and args.

## Configuration

User-level (`~/.config/kiwi/config.toml`) or repo-level (`.kiwi.toml`):

```toml
[agent]
command = "agent"          # or path to your tool binary
args = ["--model", "gpt-4"]
[agent.env]
MY_API_KEY = "…"             # explicit env overrides repo .env keys
```

GUI **Settings → Agents** lists plugins that declare `agent_command` in `plugin.toml`. Apply
persists config and emits `AgentRestart`.

Launch details (`kiwi_core::agent::session`):

- Working directory: repository root
- `KIWI_REPO_ROOT` set automatically
- Repo `.env` keys forwarded unless overridden in `[agent.env]`
- `TERM` inherited or defaults to `xterm-256color`

## Lifecycle

| Phase | Trigger | Core action | Side effect |
|-------|---------|-------------|-------------|
| Lazy spawn | First visit to Agent tab / `MainTab::Agent` | `agent_spawn_effects_if_needed` | `AgentEffect::Spawn(id)` |
| Attach PTY | Service executes spawn | `AgentSession::spawn` + `AgentOutputReader::spawn` | Updates `AgentState`, starts reader thread |
| Stream output | Reader thread | `AppEvent::AgentOutput { agent_id, data }` | Append to scrollback; optional status scan |
| Input | Keyboard in focused Agent panel | `AppCommand::AgentWrite` | Write bytes to PTY stdin |
| Resize | Panel measures viewport | `viewport.agent_cols/rows` → `resize_agent` | PTY `TIOCSWINSZ` equivalent |
| Exit | Process ends | `poll_agent_exits` → `AgentExited` | `apply_exit`, restart hint in footer |
| Restart | Palette / Ctrl+Shift+R / Settings Apply | `AgentEffect::Restart` | Shutdown session, respawn |

Multi-agent (ADR-017): up to 3 sessions in `AgentManager`; each has its own PTY, scrollback,
and reader thread. Output is routed by `agent_id`.

## Events and commands

### Commands (UI → reducer → side effects)

| Command | Purpose |
|---------|---------|
| `AgentWrite(Vec<u8>)` | Send keystrokes / paste to active agent PTY |
| `AgentScroll(i32)` | Page scroll (`±1` page) |
| `AgentScrollLines(i32)` | Line scroll (mouse wheel) |
| `AgentRestart` | Kill and respawn active session |
| `AgentNew` / `AgentSetActive` / `AgentCycle` | Multi-session management |
| `SetAgent { command, args }` | Persist config + restart (Settings panel) |

### Events (background → reducer)

| Event | Source | Reducer |
|-------|--------|---------|
| `AgentOutput { agent_id, data }` | `AgentOutputReader` | `reduce_agent_output` → append scrollback |
| `AgentExited { agent_id, code }` | Runtime poll / reader error | `reduce_agent_exited` |

`EventChannel::drain_coalesced` merges consecutive `AgentOutput` chunks **per agent** before
reducer runs, reducing per-chunk overhead during fast streams.

## State slices

### `AgentState` (per session)

| Field | Role |
|-------|------|
| `scrollback: ScrollbackBuffer` | Parsed PTY screen + history |
| `running`, `spawned`, `exit_code` | Process lifecycle |
| `follow_tail`, `viewport_offset` | Scroll position |
| `status: AgentStatus` | Inferred activity (status bar + panel chrome) |
| `status_bar_label: String` | Cached label; updated when status/running changes |
| `status_check_accum` | Bytes since last heuristic scan (512-byte threshold) |

### `AgentManager`

Tracks sessions, active id, and `cached_status_label` for the status bar (single-agent uses
per-PTY label; multi-agent shows `"N Agents (M Running)"`).

## Status heuristics (not streaming)

SPEC-010 requires keyword-based status for the status bar. This is **separate from display**:

- Runs at most every **512 bytes** of output while `MainTab::Agent` is active
- Scans the last **32 lines** via `ScrollbackBuffer::recent_stripped_text` (no full-buffer clone)
- On tab focus, `refresh_active_agent_status_heuristic` rescans once
- Patterns live in `kiwi_core::agent::status::infer_status_from_text`

When building tools, you may emit keywords (`Running tool: …`, `Error: …`) for Kiwi chrome,
but **correctness does not depend on them**—the PTY stream is authoritative.

## GUI rendering (Agent dock panel)

Files: `crates/kiwi_gui/src/dock/panels/agent.rs`, `scrollback.rs`.

Performance choices:

1. **Virtualized rows** — only visible scrollback lines (+2 buffer) are laid out each frame
2. **Repaint gating** — repaints on `dirty`, processed events, search debounce, or
   `follow_tail && has_pending_line` (in-progress line without `\n`). No idle 60 fps loop
   while agent process is running
3. **Cached labels** — panel chrome reads `AgentManager::status_bar_label()` (`&str`) without
   per-frame allocation
4. **Streaming chrome** — line-count label hidden while tailing a pending partial line

ANSI colors: `ansi_layout_job` parses SGR sequences per visible row.

## TUI parity

The TUI (`crates/kiwi`) uses the same `kiwi_core` reducer, scrollback, and status logic.
`AgentOutputReader` and coalescing are shared. GUI adds egui_dock panel + `PtyRuntime` in
`kiwi_gui`.

## Building tools and plugins

### Minimal custom agent (recommended path)

1. Ship an executable that reads stdin / writes stdout (optionally ANSI).
2. Point `[agent] command` at it (or declare `agent_command` in `plugin.toml` for Settings
   discovery).
3. Use repo root as cwd; read `KIWI_REPO_ROOT` if you need an absolute path.

No Kiwi SDK required for PTY integration.

### Plugin manifest (Settings discovery only)

```toml
# plugins/my_agent/plugin.toml
[plugin]
name = "my-agent"
display_name = "My Agent"

[agent]
command = "my-agent-cli"
args = ["--workspace"]
```

This registers the agent in **Settings → Agents**; runtime still uses the PTY pipeline above.

### MCP / memory tools (separate from Agent stream)

MCP servers (e.g. `kiwi-mcp-memory`, `kiwi-mcp-context`) are **not** wired into the Agent PTY
stream today. Agents consume them via their own MCP client configuration (see
[mcp-memory-servers.md](./mcp-memory-servers.md)).

Future work (not implemented): plugin API event hooks to push structured tool cards into the
Agent panel without bypassing the PTY. Until then, tool output should go through the agent
binary's stdout.

### What to avoid in tool design

| Avoid | Prefer |
|-------|--------|
| Assuming Kiwi parses JSON message frames | Terminal output with clear line breaks |
| Relying on status keywords for logic | Exit codes and stdout content |
| Huge single-line payloads without `\n` | Periodic newlines or `\r` progress updates |
| Spawning heavy work on every keystroke | Batch work inside your agent process |

## Key source files

| Area | Path |
|------|------|
| PTY spawn / env | `crates/kiwi_core/src/agent/session.rs` |
| Output reader thread | `crates/kiwi_core/src/agent/io.rs` |
| Status heuristics | `crates/kiwi_core/src/agent/status.rs` |
| Multi-session | `crates/kiwi_core/src/agent/manager.rs` |
| Reducer | `crates/kiwi_core/src/reducer/agent.rs` |
| Event coalescing | `crates/kiwi_core/src/events/channel.rs` |
| Scrollback | `crates/kiwi_core/src/shell/scrollback.rs` |
| GUI PTY runtime | `crates/kiwi_gui/src/pty/mod.rs` |
| GUI Agent panel | `crates/kiwi_gui/src/dock/panels/agent.rs` |

## Verification

```bash
cargo test -p kiwi_core agent::
cargo test -p kiwi_core scrollback::
cargo test -p kiwi_gui scrollback::
cargo clippy --workspace -- -D warnings
```

Manual: open Agent tab, run a streaming agent, confirm scrollback keeps up; switch away and
back—history preserved; status bar updates without pegging CPU while idle.
