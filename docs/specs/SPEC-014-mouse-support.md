# SPEC-014: Mouse Support

## Purpose

Implement configurable mouse interactions per ADR-015 without breaking terminal text selection.

## Scope

### In scope

- Click, scroll, focus, paste routing
- crossterm mouse capture

### Out of scope

- In-app text selection
- Drag resize

## Functional Requirements

1. Enable mouse when `[mouse] enabled = true`.
2. `mode = "hybrid"`: handle clicks and scroll in Kiwi regions; shift+selection handled by terminal.
3. `mode = "disabled"`: no mouse capture.
4. Map click coordinates to widget via layout rects: tabs, lists, panes.
5. Wheel scroll: adjust scroll offset of focused scrollable under cursor (or focused pane if not hit).
6. Double-click file in tree: open editor.
7. Middle-click paste: send to focused input (shell, palette, agent).

## Non-Functional Requirements

- Hit test < 1ms
- No duplicate click events

## Data Structures

```rust
struct MouseState {
    enabled: bool,
    mode: MouseMode,
    last_click: Option<(Instant, RectId)>,
}

enum MouseMode { Hybrid, Disabled }
```

## Events / Commands

```rust
// Crossterm → translated
AppEvent::MouseClick { x, y, button }
AppEvent::MouseScroll { x, y, delta }
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

- [ ] Click switches tabs
- [ ] Wheel scrolls file tree when over left panel
- [ ] Double-click opens editor
- [ ] Mouse disabled via config
- [ ] Terminal shift+drag selection still copies text from PTY
