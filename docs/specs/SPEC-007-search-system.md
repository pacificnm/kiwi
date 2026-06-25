# SPEC-007: Search System

## Purpose

Enable file name and content search across the repository with results navigable from the Search left nav tab.

## Scope

### In scope

- File name fuzzy search
- Content search via ripgrep subprocess
- Result list with selection → preview/editor

### Out of scope

- Symbol search (Phase 2)
- Replace across files

## Functional Requirements

1. Search input at top of Search left panel.
2. Modes: `Files` (default), `Content` (toggle `Ctrl+M`).
3. **File search**: fuzzy match relative paths; respect ignore defaults.
4. **Content search**: run `rg --json` or `rg -n` if json unavailable; stream results.
5. Debounce input 200ms before search execution.
6. `Enter` on result: open Preview; `e` open editor; content results jump to line.
7. Double-click result row: open Preview (same as `Enter`; ADR-015).
8. `Esc` clear query.
9. Cancel previous search when query changes.

## Non-Functional Requirements

- File search completes < 500ms on 50k file repo (with ignores)
- Content search shows first result < 1s on warm cache
- Max 10_000 results displayed; show truncation notice

## Data Structures

```rust
enum SearchMode { Files, Content }

struct SearchResult {
    id: String,           // stable: path or path:line
    path: PathBuf,
    line: Option<u32>,
    preview: String,      // match snippet for content
}

struct SearchState {
    mode: SearchMode,
    query: String,
    results: Vec<SearchResult>,
    selected: usize,
    running: bool,
    error: Option<String>,
}
```

## Events / Commands

```rust
AppCommand::SearchSetQuery(String)
AppCommand::SearchSetMode(SearchMode)
AppCommand::SearchExecute
AppCommand::SearchCancel
AppEvent::SearchResults(Vec<SearchResult>)
AppEvent::SearchError(String)
```

## Configuration Options

```toml
[search]
command = "rg"           # optional override
debounce_ms = 200
```

## Error Handling

| Error | Behavior |
|-------|----------|
| `rg` not found | Show install hint; disable content mode |
| `rg` exit 1 (no matches) | Empty results, not error |
| `rg` exit 2 | Show error message |

## Acceptance Criteria

- [x] Fuzzy file find returns expected paths
- [x] Content search finds string in repo
- [x] Debounce prevents search storm while typing
- [x] Selection opens correct file/line
- [x] Double-click result opens Preview at file/line
- [x] Ignore dirs excluded from file search
