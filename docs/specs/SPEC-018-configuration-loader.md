# SPEC-018: Configuration Loader

## Purpose

Load, merge, validate, and expose configuration from CLI, project, and user sources.

## Scope

### In scope

- TOML parsing
- Precedence merge
- Path expansion
- Defaults

### Out of scope

- Config hot reload
- JSON schema export

## Functional Requirements

1. Load defaults struct in code.
2. Overlay `~/.config/kiwi/config.toml` if exists.
3. Overlay `.kiwi.toml` in repo root if exists.
4. Apply CLI overrides: `--config`, `--theme`, `--left-width`, etc.
5. Expand `~` in paths.
6. Validate ranges: `left_width` 10–50, theme name non-empty.
7. Produce `ResolvedConfig` immutable snapshot for app lifetime.

## Non-Functional Requirements

- Load < 20ms
- Clear error messages with file path and key

## Data Structures

```rust
struct RawConfig {
    app: Option<AppSection>,
    theme: Option<ThemeSection>,
    editor: Option<EditorSection>,
    agent: Option<AgentSection>,
    shell: Option<ShellSection>,
    mouse: Option<MouseSection>,
    git: Option<GitSection>,
    github: Option<GitHubSection>,
    workspace: Option<WorkspaceSection>,
    search: Option<SearchSection>,
    preview: Option<PreviewSection>,
    diff: Option<DiffSection>,
}

struct ResolvedConfig {
    // All fields required with resolved values
}
```

## Events / Commands

```rust
// Startup only
fn load_config(cli: &Cli, repo_root: &Path) -> Result<ResolvedConfig>
```

## Configuration Options

See [plan.md](../../plan.md) example; all sections documented across SPECs.

### Full key reference

| Section | Keys |
|---------|------|
| `app` | `left_width` |
| `theme` | `name`, `custom` |
| `editor` | `command` |
| `agent` | `command`, `args`, `env` |
| `shell` | `command`, `args` |
| `mouse` | `enabled`, `mode` |
| `git` | `watch`, `show_untracked` |
| `github` | `command` |
| `workspace` | `persist`, `save_interval_secs` |
| `search` | `command`, `debounce_ms` |
| `preview` | `max_size_bytes`, `line_numbers`, `wrap` |
| `diff` | `context_lines`, `word_wrap` |

## Error Handling

| Error | Behavior |
|-------|----------|
| Parse error | Return `StartupError::ConfigParse` |
| Unknown key | serde deny_unknown_fields false; ignore with warn log |
| Invalid value | Parse error with key name |

## Acceptance Criteria

- [ ] Project config overrides user config
- [ ] CLI overrides project
- [ ] Defaults apply when no files
- [ ] `~` expansion works on custom theme path
- [ ] Invalid TOML fails with line number
