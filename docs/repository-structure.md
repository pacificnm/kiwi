# Repository Structure Proposal

Proposed layout for the Kiwi Rust codebase. This is documentation only—no crates are scaffolded yet.

## Top-Level Layout

```text
kiwi/
├── Cargo.toml                 # Workspace manifest
├── Cargo.lock
├── README.md
├── CHANGELOG.md
├── config.example.toml
├── plan.md                    # Project initiation (source of truth)
├── docs/                      # This documentation package
├── assets/
│   └── themes/                # Bundled theme TOML files
├── crates/
│   ├── kiwi/                  # Main application binary
│   ├── kiwi_core/             # Shared types, events, config
│   ├── kiwi_plugin_api/       # Stable plugin interface (M7)
│   └── kiwi_tui/              # Widgets and layout (optional split)
├── tests/
│   ├── integration/           # End-to-end tests
│   └── fixtures/              # Sample repos, mock gh output
└── .github/
    └── workflows/
        └── ci.yml
```

## Workspace Crates

### `kiwi` (binary)

Main entry point and application loop.

```text
crates/kiwi/
├── Cargo.toml
└── src/
    ├── main.rs                # CLI, startup, run loop
    ├── app.rs                 # App struct, event loop
    ├── bootstrap.rs           # Service initialization
    └── shutdown.rs            # Cleanup, persistence save
```

### `kiwi_core` (library)

Domain-agnostic types shared across UI and services.

```text
crates/kiwi_core/
├── Cargo.toml
└── src/
    ├── lib.rs
    ├── config/
    │   ├── mod.rs
    │   ├── loader.rs          # SPEC-018
    │   └── types.rs
    ├── events/
    │   ├── mod.rs
    │   ├── app_event.rs
    │   └── app_command.rs
    ├── state/
    │   ├── mod.rs
    │   ├── app_state.rs       # SPEC-016
    │   └── patches.rs
    ├── workspace/
    │   └── persistence.rs     # SPEC-017
    └── theme/
        ├── mod.rs
        ├── palette.rs         # SPEC-003
        └── roles.rs
```

### `kiwi_tui` (library, optional for M1)

Presentation layer: layout, widgets, input translation. May live inside `kiwi` initially; extract when binary grows.

```text
crates/kiwi_tui/
├── Cargo.toml
└── src/
    ├── lib.rs
    ├── layout/
    │   ├── mod.rs
    │   └── engine.rs          # SPEC-002
    ├── widgets/
    │   ├── mod.rs
    │   ├── tab_bar.rs
    │   ├── file_tree.rs
    │   ├── diff_view.rs
    │   ├── preview.rs
    │   ├── status_bar.rs
    │   ├── command_palette.rs
    │   └── pty_view.rs
    ├── input/
    │   ├── mod.rs
    │   ├── keyboard.rs
    │   └── mouse.rs           # SPEC-014
    └── render/
        └── mod.rs
```

### `kiwi_plugin_api` (library, M7)

```text
crates/kiwi_plugin_api/
├── Cargo.toml
└── src/
    ├── lib.rs
    └── api.rs                 # SPEC-020
```

## Main Binary Module Map (`kiwi` crate)

If `kiwi_tui` is deferred, use:

```text
crates/kiwi/src/
├── main.rs
├── app/
│   ├── mod.rs
│   ├── loop.rs
│   └── reducer.rs
├── services/
│   ├── mod.rs
│   ├── shell.rs               # SPEC-011
│   ├── agent.rs               # SPEC-010
│   ├── git.rs                 # SPEC-008
│   ├── github.rs              # SPEC-009
│   ├── search.rs              # SPEC-007
│   ├── editor.rs              # SPEC-015
│   ├── files.rs               # SPEC-005
│   └── watcher.rs             # ADR-011
├── ui/
│   ├── mod.rs
│   ├── layout.rs
│   ├── navigation.rs
│   └── widgets/
└── util/
    ├── paths.rs
    └── pty.rs
```

## Service Architecture

Each service follows the same pattern:

```rust
// Conceptual pattern
pub struct GitService {
    tx: mpsc::Sender<AppEvent>,
}

impl GitService {
    pub fn spawn(repo: PathBuf, config: GitConfig) -> Self { ... }
    pub fn handle(&self, cmd: GitCommand) { ... }
}
```

Services communicate **only** via `AppEvent` to the main loop.

## Assets

```text
assets/themes/
├── kiwi-dark.toml
├── kiwi-light.toml
├── dracula.toml
├── catppuccin-mocha.toml
├── catppuccin-latte.toml
├── gruvbox.toml
├── nord.toml
└── tokyo-night.toml
```

Embedded via `include_str!` or `rust-embed` at build time.

## Configuration Paths (Runtime)

| Path | Purpose |
|------|---------|
| `~/.config/kiwi/config.toml` | User config |
| `.kiwi.toml` | Project config |
| `~/.config/kiwi/themes/` | User themes |
| `~/.local/state/kiwi/workspaces/` | Persistence |
| `~/.config/kiwi/plugins/` | Dynamic plugins (M7) |

## Testing Structure

```text
tests/
├── integration/
│   ├── startup_test.rs
│   ├── config_merge_test.rs
│   └── reducer_git_patch_test.rs
└── fixtures/
    ├── sample_repo/
    └── gh_issue_list.json
```

PTY integration tests may use `portable-pty` with scripted child or mock PTY for CI.

## CI Pipeline

```yaml
# .github/workflows/ci.yml (conceptual)
- cargo fmt --check
- cargo clippy -- -D warnings
- cargo test --workspace
- cargo build --release
```

## Dependency Graph (Crates)

```text
kiwi → kiwi_core, kiwi_tui (optional)
kiwi_tui → kiwi_core, ratatui, crossterm
kiwi_plugin_api → (minimal, no kiwi dep)
plugins → kiwi_plugin_api
```

## Scaffolding Order

1. Workspace `Cargo.toml` with `kiwi` + `kiwi_core`
2. `main.rs` empty loop + quit
3. `kiwi_core::config` + `events` + `state`
4. Add `kiwi_tui` or `kiwi/ui` when widgets multiply
5. `kiwi_plugin_api` stub in M1 (empty trait); implement M7

## Related

- [architecture/README.md](./architecture/README.md)
- [roadmap/backlog.md](./roadmap/backlog.md) issues #1–#4
