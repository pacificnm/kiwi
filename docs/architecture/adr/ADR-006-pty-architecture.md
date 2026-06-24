# ADR-006: PTY Architecture

## Status

Accepted

## Context

Kiwi embeds interactive shell sessions and AI agent sessions in the bottom-right pane and Agent main tab respectively. These require pseudo-terminal support for colors, cursor movement, interactive prompts, and long-running processes.

## Decision

Use **portable-pty** with **tokio**-compatible async read/write loops.

### Two PTY instances

| Instance | Purpose | Default command |
|----------|---------|-----------------|
| `ShellPty` | User shell in bottom panel | `$SHELL`, config `[shell]`, or `bash` |
| `AgentPty` | AI agent in Agent tab | config `[agent]`, default `agent` (Cursor Agent) |

### Architecture

```text
PTY Master (Kiwi)  ←→  portable-pty  ←→  Child Process (shell/agent)
        ↓
   Ring buffer per PTY
        ↓
   ratatui Paragraph / custom ANSI parser (viewport)
```

- Spawn child with working directory = repository root
- Forward keyboard input when shell/agent pane focused
- Support bracketed paste in shell and command palette
- Resize PTY on terminal resize event (rows/cols for bottom pane)
- Agent PTY may run full-screen within main tab content area

### Output handling

Buffer PTY output in a scrollback ring (default 10_000 lines). Render only visible viewport. Do not block main loop on PTY read; use `tokio::select!` between crossterm events and PTY readiness.

## Consequences

### Positive

- Real interactive shell and agent experience
- portable-pty works on Linux and macOS
- Decoupled buffers allow scroll without losing history

### Negative

- ANSI parsing for faithful display is non-trivial
- Two PTYs increase resource usage
- Child process lifecycle must handle signals and orphan cleanup

## Alternatives Considered

| Alternative | Rejection Rationale |
|-------------|---------------------|
| Pipe without PTY | Breaks interactive programs (vim in shell, agent TUI) |
| tmux/screen embedding | External dependency; poor integration |
| Multiple windows | Violates single-workspace model |

## Follow-up Work

- SPEC-010 Agent Service, SPEC-011 Shell Service
- Handle `C-c`, `C-d`, process exit and restart UX
- Document minimum cols for agent TUI (typically 80+)
- Future: multiple agent PTYs (ADR-017)
