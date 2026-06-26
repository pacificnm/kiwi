# SPEC-003: Theme Engine

## Purpose

Load, resolve, and apply visual themes with semantic color roles across all Kiwi widgets.

## Scope

### In scope

- Built-in themes
- Custom theme files
- Role → Style resolution for ratatui
- Terminal capability detection (16/256/truecolor)

### Out of scope

- Dynamic theme switch without restart (post-MVP)
- Theming PTY child output

## Functional Requirements

1. Load theme by name from embedded assets or `custom` path.
2. Built-in themes: kiwi-dark, kiwi-light, dracula, catppuccin-mocha, catppuccin-latte, gruvbox, nord, tokyo-night.
3. Resolve roles: UI chrome, git, issues, PRs, agent, file tree (per plan.md color guidelines).
4. Missing role in custom theme: inherit from kiwi-dark base.
5. Expose `ThemePalette` to widgets via `AppState.theme`.
6. Invalid custom theme file: startup error with path and missing role.

## Non-Functional Requirements

- Theme load < 50ms
- All text meets contrast guidelines on default terminals (document exceptions)

## Data Structures

```rust
struct ThemePalette {
    roles: HashMap<SemanticRole, Style>,
}

enum SemanticRole {
    Bg, Fg, Border, Accent, Muted, Selection,
    GitAdded, GitModified, GitDeleted, GitUntracked,
    IssueOpen, IssueInProgress, IssueClosed,
    PrOpen, PrDraft, PrMerged, PrClosed,
    AgentThinking, AgentExecuting, AgentSuccess, AgentError, AgentWarning,
    FileDir, FileSource, FileScript, FileMarkup, FileConfig, FileData, FileMedia, FileOther,
}

struct ThemeDefinition {
    name: String,
    extends: Option<String>,
    colors: HashMap<String, String>,  // role name → "#hex" or "ansi_N"
}
```

## Events / Commands

| Command | Action |
|---------|--------|
| `AppCommand::SetTheme(String)` | Future: hot reload |

## Configuration Options

```toml
[theme]
name = "kiwi-dark"
custom = "~/.config/kiwi/themes/custom.toml"  # optional, overrides name
```

## Error Handling

| Error | Behavior |
|-------|----------|
| Unknown theme name | Exit 1 at startup |
| Parse error in custom TOML | Exit 1 with details |
| Unknown color format | Exit 1 |

## Acceptance Criteria

- [ ] Default kiwi-dark applies on first run
- [ ] Each built-in theme loads without error
- [ ] Git modified file shows yellow semantic color
- [ ] Custom theme with one overridden role works
- [ ] Truecolor terminals render hex colors
