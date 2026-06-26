# SPEC-014: Mouse Support

## Purpose

Implement configurable mouse interactions per ADR-015 without breaking terminal-native text selection.

## Scope

### In scope

- Click, focus, tab switching, list/tree selection
- Double-click preview from Files tree and Search results
- In-app text selection (left drag) in Preview, Agent, Shell
- crossterm mouse capture
- Shift+mouse passthrough for terminal selection

### Out of scope

- Drag resize
- Right-click context menus (planned)
- Native double-click events (synthesized via click timing)

## Functional Requirements

1. Enable mouse when `[mouse] enabled = true`.
2. `mode = "hybrid"`: handle clicks and scroll in Kiwi regions; shift+selection handled by terminal.
3. `mode = "disabled"`: no mouse capture.
4. Map click coordinates to widget via layout rects: tabs, lists, panes.
5. Wheel scroll: adjust scroll offset of the scrollable content in the pane under the cursor (3 lines per tick); fall back to the focused pane when the cursor is not over a scrollable region.
6. **Single click** on file tree or search row: select row.
7. **Double click** (500ms, same target) on file tree file or search result: open Preview tab (`Enter` / `p` keyboard equivalents); double-click directory expands.
8. **Left drag** in Preview, Agent, or Shell: text selection for clipboard copy (ADR-019).
9. Middle-click / `Event::Paste`: route to focused input (shell, palette, agent) per ADR-019.

## Non-Functional Requirements

- Hit test < 1ms
- Double-click tracker clears after successful double-click

## Data Structures

```rust
struct DoubleClickTracker {
    last: Option<(DoubleClickTarget, Instant)>,
}

enum DoubleClickTarget {
    FileTree(PathBuf),
    SearchResult(usize),
}

struct TextSelection {
    pane: Option<SelectionPane>,
    anchor: TextPosition,
    cursor: TextPosition,
    dragging: bool,
}
```

## Events / Commands

```rust
AppCommand::SelectionBegin { pane, line, col }
AppCommand::SelectionExtend { line, col }
AppCommand::SelectionEnd
AppCommand::SelectionClear
// Crossterm → app loop
Event::Mouse(MouseEvent)
```

## Configuration Options

```toml
[mouse]
enabled = true
mode = "hybrid"
```

## Error Handling

- Unsupported terminal: disable mouse silently with log info
- Click outside known regions: no-op

## Acceptance Criteria

- [x] Click switches tabs and moves focus
- [x] Single click selects file tree row
- [x] Single click selects search result row
- [x] Double-click file opens Preview tab
- [x] Double-click search result opens Preview at line (content hits)
- [x] Left drag highlights text in Preview, Agent, Shell
- [x] Shift+drag not captured by Kiwi (terminal selection)
- [x] Wheel scrolls file tree when over left panel
- [x] Mouse disabled via config
- [ ] Terminal shift+drag selection still copies text from PTY (manual terminal check)
