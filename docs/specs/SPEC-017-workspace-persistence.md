# SPEC-017: Workspace Persistence

## Purpose

Save and restore per-repository UI state across sessions per ADR-016.

## Scope

### In scope

- JSON state file in XDG state dir
- Load on startup, save on quit and periodic

### Out of scope

- Cloud sync
- PTY session restore

## Functional Requirements

1. Compute `repo_hash` from canonical repo path.
2. Load `~/.local/state/kiwi/workspaces/<repo_hash>.json` if exists.
3. Apply saved: tabs, focus, left_width, expanded_paths, selected_path, scroll_positions, palette history.
4. Save on `AppCommand::Quit` and every 30s debounced.
5. Atomic write via temp file + rename.
6. `schema_version` field; unknown version migrates or resets with warning.
7. Config `workspace.persist = false` disables load/save.

## Non-Functional Requirements

- Save < 50ms
- State file < 100 KiB typical

## Data Structures

```rust
struct WorkspaceSnapshot {
    schema_version: u32,
    left_nav_tab: String,
    main_tab: String,
    left_width: u8,
    expanded_paths: Vec<String>,
    selected_path: Option<String>,
    scroll_positions: HashMap<String, usize>,
    palette_history: Vec<String>,
}
```

## Events / Commands

```rust
AppCommand::SaveWorkspace
AppEvent::WorkspaceLoaded(WorkspaceSnapshot)
AppEvent::WorkspaceSaved
```

## Configuration Options

```toml
[workspace]
persist = true
save_interval_secs = 30
```

## Error Handling

| Error | Behavior |
|-------|----------|
| Corrupt JSON | Log warning; defaults |
| Permission denied on save | Log error; continue session |
| Missing file | Fresh state |

## Acceptance Criteria

- [ ] Restart restores last tabs and selection
- [ ] Expanded folders remain expanded
- [ ] Corrupt file does not crash
- [ ] persist=false skips save
- [ ] Two repos have isolated state files
