# ADR-015: Mouse Interaction

## Status

Accepted (amended 2026-06 — in-app selection, double-click preview)

## Context

Terminal users expect both keyboard-first workflows and basic mouse affordances (click tab, scroll, select list item). Kiwi must not break terminal-native text selection (shift+drag) where users still prefer it, while supporting in-app highlight for copy in scrollable panes.

Terminals do not emit native double-click events; Kiwi synthesizes double-clicks from click timing.

## Decision

Implement **hybrid mouse mode** via crossterm, configurable and enabled by default.

### Supported interactions

| Action | Behavior |
|--------|----------|
| Click tab | Activate left or main tab; move focus to that pane |
| Click list/tree row | Select row (single click) |
| Double-click file (Files tab) | Open **Preview** tab; directories expand |
| Double-click search result | Open **Preview** tab at result path/line |
| Click agent / shell pane | Focus Main or Shell |
| Left drag (no Shift) | Highlight text in Preview, Agent, or Shell for `Ctrl+C` copy |
| Scroll wheel | Scroll focused scrollable region (when implemented) |
| Paste | Middle-click or `Event::Paste` → focused input (see ADR-019) |

### Double-click detection

- `DoubleClickTracker` in `ui/mouse_clicks.rs`
- 500ms window; same logical target (file path or search result index)
- Second click on same file row or search row triggers preview navigation

### In-app text selection

- Panes: **Preview**, **Agent**, **Shell** scrollback
- Left mouse down + drag extends selection; release ends drag
- Shift+mouse is **not** captured (terminal-native selection still available)
- Selection highlight uses theme `selection` role; copy via ADR-019 clipboard routing
- PTY rows without an active selection keep ANSI colors; selected rows render on stripped plain text

### Not supported in v1

- Drag-to-resize panes (keyboard/config only for widths)
- Right-click context menus (planned)
- Mouse wheel scroll in all panes (partial / follow-up)
- Double-click to open external editor (use `e` or palette)

### Configuration

```toml
[mouse]
enabled = true
mode = "hybrid"   # "hybrid" | "disabled"
```

When `enabled = false`, do not enable crossterm mouse capture.

### Terminal selection coexistence

- **In-app**: left-click + drag in Preview, Agent, Shell → `Ctrl+C` to system clipboard (ADR-019).
- **Terminal-native**: Shift + drag still available when the terminal handles it; Kiwi ignores Shift+mouse for widget hit-testing.

## Consequences

### Positive

- Lowers barrier for occasional mouse users
- Double-click preview matches file-manager expectations
- In-app selection enables copy from PTY panes without fighting the host terminal
- Configurable off for pure keyboard users

### Negative

- Mouse hit-testing complexity in ratatui
- Some terminals report inconsistent mouse events; double-click timing varies (test Alacritty)
- Two selection models (in-app vs shift+terminal) require documentation

## Alternatives Considered

| Alternative | Rejection Rationale |
|-------------|---------------------|
| No mouse support | Explicit product requirement |
| Terminal-only selection for PTY | Poor UX for copy from agent/shell panes |
| Double-click opens editor | Preview is the primary inspect action; editor remains `e` |
| SGR-only mouse | Too restrictive; crossterm abstracts |

## Follow-up Work

- SPEC-014 Mouse Support
- Design: [mouse-interaction.md](../../design/mouse-interaction.md)
- Right-click context menu for copy/paste
- Mouse wheel scroll in file tree, search, preview, PTY viewport
- Test on Kitty, iTerm2, Alacritty, Windows Terminal
