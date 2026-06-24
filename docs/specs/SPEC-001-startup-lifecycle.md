# SPEC-001: Startup Lifecycle

## Purpose

Define how Kiwi initializes, validates environment, enters the main event loop, and shuts down cleanly.

## Scope

### In scope

- CLI argument parsing
- Config loading and merge
- Terminal setup (raw mode, alternate screen, mouse)
- Repository root detection
- Service spawning order
- Graceful shutdown

### Out of scope

- Auto-update mechanism
- Daemon mode

## Functional Requirements

1. **CLI** — Accept `kiwi [PATH]` where PATH defaults to current directory; flags: `--config`, `--theme`, `--help`, `--version`.
2. **Repository validation** — PATH must exist and be a directory; warn if not a git repo but continue.
3. **Config load** — Merge per ADR-005; produce `ResolvedConfig`.
4. **Terminal init** — Enable raw mode, alternate screen, bracketed paste, mouse if configured.
5. **State init** — Load workspace persistence if present (SPEC-017); else defaults.
6. **Service start** — Spawn tokio runtime tasks: watcher (if git repo), shell PTY, optional agent PTY lazy on first Agent tab visit.
7. **Main loop** — Poll crossterm events + app event channel; render at 60fps cap or on dirty flag.
8. **Shutdown** — On `q` or `Ctrl+C`: save workspace, restore terminal, kill PTY children, exit 0.

## Non-Functional Requirements

- Cold start to first frame: < 500ms on typical laptop (excluding large repo git init)
- No panic on missing optional tools (`gh`, `rg`) — degrade gracefully
- All terminal state restored on panic via `Drop` guard

## Data Structures

```rust
struct StartupContext {
    repo_root: PathBuf,
    config: ResolvedConfig,
    is_git_repo: bool,
}

enum StartupError {
    ConfigParse { path: PathBuf, source: String },
    NotADirectory(PathBuf),
    TerminalInit(String),
}
```

## Events / Commands

| Event | Source | Action |
|-------|--------|--------|
| `AppEvent::StartupComplete` | Bootstrap | Enable input |
| `AppCommand::Quit` | User | Begin shutdown |
| `AppEvent::ShutdownComplete` | Bootstrap | Exit process |

## Configuration Options

Inherited from SPEC-018; startup reads full resolved config.

## Error Handling

| Condition | Behavior |
|-----------|----------|
| Invalid TOML | Print error with line number; exit 1 |
| Terminal too small (< 80×24) | Warning banner; continue |
| PTY spawn fail | Error modal; shell pane shows retry hint |
| Missing repo path | Exit 1 with message |

## Acceptance Criteria

- [ ] `kiwi .` opens TUI in current directory
- [ ] `kiwi /path/to/repo` opens with that root
- [ ] Invalid config prevents startup with clear error
- [ ] Terminal restored after quit (no broken echo)
- [ ] Workspace saved on quit when persistence enabled
- [ ] `--help` and `--version` work without TUI
