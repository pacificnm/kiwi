# SPEC-010: Agent Service

## Purpose

Manage embedded PTY session for AI agent (default Cursor Agent) in the Agent main tab.

## Scope

### In scope

- Single agent PTY (MVP)
- Spawn, I/O, resize, scroll
- Agent status for status bar

### Out of scope

- Multi-agent (SPEC future / ADR-017)
- Agent prompt templates UI

## Functional Requirements

1. Lazy-spawn agent on first visit to Agent tab using `[agent] command` (default `agent`).
2. Working directory: repository root.
3. Forward keyboard when main focus + Agent tab active.
4. Scrollback buffer 10_000 lines; viewport scroll with `PgUp`/`PgDn` and wheel.
5. Parse agent output heuristically for status bar: Idle, Running, Error (keyword patterns configurable later).
6. Restart agent: command palette `Agent: Restart`.
7. On agent process exit: show exit code; offer restart.

## Non-Functional Requirements

- PTY read latency < 50ms batching
- No input lag on typing
- Agent spawn < 2s

## Data Structures

```rust
struct PtyState {
    buffer: ScrollbackBuffer,
    viewport_offset: usize,
    cols: u16,
    rows: u16,
    running: bool,
    exit_code: Option<i32>,
    child_pid: Option<u32>,
}

struct AgentState {
    pty: PtyState,
    status: AgentStatus,  // Idle, Thinking, Executing, Success, Error, Warning
}

// Placeholder for multi-agent (ADR-017)
type AgentId = u32;
```

## Events / Commands

```rust
AppCommand::AgentSpawn
AppCommand::AgentRestart
AppCommand::AgentWrite(Vec<u8>)
AppCommand::AgentScroll(i32)
AppEvent::AgentOutput(Vec<u8>)
AppEvent::AgentExited(i32)
AppEvent::TerminalResize { cols, rows }
```

## Configuration Options

```toml
[agent]
command = "agent"
args = []              # optional
env = { KEY = "val" }  # optional
```

## Error Handling

| Error | Behavior |
|-------|----------|
| Command not found | Error panel with config hint |
| Spawn fail | Retry button in UI |
| Write to dead PTY | Ignore; prompt restart |

## Acceptance Criteria

- [x] Agent starts in Agent tab
- [x] Interactive prompts work (Main focus + Agent tab; keyboard forwarded)
- [x] Scrollback preserves history while scrolling
- [ ] Resize propagates to PTY
- [x] Status bar reflects running state (heuristics — #25)
- [ ] Restart recovers from crash (#26)
