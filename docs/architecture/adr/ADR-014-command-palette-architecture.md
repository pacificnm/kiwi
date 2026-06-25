# ADR-014: Command Palette Architecture

## Status

Accepted

## Context

Kiwi exposes many actions across panes (switch tab, refresh git, create PR, focus shell). A command palette provides discoverability similar to IDE Cmd+K palettes while fitting the bottom-left layout region.

## Decision

Implement a **modal command palette** in the bottom-left panel with fuzzy filtering and keyboard-first interaction.

### Modes

| Mode | Trigger | Purpose |
|------|---------|---------|
| Command | `Ctrl+P` default | Execute app commands |
| File jump | `Ctrl+G` | Quick open file by path |
| Shell prefix | `:` when palette focused | Pass to shell (optional v1 — defer `:shell` to shell focus) |

### Command registry

Static registry at compile time for MVP; plugins add commands later (ADR-018).

Each command:

```rust
// Conceptual
Command {
    id: "git.refresh",
    title: "Git: Refresh Status",
    shortcut: Some("R"),
    context: CommandContext::GitPanel,
    action: AppCommand::GitRefresh,
}
```

### UI behavior

- Palette height grows with input (max 10 visible results)
- Fuzzy match on title and id
- `Enter` executes; `Esc` dismisses and restores focus
- Mouse: click result row to execute (ADR-015)
- Clipboard commands: **Clipboard: Copy**, **Clipboard: Cut**, **Clipboard: Paste** (ADR-019); palette input is copy/cut target when open

### Layout interaction

Bottom panel split: palette (top of bottom-left) shares column with shell below-right. When palette active, it captures keyboard input.

## Consequences

### Positive

- Discoverability without memorizing all shortcuts
- Extensible registry for plugins
- Consistent with Cursor/VS Code mental model

### Negative

- Overlapping shortcuts with shell when focus wrong
- Static registry requires recompile to add commands until plugins land

## Alternatives Considered

| Alternative | Rejection Rationale |
|-------------|---------------------|
| Vim-style `:` only | Poor discoverability |
| Top overlay palette | Conflicts with layout wireframe |
| External `fzf` | Weaker integration |

## Follow-up Work

- SPEC-013 Command Palette
- Design: [keyboard-shortcuts.md](../../design/keyboard-shortcuts.md)
- Initial command list: ~30 commands covering navigation, git, github, editor
- Bracketed paste support in palette input
