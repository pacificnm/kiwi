# ADR-010: Git Integration

## Status

Accepted

## Context

Git status drives file tree badges, left Git panel, diff navigation, status bar, and issue-branch workflows. Status must stay current without polling, which wastes CPU and causes UI flicker.

## Decision

Use **libgit2 via git2 crate** for read operations and **notify-based file watcher** (ADR-011) for refresh triggers.

### Displayed data

| Field | Source |
|-------|--------|
| Current branch | `git2` |
| Ahead/behind upstream | `git2` revwalk |
| Modified, added, deleted, untracked | `git2` status |
| Diff content | `git2` diff or `git diff` for unified view |

### Refresh strategy

**No polling.** Refresh on:

1. File watcher debounced event (default 300ms)
2. User explicit refresh command (`R` in Git panel)
3. Post-command hooks (after commit via shell — detect via watcher)

### Incremental updates

Compare previous `GitState` to new snapshot; emit patch for changed paths only. Preserve list selection by path.

### Configuration

```toml
[git]
watch = true
show_untracked = true
```

## Consequences

### Positive

- git2 is fast and embeddable
- Event-driven updates align with flicker-free goals
- Works offline

### Negative

- git2 may lag newest Git features (e.g., some worktree edge cases)
- Watcher may miss events on some network filesystems (document limitation)
- Write operations (commit) initially via shell, not libgit2 UI

## Alternatives Considered

| Alternative | Rejection Rationale |
|-------------|---------------------|
| Parse `git status` subprocess only | Slower; harder incremental parsing |
| Poll every N seconds | Forbidden by product requirements |
| Jujutsu (jj) support | Out of MVP scope; plugin future |

## Follow-up Work

- SPEC-008 Git Service, SPEC-012 Diff Viewer
- Color mapping per ADR-004 semantic roles
- Branch create from issue links to GitHub service
- Document unsupported: submodules UI (read-only status only in v1)
