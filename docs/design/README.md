# Design Documentation

UX and interaction design for Kiwi. These documents describe how the product should look and feel; specifications define implementable contracts.

## Documents

| Document | Description |
|----------|-------------|
| [layout.md](./layout.md) | Overall TUI layout, regions, and proportions |
| [navigation.md](./navigation.md) | Tab model, focus, and pane behavior |
| [../agents/pty-pipeline.md](../agents/pty-pipeline.md) | Agent PTY streaming pipeline (tool/plugin authors) |
| [../development/pty-panes.md](../development/pty-panes.md) | Agent and shell PTY focus, scrollback, quit |
| [themes.md](./themes.md) | Visual design, colors, and built-in themes |
| [mouse-interaction.md](./mouse-interaction.md) | Mouse affordances, double-click preview, in-app selection |
| [keyboard-shortcuts.md](./keyboard-shortcuts.md) | Default keybindings reference |
| [workflows.md](./workflows.md) | Issue-driven, AI-driven, and traditional flows |
| [gui-layout.md](./gui-layout.md) | Desktop GUI dock layout and panel design |
| [gui-implementation-plan.md](./gui-implementation-plan.md) | Phased plan for `kiwi_gui` |
| [gui-keyboard-shortcuts.md](./gui-keyboard-shortcuts.md) | GUI default keybindings |

## Design Principles

1. **Density with clarity** — Show maximum context without clutter; use muted chrome and accent for focus.
2. **Keyboard first, mouse optional** — Every action reachable via keyboard; mouse accelerates common tasks.
3. **Stable views** — Lists and trees should not jump on background updates.
4. **Orchestrator cues** — Copy and affordances say "open in editor" not "edit here".
5. **Progressive disclosure** — Lazy file tree, paginated GitHub comments, truncated diffs.

## Dual Frontend

The **TUI** (`kiwi`) and **desktop GUI** (`kiwi-gui`) share domain logic via `kiwi_core`. TUI design docs apply to the terminal binary only; GUI docs describe egui/eframe chrome. See ADR-020.

## Wireframe Reference

From [plan.md](../plan.md):

```text
┌────────────────────────────┬─────────────────────────────────────────────┐
│ Files Git Diff GH Search   │ Agent Issues PRs Diff Preview Logs         │
├────────────────────────────┼─────────────────────────────────────────────┤
│ Left Navigation Content    │ Main Workspace Content                      │
│                            │                                             │
├────────────────────────────┼─────────────────────────────────────────────┤
│ Command Palette            │ Shell                                      │
└────────────────────────────┴─────────────────────────────────────────────┘
Status Bar
```

## Personas

| Persona | Needs |
|---------|-------|
| Terminal-native developer | Vim/Neovim, keyboard shortcuts, no mouse required |
| AI-assisted builder | Agent tab, issue linking, diff review |
| OSS maintainer | GH left issue list + Issues detail, PRs, `gh` workflows in one place |

## Related

- [Architecture ADRs](../architecture/adr/README.md)
- [Specifications](../specs/README.md)
