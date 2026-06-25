# SPEC-006: File Preview

## Purpose

Provide read-only file content preview in the main Preview tab for quick inspection without launching an editor.

## Scope

### In scope

- Text file preview with syntax-free plain rendering
- Line numbers optional
- Scroll and wrap modes
- Auto-load on file selection from tree/search

### Out of scope

- Syntax highlighting (future tree-sitter)
- Binary file preview (show hex summary or "binary file" message)
- Editing

## Functional Requirements

1. Display selected file content in Preview tab; switch to Preview tab on explicit preview action.
2. Max file size for preview: 1 MiB; larger files show truncation message with path and size.
3. Line numbers in gutter (muted color).
4. Scroll with `j`/`k`, `PgUp`/`PgDn`, mouse wheel when focused.
5. Reload on file watcher event if same file changed.
6. Status line: path, line count, encoding assumption (UTF-8 with lossy fallback).
7. `e` opens file in external editor at current line (if line tracking supported).
8. Copy via `Ctrl+C` or mouse selection + `Ctrl+C` (ADR-019).

## Non-Functional Requirements

- Load and render 10k lines < 300ms
- Virtualized rendering for large files (viewport only)
- No full-file string allocation for files > 256 KiB if possible (chunked read)

## Data Structures

```rust
struct PreviewState {
    path: Option<PathBuf>,
    lines: Vec<String>,       // or rope/chunks for large files
    scroll_offset: usize,
    cursor_line: usize,
    truncated: bool,
    load_error: Option<String>,
}
```

## Events / Commands

```rust
AppCommand::PreviewFile(PathBuf)
AppCommand::PreviewScroll(i32)
AppEvent::PreviewLoaded { path, result }
AppEvent::FsChanged { paths }  // reload if matches
```

## Configuration Options

```toml
[preview]
max_size_bytes = 1048576
line_numbers = true
wrap = false
```

## Error Handling

| Error | Behavior |
|-------|----------|
| File not found | Show error in preview pane |
| Permission denied | Show error message |
| Non-UTF8 | Lossy replacement; indicator in status |

## Acceptance Criteria

- [x] Selecting file + preview shows content
- [x] Scrolling works for 1000+ line file
- [ ] Watcher reload updates content without losing scroll if possible
- [x] Binary file shows appropriate message
- [x] `e` launches editor
