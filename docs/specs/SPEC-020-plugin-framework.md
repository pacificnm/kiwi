# SPEC-020: Plugin Framework

## Purpose

Define extension interface for third-party plugins per ADR-018 (implementation Milestone 7).

## Scope

### In scope

- Plugin API crate design
- Discovery and loading
- Plugin registry (enable/disable persistence)
- Command registration hook
- CLI plugin management (`kiwi plugin *`)
- Plugin Manager UI (TUI + GUI)

### Out of scope

- Plugin marketplace
- Sandbox/security enforcement

## Functional Requirements

### Phase 1 (M7) — Implemented

1. Load plugins from `~/.config/kiwi/plugins/`.
2. Each plugin exports `kiwi_plugin_init` returning `PluginDescriptor`.
3. Register commands into palette via `PluginApi::register_command`.
4. Plugin metadata in `plugin.toml`: name, version, `min_kiwi_version`, optional
   `display_name`, `description`, `author`, `capabilities`.
5. Plugin registry persisted at `~/.config/kiwi/plugin-registry.toml`; plugins
   auto-registered as enabled on first discovery.
6. `kiwi plugin list/info/enable/disable/install/remove/reload` CLI subcommands.
7. TUI Plugin Manager tab (`9` or palette: "Plugins: Open Manager").
8. GUI Plugin Manager dock panel (File → Plugins, or View → Plugins checkbox).

### Phase 2+

9. Event subscriptions: `on_git_refresh`, `on_file_save`.
10. Optional custom tab registration.

### API stability

- `kiwi_plugin_api` crate versioned semver
- Kiwi checks `min_kiwi_version` on load; skip incompatible with warning

## Non-Functional Requirements

- Plugin load < 200ms each
- Failed plugin does not prevent startup

## Data Structures

```rust
// kiwi_plugin_api
struct PluginDescriptor {
    name: &'static str,
    version: &'static str,
    register: fn(&mut PluginApi),
}

struct PluginManifest {
    name: String,
    version: String,
    min_kiwi_version: String,
    entry: String,                       // default: "kiwi_plugin_init"
    display_name: Option<String>,
    description: Option<String>,
    author: Option<String>,
    capabilities: Option<PluginCapabilities>,
}

struct PluginCapabilities {
    commands: bool,
    panels: bool,
    tabs: bool,
    events: bool,
    mcp: bool,
}

// kiwi_core::state
enum PluginStatus { Loaded, Disabled, Failed(String), Incompatible(String), Missing }

struct PluginEntry {
    name: String,
    display_name: String,
    version: String,
    description: String,
    author: String,
    enabled: bool,
    status: PluginStatus,
    command_ids: Vec<String>,
}
```

## Plugin Registry

`~/.config/kiwi/plugin-registry.toml` is auto-created on first startup. Format:

```toml
[plugins.my-plugin]
name = "my-plugin"
version = "0.1.0"
enabled = true
installed_path = "/home/user/.config/kiwi/plugins/my-plugin"
entry = "libmy_plugin.so"
source = "local"
```

A missing file means all discovered plugins are treated as enabled (backward compatible).

## Events / Commands

```rust
// Command palette entry added in kiwi_core
CommandDef { id: "main.plugins", title: "Plugins: Open Manager", shortcut: "9" }
```

## Configuration Options

```toml
[plugins]
enabled = true
directory = "~/.config/kiwi/plugins"
```

## Error Handling

| Error | Behavior |
|-------|----------|
| ABI mismatch | Skip plugin; log; status = Failed |
| `min_kiwi_version` too high | Skip plugin; status = Incompatible |
| Panic in plugin | Isolate; disable plugin for session; status = Failed |
| Invalid manifest | Skip; log warning |
| Registry file corrupt | Log warning; start with empty registry |

## Acceptance Criteria

- [x] Sample plugin registers one palette command
- [x] Incompatible version skipped gracefully
- [x] Plugin failure does not crash Kiwi
- [x] API documented in `kiwi_plugin_api` README
- [x] Security warning in user docs
- [x] Plugin registry persists enable/disable state
- [x] `kiwi plugin` CLI subcommands for management
- [x] TUI Plugin Manager screen (tab index 8, shortcut `9`)
- [x] GUI Plugin Manager dock panel (File → Plugins)
