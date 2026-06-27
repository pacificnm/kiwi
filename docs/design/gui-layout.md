# GUI Layout Design

Visual and interaction design for the `kiwi_gui` desktop frontend. Behavioral contracts are in SPEC-021–023 and ADR-022.

## Design Goals

1. **IDE familiarity** — Draggable tabs and splits like VS Code; users should recognize the pattern immediately.
2. **Orchestrator, not editor** — Same philosophy as the TUI: Kiwi coordinates tools; external editor for heavy editing.
3. **Density with clarity** — Reuse TUI design principles; more horizontal space for diffs and issues.
4. **Mouse-first, keyboard complete** — Point-and-click for exploration; power users keep palette and shortcuts.

## Chrome Layout

```text
┌─────────────────────────────────────────────────────────────────────────┐
│ File   View   Git   Help                                    ─  □  ×   │  menu bar
├─────────────┬───────────────────────────────────────────────────────────┤
│ Explorer    │ Agent                                                     │
│ Git Status  │                                                           │
│ GitHub      │              (active dock panel)                          │
│             │                                                           │
├─────────────┴───────────────────────────────────────────────────────────┤
│ Terminal                                                                │
├─────────────────────────────────────────────────────────────────────────┤
│ main ● feature/foo │ 2 modified │ agent: idle │ catppuccin-mocha       │  status bar
└─────────────────────────────────────────────────────────────────────────┘
```

### Regions

| Region | Height / width | Notes |
|--------|----------------|-------|
| Menu bar | 28px | Native menus; accelerators shown |
| Dock area | Remaining | egui_dock manages internal splits |
| Status bar | 24px | Same information density as SPEC-019 |

## Default Dock Tree

First-run layout (matches ADR-022):

- **Left column (~22% width):** tab stack Explorer → Git Status → GitHub Issues
- **Center (~78%):** Agent
- **Bottom strip (~28% height):** Terminal

Proportions are hints; egui_dock stores actual splits in persistence.

## Panel Design Notes

### Explorer

- Tree view with lazy expand (SPEC-005)
- Git status icons inline (color from semantic roles)
- Double-click opens external editor (SPEC-015)
- Single-click selects and loads Preview if open

### Git Status

- Table: path, status badge, staged/unstaged
- Click row opens Diff tab with file selected

### Diff

- Side-by-side or unified toggle
- Syntax-colored hunks (SPEC-012 semantics)
- Horizontal scroll for long lines

### GitHub Issues / PRs

- List in **GitHub Issues** left dock tab; detail in center **Issues** / **GitHub PRs** tabs
- Right-click a list row for **View**, **Create Branch** (issues only), or **Send To Agent** (parity with TUI #193)
- Replaces TUI pattern of GH left nav + Issues main tab

### Terminal / Agent

- Monospace scrollback buffer
- Distinct tab icons; agent shows status chip (idle / thinking / error)
- Copy/paste via system clipboard (ADR-019 GUI path)

### Preview

- Read-only; line numbers optional
- "Open in editor" prominent button

### Search

- Fuzzy file find + ripgrep results in tabs or stacked sections

### Command palette

- Centered modal, 600px wide, fuzzy filter (SPEC-013 commands)
- `Ctrl+Shift+P` default; does not occupy dock slot

## TUI vs GUI Feature Mapping

| TUI concept | GUI equivalent |
|-------------|----------------|
| Left nav tabs | Left dock stack or user-moved tabs |
| Main tabs | Center dock tabs |
| Bottom shell | Terminal dock tab (default bottom) |
| Command palette row | Modal overlay |
| Status bar | Bottom chrome bar |
| Focus: left/main/shell | Active dock tab + palette focus |

## Visual Style

- Use SPEC-023 theme bridge; no separate GUI-only palette
- Rounded corners: subtle (egui default, 4px)
- Panel padding: 8px
- Tab bar: icons + text for primary tabs (Explorer, Terminal, Agent)
- Spacing between dock splits: 4px draggable handle

## Responsive Behavior

| Window width | Behavior |
|--------------|----------|
| < 900px | Collapse GitHub detail to stacked view |
| < 800px | Min size; horizontal scroll in panels |
| Maximized | Persist in workspace JSON |

## Empty States

| Panel | Empty message |
|-------|---------------|
| Explorer | "Open a repository to browse files" |
| Git Status | "Not a git repository" or "Clean working tree" |
| GitHub | "Run `gh auth login`" with link button |
| Agent | "Start agent" button |
| Terminal | Shell ready prompt |

## Related

- [gui-implementation-plan.md](./gui-implementation-plan.md)
- [gui-keyboard-shortcuts.md](./gui-keyboard-shortcuts.md)
- [layout.md](./layout.md) — TUI layout reference
- ADR-022, SPEC-022
