# ADR-002: TUI Framework Selection

## Status

Accepted

## Context

Kiwi requires a rich terminal UI: split panes, tabs, scrollable lists, diff rendering, embedded PTY output, mouse support, and theming. The stack must be actively maintained, work across Linux/macOS, and integrate with async I/O for PTY and subprocess management.

## Decision

Use **ratatui** for widget rendering and layout, **crossterm** for terminal backend (input, output, mouse, colors), and **tokio** as the async runtime.

```toml
# Core dependencies
ratatui = "0.29"   # pin at scaffold time
crossterm = "0.28"
tokio = { version = "1", features = ["full"] }
```

Rendering runs on the main thread in a tight event loop. Async services communicate via channels; the main loop drains pending messages each tick before redraw.

## Consequences

### Positive

- ratatui is the de facto Rust TUI ecosystem successor to tui-rs
- crossterm provides cross-platform terminal control without ncurses
- tokio integrates cleanly with PTY reads, file watcher, and subprocess I/O
- Large community examples for tabs, lists, and layouts

### Negative

- Main-thread rendering can bottleneck on very large diff renders (mitigate with virtualization)
- ratatui + crossterm version coupling must be tracked
- No built-in PTY widget; custom integration required (ADR-006)

## Alternatives Considered

| Alternative | Rejection Rationale |
|-------------|---------------------|
| **tuirealm** | Higher-level but smaller ecosystem; less flexibility for custom PTY panes |
| **ncurses via ncurses-rs** | Platform portability and mouse handling more painful |
| **termion** | Less maintained; crossterm is preferred for new projects |
| **egui + terminal** | Not terminal-native; wrong abstraction |

## Follow-up Work

- SPEC-002 Layout Engine: define widget hierarchy and render budget
- Benchmark diff rendering with 10k+ line hunks; add viewport virtualization if needed
- Document minimum terminal size (80×24) and recommended size (120×40+)
- Enable bracketed paste via crossterm for shell and command palette
