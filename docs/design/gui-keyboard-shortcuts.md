# GUI Keyboard Shortcuts

Default keybindings for `kiwi_gui`. Terminal users can map familiar TUI bindings where they apply.

## Global

| Action | Default | Notes |
|--------|---------|-------|
| Quit | `Ctrl+Q` | |
| Command palette | `Ctrl+Shift+P` | Also `Ctrl+K` optional alias |
| Toggle menu | `F10` | |
| Next tab (active dock) | `Ctrl+Tab` | |
| Previous tab | `Ctrl+Shift+Tab` | |

## View

| Action | Default |
|--------|---------|
| Show Explorer | `Ctrl+Shift+E` |
| Show Terminal | `Ctrl+`` ` |
| Show Agent | `Ctrl+Shift+A` |
| Show Search | `Ctrl+Shift+F` |
| Reset layout | — (menu only v1) |

## Explorer

| Action | Default | TUI equivalent |
|--------|---------|----------------|
| Open in editor | `Enter` | `e` |
| Preview file | `Space` | preview tab switch |
| Collapse folder | `Left` | |
| Expand folder | `Right` | |

## Git / Diff

| Action | Default |
|--------|---------|
| Refresh status | `F5` |
| Open diff for selected | `Enter` |

## Terminal / Agent (when tab focused)

| Action | Default |
|--------|---------|
| Copy | `Ctrl+Shift+C` |
| Paste | `Ctrl+Shift+V` |
| Scroll page up | `Shift+PageUp` |
| Scroll page down | `Shift+PageDown` |

PTY typing passes through when no egui widget holds focus.

## GitHub

| Action | Default |
|--------|---------|
| Refresh issues | `F5` |
| Open in browser | `Ctrl+Enter` |

## Configuration

Shortcuts are not yet user-remappable in v1. Future: `[gui.keybindings]` in config.

## Related

- [keyboard-shortcuts.md](./keyboard-shortcuts.md) — TUI reference
- SPEC-022 GUI Dock Layout Engine
