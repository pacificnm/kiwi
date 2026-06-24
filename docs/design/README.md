# Design Documentation

UX and interaction design for Kiwi. These documents describe how the product should look and feel; specifications define implementable contracts.

## Documents

| Document | Description |
|----------|-------------|
| [layout.md](./layout.md) | Overall TUI layout, regions, and proportions |
| [navigation.md](./navigation.md) | Tab model, focus, and pane behavior |
| [themes.md](./themes.md) | Visual design, colors, and built-in themes |
| [mouse-interaction.md](./mouse-interaction.md) | Mouse affordances and terminal selection |
| [keyboard-shortcuts.md](./keyboard-shortcuts.md) | Default keybindings reference |
| [workflows.md](./workflows.md) | Issue-driven, AI-driven, and traditional flows |

## Design Principles

1. **Density with clarity** — Show maximum context without clutter; use muted chrome and accent for focus.
2. **Keyboard first, mouse optional** — Every action reachable via keyboard; mouse accelerates common tasks.
3. **Stable views** — Lists and trees should not jump on background updates.
4. **Orchestrator cues** — Copy and affordances say "open in editor" not "edit here".
5. **Progressive disclosure** — Lazy file tree, paginated GitHub comments, truncated diffs.

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
| OSS maintainer | Issues, PRs, `gh` workflows in one place |

## Related

- [Architecture ADRs](../architecture/adr/README.md)
- [Specifications](../specs/README.md)
