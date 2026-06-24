# Layout Design

## Overview

Kiwi uses a **fixed three-band layout**: header tab rows, content area, and bottom split (palette + shell), plus a global status bar. The layout optimizes for simultaneous visibility of navigation context (left) and active work (main), with always-available shell access.

## Region Diagram

```text
 Row 0-1:  [ Left Tab Bar  |  Main Tab Bar                    ]
 Row 2-N:  [ Left Content  |  Main Content                    ]
 Row N+1:  [ Command Pal.  |  Shell PTY                       ]
 Last row: [ Status Bar (full width)                          ]
```

### Proportions (default terminal 120×40)

| Region | Width | Height |
|--------|-------|--------|
| Left panel | 30% (~36 cols) | Content + tab bar |
| Main panel | 70% | Content + tab bar |
| Bottom band | 100% | ~25% of rows below tabs (~8 rows) |
| Status bar | 100% | 1 row |

Minimum terminal: **80×24**. Below minimum, show warning; collapse bottom panel to 5 rows minimum.

## Left Panel

### Tab bar

Horizontal tabs: **Files | Git | Diff | GH | Search**

- Active tab: accent underline + bold label
- Inactive: muted foreground
- Mouse: click to switch

### Content area

Scrollable list or tree depending on tab:

| Tab | Widget |
|-----|--------|
| Files | Tree view with indent guides |
| Git | Flat list grouped by status |
| Diff | Changed files list |
| GH | Quick links + issue/PR counts |
| Search | Input + results list |

Left content scrolls independently of main content.

## Main Panel

### Tab bar

**Agent | Issues | PRs | Diff | Preview | Logs**

- Agent: full PTY viewport (ANSI rendered)
- Issues/PRs: master-detail or list + detail split within main (list 40% / detail 60% when width ≥ 100)
- Diff: unified diff with gutter
- Preview: read-only buffer with line numbers
- Logs: structured app log lines (timestamp, level, message)

## Bottom Panel

### Left: Command palette

- Single-line input when closed shows hint: `Ctrl+P for commands`
- When open: grows to show up to 10 results above input
- Border accent when active

### Right: Shell

- PTY output fills area
- Title in border: `Shell: bash` (or detected shell)
- Scrollback when focused

## Status Bar

Single line, inverted or muted background. Never scrolls away.

Format: `Kiwi | repo | branch | agent | git | issue`

## Focus Indicators

- Focused pane: `accent` colored border (double line optional)
- Unfocused: `border` muted
- Tab bars: focus does not require tab bar focus for `Alt+1` style shortcuts

## Resize Behavior

- Terminal resize reflows all rects immediately
- PTY receives new rows/cols for shell region
- No horizontal scrollbar on layout; panes clip content

## Future Enhancements

- Draggable vertical split for left width
- Draggable horizontal split for bottom height
- Collapse bottom panel shortcut (`Ctrl+_`)

## Related

- [navigation.md](./navigation.md)
- SPEC-002 Layout Engine
