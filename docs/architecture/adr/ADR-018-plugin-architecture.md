# ADR-018: Plugin Architecture

## Status

Accepted — implemented (Milestone 7, #319)

## Context

Kiwi cannot implement every integration (Jira, GitLab, custom agents, linters) in core. A plugin system enables community extensions without bloating the main binary.

## Decision

Adopt a **Rust dynamic library plugin model** with a stable C ABI or `abi_stable` interface for v1 plugins, plus a **command registration** hook.

### Plugin capabilities (phased)

| Phase | Capability |
|-------|------------|
| P1 | Register commands in palette |
| P2 | Register left nav tab or main tab |
| P3 | Subscribe to events (git changed, file saved) |
| P4 | Render custom widget panel (advanced) |

### Discovery

```text
~/.config/kiwi/plugins/*.so   # Linux
~/.config/kiwi/plugins/*.dylib # macOS
```

Manifest per plugin: `plugin.toml` with name, version, entry symbol, min_kiwi_version.

### Security model

Plugins run **in-process** with full user privileges. Document risk; no sandbox in v1. Curated plugin list recommended.

### API surface (conceptual)

```rust
// kiwi_plugin_api crate — stable semver
trait KiwiPlugin {
    fn register(&self, api: &mut PluginApi);
}
```

Core ships `kiwi_plugin_api` as a separate crate; plugins depend on it.

## Consequences

### Positive

- Extensibility without core bloat
- Command palette extensibility immediately useful
- Aligns with editor-agnostic orchestrator role

### Negative

- ABI stability burden across Kiwi versions
- In-process plugins are security-sensitive
- P4 custom rendering is complex

## Alternatives Considered

| Alternative | Rejection Rationale |
|-------------|---------------------|
| WASM plugins | Heavier runtime; PTY/GIT access harder |
| External RPC plugins | Latency and packaging complexity |
| No plugins, fork only | Poor ecosystem growth |
| Lua scripting | Less idiomatic for Rust project |

## Implementation Notes (M7, #319)

Phase 1 is fully implemented:

- `kiwi_plugin_api` crate with `PluginManifest`, `PluginCapabilities`, `declare_plugin!`
- `PluginRegistry` persisting to `~/.config/kiwi/plugin-registry.toml`
- `kiwi plugin list/info/enable/disable/install/remove/reload` CLI
- `PluginStatus` lifecycle (Loaded/Disabled/Failed/Incompatible/Missing)
- TUI Plugin Manager (main tab 8, shortcut `9`, palette "Plugins: Open Manager")
- GUI Plugin Manager dock panel (`KiwiTab::Plugins`, File → Plugins menu)
- Reference plugin: `plugins/kiwi_plugin_hello/`

## Follow-up Work

- Phase 2: event subscriptions (`on_git_refresh`, `on_file_save`)
- Phase 3: custom tab registration via manifest
- Phase 4: custom widget panel rendering
