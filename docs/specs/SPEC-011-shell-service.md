# SPEC-011: Shell Service

## Purpose

Embed an interactive user shell in the bottom-right pane for commands, git operations, and long-running tasks.

## Scope

### In scope

- PTY-backed shell (bash, zsh, fish, user shell)
- Focus and input routing
- Bracketed paste

### Out of scope

- Multiple shell tabs
- Shell history persistence across sessions

## Functional Requirements

1. Spawn shell at startup using `[shell] command`, else `$SHELL`, else `bash`.
2. CWD: repository root; respect `cd` within session.
3. Focus target `Shell` routes keyboard to PTY.
4. Support interactive programs, long-running commands, `Ctrl+C` to interrupt.
5. Scrollback 10_000 lines; scroll when shell focused.
6. Resize PTY on layout change.
7. Click shell pane focuses shell (mouse).

## Non-Functional Requirements

- Same PTY performance targets as SPEC-010
- Bracketed paste enabled for safe paste of multi-line scripts

## Data Structures

```rust
struct ShellState {
    pty: PtyState,
    shell_name: String,
}
```

## Events / Commands

```rust
AppCommand::ShellWrite(Vec<u8>)
AppCommand::ShellScroll(i32)
AppCommand::ShellFocus
AppEvent::ShellOutput(Vec<u8>)
AppEvent::ShellExited(i32)  // auto-restart optional
```

## Configuration Options

```toml
[shell]
command = "bash"
args = ["-l"]    # login shell optional
```

## Error Handling

| Error | Behavior |
|-------|----------|
| Shell exit | Show message; auto-restart shell (default on) |
| Spawn fail | Fatal banner; retry command |

## Acceptance Criteria

- [x] bash/zsh interactive prompt works (prompt visible; `-i` for bash)
- [ ] `git commit`, `npm test` run successfully (manual verification)
- [x] Ctrl+C interrupts running command (shell focus)
- [ ] Paste multi-line script works with bracketed paste
- [x] Focus indicator shows when shell active (accent border)
