# Mouse Interaction Design

## Goals

Provide **lightweight mouse support** that complements keyboard workflows without breaking terminal-native text selection.

## Supported Interactions

| Interaction | Target | Result |
|-------------|--------|--------|
| Single click | Tab label | Activate tab |
| Single click | List/tree row | Select item |
| Double click | File row | Open external editor |
| Single click | Pane background | Focus pane |
| Scroll wheel | Scrollable under cursor | Scroll content |
| Middle click | Focused input area | Paste (terminal convention) |

## Unsupported (v1)

- Drag to resize panes
- Right-click context menus
- In-widget text selection for copy
- Click-and-drag scrolling (wheel only)

## Terminal Text Selection

Users copy from PTY and preview using **terminal emulator selection**:

- **Shift + drag** to select text in most terminals
- Selection copies to system clipboard per terminal settings
- Kiwi hybrid mode must not capture shift+mouse for widget hit-testing

Document in help: "Use terminal selection to copy from shell and preview."

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

## Platform Notes

| Terminal | Notes |
|----------|-------|
| Kitty | Full SGR mouse; excellent scroll |
| Alacritty | Test double-click timing |
| iTerm2 | macOS; verify paste |
| Windows Terminal | Enable mouse in settings |

## Related

- ADR-015 Mouse Interaction
- SPEC-014 Mouse Support
