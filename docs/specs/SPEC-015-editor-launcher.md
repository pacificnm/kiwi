# SPEC-015: Editor Launcher

## Purpose

Resolve and spawn external editor for file paths per ADR-013.

## Scope

### In scope

- Command resolution chain
- Terminal suspend/resume for TUI editors
- Detached spawn for GUI editors
- Absolute path argument

### Out of scope

- Diff editor (git mergetool)

## Functional Requirements

1. Resolution order: config → `$VISUAL` → `$EDITOR` → `nano`.
2. Accept absolute `PathBuf`; error if not exists.
3. **GUI editors** (`code`, `cursor`, `zed`, …): spawn detached; Kiwi stays responsive.
4. **Terminal editors** (`vim`, `nvim`, `nano`, …): suspend Kiwi TUI, run editor on controlling TTY, wait for exit, resume Kiwi.
5. Optional `[editor] terminal` config overrides auto-detection.
6. Log launch to Logs tab at info level.
7. Optional line number: pass `+N` for vim family when `line` provided.
8. Palette command: "Open in Editor" uses current selection path.

## Non-Functional Requirements

- Spawn < 100ms
- No zombie processes (use appropriate spawn flags)

## Data Structures

```rust
struct EditorConfig {
    command: String,
    args_template: Option<String>,
}

struct EditorLauncher {
    resolved_command: OsString,
}
```

## Events / Commands

```rust
AppCommand::OpenEditor { path: PathBuf, line: Option<u32> }
AppEvent::EditorLaunched { path, command }
AppEvent::EditorLaunchFailed { path, error }
```

## Configuration Options

```toml
[editor]
command = "nvim"
```

## Error Handling

| Error | Behavior |
|-------|----------|
| Command not on PATH | Modal with resolution chain hint |
| File not found | Toast error |
| Spawn fail | Log + toast |

## Acceptance Criteria

- [x] `nvim` opens with file when configured
- [x] Falls back to EDITOR env
- [x] Falls back to nano when unset
- [x] Kiwi remains responsive after GUI editor launch
- [x] Terminal editors run on the TTY while Kiwi is suspended
- [ ] Works from file tree, preview, search results
