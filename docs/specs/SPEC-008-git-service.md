# SPEC-008: Git Service

## Purpose

Provide accurate, incrementally updated Git repository status via libgit2 and file watcher triggers.

## Scope

### In scope

- Branch, ahead/behind, file status lists
- Refresh on watcher + manual
- Git panel data model

### Out of scope

- Commit UI (use shell)
- Merge/rebase UI

## Functional Requirements

1. Detect git repo at startup; disable git features if absent.
2. Display: current branch, upstream ahead/behind counts, modified/added/deleted/untracked lists.
3. Refresh triggered by debounced watcher (ADR-011) when `[git] watch = true`.
4. Manual refresh via command palette and `R` in Git panel.
5. Incremental patch updates to `GitState`; preserve selection by path.
6. Respect `show_untracked` config.
7. Emit events for status bar summary (modified count).

## Non-Functional Requirements

- Status refresh < 100ms for repos with < 5000 tracked files
- No polling loops
- Thread-safe git operations via `spawn_blocking`

## Data Structures

```rust
struct GitFileEntry {
    path: PathBuf,
    status: GitFileStatus,  // Modified, Added, Deleted, Untracked, Renamed
}

struct GitState {
    branch: String,
    ahead: u32,
    behind: u32,
    entries: Vec<GitFileEntry>,
    selected: Option<PathBuf>,
    scroll_offset: usize,
    last_refresh: Instant,
    loading: bool,
}
```

## Events / Commands

```rust
AppCommand::GitRefresh
AppEvent::GitStatusUpdated(GitStatePatch)
AppEvent::FsChanged { paths }
```

## Configuration Options

```toml
[git]
watch = true
show_untracked = true
```

## Error Handling

| Error | Behavior |
|-------|----------|
| git2 open fail | Git panels show "not a git repository" |
| Corrupt repo | Error banner with message |
| Watcher fail | Disable watch; show warning once |

## Acceptance Criteria

- [x] Branch name correct
- [x] Modified files listed with correct status
- [x] Saving file updates status within debounce window
- [x] No visible flicker on refresh (selection stable)
- [x] Untracked hidden when config false

## Git panel UI

- [x] Branch and ahead/behind shown in Git left tab
- [x] Files grouped by status with badge and path
- [x] j/k selection with scroll preservation
- [x] R refreshes git status from Git tab
- [x] Enter selects file and opens main Diff tab

## Manual refresh

- [x] Command palette "Git: Refresh Status" triggers refresh in git repos
- [x] Manual refresh sets `git.loading` until `GitStatusUpdated` arrives
- [x] Palette command hidden from execution when not in a git repo
