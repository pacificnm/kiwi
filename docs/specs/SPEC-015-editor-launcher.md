# SPEC-015: Editor Launcher

## Purpose

Resolve and spawn external editor for file paths per ADR-013.

## Scope

### In scope

- Command resolution chain
- Detached spawn
- Absolute path argument

### Out of scope

- Wait for editor close
- Diff editor (git mergetool)

## Functional Requirements

1. Resolution order: config → `$VISUAL` → `$EDITOR` → `nano`.
2. Accept absolute `PathBuf`; error if not exists.
3. Spawn detached process; do not block TUI.
4. Log launch to Logs tab at info level.
5. Optional line number: pass `+N` for vim family when `line` provided.
6. Palette command: "Open in Editor" uses current selection path.

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

- [ ] `nvim` opens with file when configured
- [ ] Falls back to EDITOR env
- [ ] Falls back to nano when unset
- [ ] Kiwi remains responsive after launch
- [ ] Works from file tree, preview, search results
