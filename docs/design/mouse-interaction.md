# Mouse Interaction Design

## Goals

Provide **lightweight mouse support** that complements keyboard workflows without breaking terminal-native text selection.

## Supported Interactions

| Interaction | Target | Result |
|-------------|--------|--------|
| Single click | Left tab label | Activate left tab and focus Left pane |
| Single click | Main tab label | Activate main tab and focus Main pane |
| Single click | Agent pane (Agent tab) | Focus Main pane |
| Single click | Shell pane | Focus Shell pane |
| Single click | GH issue row (Issues hub) | Select issue |
| Double click | GH issue row (Issues hub) | Open Issues main tab detail |
| Single click | List/tree row | Select item |
| Double click | File row (Files tab) | Open Preview tab |
| Double click | Search result row | Open Preview tab at result line |
| Scroll wheel | Scrollable under cursor | Scroll content |
| Left drag | Preview, Issues detail, Agent, Shell text | Highlight selection for copy |
| Middle click | Focused input area | Paste (terminal convention) |

## Unsupported (v1)

- Drag to resize panes
- Right-click context menus (planned)
- Click-and-drag scrolling (wheel only)

## Terminal Text Selection

Kiwi supports **in-app text selection** in Preview, Issues detail, Agent, and Shell panes: left-click and drag to highlight, then `Ctrl+C` to copy. Paste into Agent or Shell with `Ctrl+V` when that pane is focused.

For hybrid mode, **Shift + drag** can still be used for terminal-native selection when needed.

## Focus vs Hover

- No hover highlight in v1 (terminals lack reliable hover)
- Click implies focus + action (select row)

## Scroll Behavior

Wheel events route to:

1. Widget under cursor if scrollable
2. Else focused pane's scrollable content
3. Else no-op

Scroll step: 3 lines per wheel tick (configurable future).

## Configuration

```toml
[mouse]
enabled = true
mode = "hybrid"   # or "disabled"
```

**Hybrid**: Kiwi handles clicks/scroll; terminal handles shift+selection.

**Disabled**: No mouse capture; pure keyboard.

### Double-click timing

Terminals report repeated `MouseEventKind::Down` events, not a distinct double-click. Kiwi uses `DoubleClickTracker` (`ui/mouse_clicks.rs`) with a **500ms** window and matching target (file path or search index). Test on Alacritty if double-click feels sluggish.

## Platform Notes

| Terminal | Notes |
|----------|-------|
| Kitty | Full SGR mouse; excellent scroll |
| Alacritty | Test double-click timing |
| iTerm2 | macOS; verify paste |
| Windows Terminal | Enable mouse in settings |

## Related

- ADR-015 Mouse Interaction
- ADR-019 System Clipboard Integration
- SPEC-014 Mouse Support
