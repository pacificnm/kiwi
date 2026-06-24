# SPEC-012: Diff Viewer

## Purpose

Display unified diffs for working tree and staged changes with navigation from Git/Diff panels.

## Scope

### In scope

- Per-file unified diff
- Side-by-side optional (future flag; unified default)
- Syntax-colored lines (added/removed/context)
- Navigation between changed files

### Out of scope

- Stage/unstage hunks (use shell)
- Merge conflict resolution UI

## Functional Requirements

1. Left Diff nav: list of changed files from `GitState`.
2. Main Diff tab: unified diff for selected file.
3. Colors: added green, removed red, context muted (theme git roles).
4. Navigate files: `n`/`p` next/previous changed file.
5. Horizontal scroll for long lines; line numbers in gutter.
6. Support staged vs unstaged toggle (`s` key).
7. Large diff virtualization: render viewport ± 100 lines.

## Non-Functional Requirements

- 5000-line diff scrolls smoothly
- Diff compute via git2 off main thread
- Preserve scroll when switching files and returning

## Data Structures

```rust
enum DiffSource { Unstaged, Staged }

struct DiffHunk {
    old_start: u32,
    new_start: u32,
    lines: Vec<DiffLine>,
}

enum DiffLineKind { Context, Addition, Deletion, Header }

struct DiffState {
    file: Option<PathBuf>,
    hunks: Vec<DiffHunk>,
    source: DiffSource,
    scroll_offset: usize,
    loading: bool,
}
```

## Events / Commands

```rust
AppCommand::DiffSelectFile(PathBuf)
AppCommand::DiffSetSource(DiffSource)
AppCommand::DiffNextFile / DiffPrevFile
AppEvent::DiffLoaded { path, hunks }
```

## Configuration Options

```toml
[diff]
context_lines = 3
word_wrap = false
```

## Error Handling

| Error | Behavior |
|-------|----------|
| Binary file | Show "binary diff not supported" |
| Diff fail | Show git2 error message |

## Acceptance Criteria

- [ ] Modified file shows correct +/- lines
- [ ] Colors match theme
- [ ] File list matches git status
- [ ] Staged/unstaged toggle works
- [ ] Large diff does not freeze UI
