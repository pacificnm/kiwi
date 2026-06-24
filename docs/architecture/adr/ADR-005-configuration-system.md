# ADR-005: Configuration System

## Status

Accepted

## Context

Kiwi must support per-user defaults and per-project overrides for editor, agent, shell, theme, layout, Git, GitHub, and mouse behavior. Configuration must be human-editable and version-controllable (project config in repo).

## Decision

Use **TOML** configuration files with strict resolution order:

1. CLI arguments (highest precedence)
2. Project: `.kiwi.toml` in repository root
3. User: `~/.config/kiwi/config.toml`
4. Built-in defaults (lowest)

### Config locations

| File | Purpose |
|------|---------|
| `~/.config/kiwi/config.toml` | User-wide defaults |
| `.kiwi.toml` | Project overrides (commit-friendly) |
| `~/.config/kiwi/themes/*.toml` | Custom themes |

### Example

```toml
[app]
left_width = 30

[theme]
name = "kiwi-dark"

[editor]
command = "nvim"

[agent]
command = "agent"

[shell]
command = "bash"

[mouse]
enabled = true
mode = "hybrid"

[git]
watch = true
show_untracked = true

[github]
command = "gh"
```

Deserialize with `serde` + `toml`. Validate on load; fail fast with actionable error messages.

**Hot reload** is out of scope for MVP; restart required after config change.

## Consequences

### Positive

- Familiar format for Rust ecosystem
- Project config enables team consistency
- Clear precedence rules

### Negative

- No runtime reload in v1
- Invalid TOML blocks startup (intentional fail-fast)
- Path expansion (`~`) must be implemented consistently

## Alternatives Considered

| Alternative | Rejection Rationale |
|-------------|---------------------|
| YAML | Less common in Rust CLI tools; ambiguity issues |
| JSON | Poor human editability for config |
| Environment variables only | Insufficient for structured nested config |
| XDG JSON | TOML aligns with Cargo/Rust conventions |

## Follow-up Work

- SPEC-018 Configuration Loader
- Document all keys in specs and `config.example.toml` at scaffold
- Support `KIWI_*` env overrides as optional future enhancement
- Validate `editor.command`, `agent.command`, `shell.command` exist on PATH at startup (warn, don't fail)
