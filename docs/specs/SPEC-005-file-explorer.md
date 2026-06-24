# SPEC-005: File Explorer

## Purpose

Display a lazy-loaded directory tree with expand/collapse, selection, ignore rules, and editor launch integration.

## Scope

### In scope

- Tree view in Files left nav tab
- Lazy directory reads
- Ignore defaults
- Watcher-driven cache invalidation

### Out of scope

- File create/delete/rename in Kiwi
- Multi-select

## Functional Requirements

1. Show repository root as tree root; sorted dirs first, then files, case-insensitive.
2. Expand/collapse directories; lazy-load children on first expand.
3. Skip default ignored names: `.git`, `node_modules`, `target`, `dist`, `build`, `.next`, `.nuxt`, `.venv`.
4. Selection: single row; `Enter` or `e` opens editor (SPEC-015); `p` opens Preview tab with file.
5. `r` refresh tree preserving expansion set.
6. Visual git status badge per file when git repo (color per theme).
7. Watcher invalidates affected directory caches (ADR-011).

## Non-Functional Requirements

- Initial render < 100ms (root only)
- Expand directory < 200ms for < 1000 entries
- Preserve scroll/selection on git status update

## Data Structures

```rust
struct FileNode {
    path: PathBuf,
    name: String,
    is_dir: bool,
    expanded: bool,
    children_loaded: bool,
    git_status: Option<GitFileStatus>,
}

struct FileTreeState {
    root: PathBuf,
    nodes: HashMap<PathBuf, FileNode>,
    children: HashMap<PathBuf, Vec<PathBuf>>,
    selected: Option<PathBuf>,
    scroll_offset: usize,
}
```

## Events / Commands

```rust
AppCommand::FileTreeExpand(PathBuf)
AppCommand::FileTreeCollapse(PathBuf)
AppCommand::FileTreeSelect(PathBuf)
AppCommand::FileTreeRefresh
AppEvent::FileTreeChildrenLoaded { parent, children }
AppEvent::FsChanged { paths }
```

## Configuration Options

```toml
[git]
show_untracked = true
```

## Error Handling

| Error | Behavior |
|-------|----------|
| Permission denied on read | Show error icon on node; skip children |
| Symlink loop | Detect; stop at depth 40 |

## Acceptance Criteria

- [ ] Large repo opens instantly showing root only
- [ ] Expand loads children
- [ ] Ignored dirs not shown
- [ ] Selection survives git status refresh
- [ ] Double-click opens editor (mouse)
- [ ] Git colors match theme roles
