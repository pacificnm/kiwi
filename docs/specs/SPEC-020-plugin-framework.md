# SPEC-020: Plugin Framework

## Purpose

Define extension interface for third-party plugins per ADR-018 (implementation Milestone 7).

## Scope

### In scope

- Plugin API crate design
- Discovery and loading
- Command registration hook

### Out of scope

- Plugin marketplace
- Sandbox/security enforcement

## Functional Requirements

### Phase 1 (M7)

1. Load plugins from `~/.config/kiwi/plugins/`.
2. Each plugin exports `kiwi_plugin_init` returning `PluginDescriptor`.
3. Register commands into palette via `PluginApi::register_command`.
4. Plugin metadata in `plugin.toml`: name, version, `min_kiwi_version`.

### Phase 2+

5. Event subscriptions: `on_git_refresh`, `on_file_save`.
6. Optional custom tab registration.

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

struct PluginApi {
    // register_command, subscribe, etc.
}

struct PluginCommand {
    id: String,
    title: String,
    callback: extern "C" fn() -> PluginResult,
}
```

## Events / Commands

```rust
AppEvent::PluginLoaded { name }
AppEvent::PluginCommand { id }
AppCommand::ReloadPlugins  // dev only
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
| ABI mismatch | Skip plugin; log |
| Panic in plugin | Isolate; disable plugin for session |
| Invalid manifest | Skip |

## Acceptance Criteria

- [ ] Sample plugin registers one palette command
- [ ] Incompatible version skipped gracefully
- [ ] Plugin failure does not crash Kiwi
- [ ] API documented in `kiwi_plugin_api` README
- [ ] Security warning in user docs
