# ADR-004: Theme System

## Status

Accepted

## Context

Kiwi must feel modern and readable. Developers expect dark/light modes and popular third-party palettes. Git, GitHub, and agent states need semantic colors consistent across all panes.

## Decision

Implement a **declarative theme engine** with:

1. **Built-in themes** (bundled TOML): Kiwi Dark (default), Kiwi Light, Dracula, Catppuccin Mocha/Latte, Gruvbox, Nord, Tokyo Night
2. **User themes** via `~/.config/kiwi/themes/*.toml` or `[theme] custom = "path"`
3. **Semantic color roles** resolved at render time, not hard-coded hex in widgets

### Semantic roles

| Category | Roles |
|----------|-------|
| UI chrome | `bg`, `fg`, `border`, `accent`, `muted`, `selection` |
| Git | `git_added`, `git_modified`, `git_deleted`, `git_untracked` |
| Issues | `issue_open`, `issue_in_progress`, `issue_closed` |
| PRs | `pr_open`, `pr_draft`, `pr_merged`, `pr_closed` |
| Agent | `agent_thinking`, `agent_executing`, `agent_success`, `agent_error`, `agent_warning` |

### Configuration

```toml
[theme]
name = "kiwi-dark"
# OR
custom = "~/.config/kiwi/themes/my-theme.toml"
```

Themes map roles to ANSI 16/256/truecolor as supported by terminal. crossterm queries terminal capabilities at startup.

## Consequences

### Positive

- Consistent semantics across all views
- Users can share theme files without recompiling
- Popular palettes ship out of the box

### Negative

- Truecolor fallback logic adds complexity
- Custom themes must define all required roles or inherit from a base theme
- PTY child processes (shell, agent) do not auto-inherit Kiwi theme

## Alternatives Considered

| Alternative | Rejection Rationale |
|-------------|---------------------|
| Hard-coded palettes in Rust only | Poor extensibility |
| CSS-like theme inheritance | Over-engineered for v1 |
| No built-in themes, config only | Poor first-run experience |

## Follow-up Work

- SPEC-003 Theme Engine
- Ship theme TOML files under `assets/themes/`
- Design doc: [themes.md](../../design/themes.md)
- Document color guidelines from plan.md in theme schema
- Milestone 6: theme packs distribution
