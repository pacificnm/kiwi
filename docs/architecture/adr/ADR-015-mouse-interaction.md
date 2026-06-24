# ADR-015: Mouse Interaction

## Status

Accepted

## Context

Terminal users expect both keyboard-first workflows and basic mouse affordances (click tab, scroll, select list item). Kiwi must not break terminal-native text selection (shift+drag, clipboard).

## Decision

Implement **hybrid mouse mode** via crossterm, configurable and enabled by default.

### Supported interactions

| Action | Behavior |
|--------|----------|
| Click tab | Activate left or main tab |
| Click list item | Select row; single-click |
| Double-click file | Open in editor |
| Click pane border/background | Focus pane |
| Scroll wheel | Scroll focused scrollable region |
| Paste | Middle-click or terminal paste → focused input (shell, palette, agent) |

### Not supported in v1

- Mouse text selection inside Kiwi widgets (use terminal shift+drag)
- Drag-to-resize panes (keyboard/config only for widths)
- Right-click context menus

### Configuration

```toml
[mouse]
enabled = true
mode = "hybrid"   # "hybrid" | "disabled"
```

When `enabled = false`, do not enable crossterm mouse capture.

### Terminal selection

Document that **terminal-native selection** remains primary for copying text from PTY and preview. Kiwi does not intercept shift+mouse events intended for terminal selection.

## Consequences

### Positive

- Lowers barrier for occasional mouse users
- Scroll wheel essential for long lists and PTY scrollback
- Configurable off for pure keyboard users

### Negative

- Mouse hit-testing complexity in ratatui
- Some terminals report inconsistent mouse events
- Risk of conflict with terminal selection (mitigate with hybrid mode docs)

## Alternatives Considered

| Alternative | Rejection Rationale |
|-------------|---------------------|
| No mouse support | Explicit product requirement |
| Full mouse selection in TUI | Fights terminal emulator; poor UX |
| SGR-only mouse | Too restrictive; crossterm abstracts |

## Follow-up Work

- SPEC-014 Mouse Support
- Design: [mouse-interaction.md](../../design/mouse-interaction.md)
- Test on Kitty, iTerm2, Alacritty, Windows Terminal
- Map wheel events to scroll offset in file tree, lists, PTY viewport
