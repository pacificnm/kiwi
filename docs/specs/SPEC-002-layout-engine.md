# SPEC-002: Layout Engine

## Purpose

Compute pane dimensions and provide a widget tree for each frame based on terminal size and workspace state.

## Scope

### In scope

- Region layout per ADR-003
- Minimum size enforcement
- Resize handling
- Focus-aware border highlighting

### Out of scope

- Mouse drag resize (future)
- Floating windows

## Functional Requirements

1. Divide terminal into: status bar (1 row), left panel, main panel, bottom panel.
2. Left panel width from `app.left_width` (percent 10–50, default 30).
3. Bottom panel height: max(5 rows, 25% of content area below tab bars).
4. Tab bars: 1 row each for left nav and main tabs.
5. On resize: recompute `Rect`s; emit `AppEvent::LayoutChanged`; resize PTYs.
6. Focused pane draws accent border using theme `accent` color.
7. Clip content to pane bounds; no bleed between regions.

## Non-Functional Requirements

- Layout computation < 1ms
- Deterministic rects for same terminal size and config
- Support terminal sizes 80×24 through 500×200

## Data Structures

```rust
struct LayoutRects {
    status_bar: Rect,
    left_tabs: Rect,
    left_content: Rect,
    main_tabs: Rect,
    main_content: Rect,
    palette: Rect,
    shell: Rect,
}

struct LayoutState {
    rects: LayoutRects,
    terminal_size: (u16, u16),
}
```

## Events / Commands

| Event | Action |
|-------|--------|
| `CrosstermEvent::Resize(w, h)` | Recompute layout, mark dirty |
| `AppCommand::SetLeftWidth(u8)` | Update width, recompute |

## Configuration Options

```toml
[app]
left_width = 30   # percent
```

## Error Handling

- If height < minimum: render single error screen with message, skip normal layout.

## Acceptance Criteria

- [ ] All regions visible at 120×40
- [ ] Degrades gracefully at 80×24 (no panic)
- [ ] Resize updates PTY cols/rows for shell pane
- [ ] Focus border moves when focus changes
- [ ] Left width config reflected on startup
