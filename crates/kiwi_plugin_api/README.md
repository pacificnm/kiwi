# kiwi_plugin_api

Stable Rust API for [Kiwi](https://github.com/pacificnm/kiwi) dynamic-library plugins.

This crate is intentionally minimal: it has **no dependency on Kiwi itself**. Plugin
crates depend on `kiwi_plugin_api`; Kiwi depends on it to load and invoke plugins.

Specifications: [SPEC-020](https://github.com/pacificnm/kiwi/blob/main/docs/specs/SPEC-020-plugin-framework.md),
[ADR-018](https://github.com/pacificnm/kiwi/blob/main/docs/architecture/adr/ADR-018-plugin-architecture.md).

## Security warning

Kiwi plugins are **native dynamic libraries loaded in-process**. A plugin runs with the
same privileges as your user account and can read files, invoke shells, and access
network resources. Only install plugins from sources you trust.

There is **no sandbox** in v1. Treat third-party plugins like running arbitrary code.

## Phase 1 (M7) surface

| Type / constant | Purpose |
| --- | --- |
| `API_VERSION` | Semver of this crate; major version must match at load time |
| `PLUGIN_INIT_SYMBOL` | Entry symbol (`kiwi_plugin_init`) exported by plugin `.so`/`.dylib` |
| `PluginDescriptor` | Returned from `kiwi_plugin_init` |
| `PluginApi` | Passed to `register`; call `register_command` |
| `PluginCommand` | Palette command (`id`, `title`, `callback`) |
| `PluginManifest` | Deserialized from `plugin.toml` |
| `PluginCapabilities` | Optional capability flags in `plugin.toml` (informational) |
| `KiwiPlugin` + `declare_plugin!` | Ergonomic plugin authoring |

Event subscriptions and custom tabs are reserved for later phases.

## Plugin layout

```text
~/.config/kiwi/plugins/
├── my-plugin/
│   ├── plugin.toml
│   └── libmy_plugin.so      # Linux (.dylib on macOS)
```

### `plugin.toml`

```toml
name = "my-plugin"
version = "0.1.0"
min_kiwi_version = "0.1.0"

# Optional metadata (shown in the Plugin Manager)
display_name = "My Plugin"
description  = "Does something useful."
author       = "Your Name"

# entry = "kiwi_plugin_init"   # optional; this is the default

# Optional capability hints (informational only, not enforced)
[capabilities]
commands = true   # registers palette commands
panels   = false  # adds custom panels
tabs     = false  # adds main/left nav tabs
events   = false  # subscribes to app events
mcp      = false  # exposes an MCP tool
```

All fields except `name`, `version`, and `min_kiwi_version` are optional and
backward-compatible with existing manifests.

## Plugin registry

Kiwi tracks installed plugins in `~/.config/kiwi/plugin-registry.toml`. This file is
created automatically when Kiwi starts and discovers plugins for the first time — you
do not need to create it manually.

New plugin directories are **auto-registered as enabled** on startup.

## Installing plugins

The recommended way is the `kiwi plugin` CLI (runs without starting the TUI):

```bash
# Install from a local directory (copies files, registers in registry)
kiwi plugin install /path/to/my-plugin

# List all registered plugins
kiwi plugin list

# Show details for a plugin
kiwi plugin info my-plugin

# Enable / disable (takes effect on next restart)
kiwi plugin enable my-plugin
kiwi plugin disable my-plugin

# Remove from registry (files are NOT deleted)
kiwi plugin remove my-plugin

# Normalise the registry file
kiwi plugin reload
```

You can also install manually by placing the plugin directory under
`~/.config/kiwi/plugins/` and restarting Kiwi.

## Plugin Manager UI

Once Kiwi is running, open the Plugin Manager to inspect installed plugins:

- **TUI**: press `9` or open the command palette (`Ctrl+P`) and search for
  **"Plugins: Open Manager"**.
- **GUI**: use the **File → Plugins** menu or toggle the **Plugins** panel from
  the **View** menu.

The Plugin Manager displays a list of all registered plugins with their status
(`Loaded`, `Disabled`, `Failed`, `Incompatible`) and a detail view showing version,
author, description, and registered palette commands.

## Authoring a plugin

Add to your plugin `Cargo.toml`:

```toml
[lib]
crate-type = ["cdylib"]

[dependencies]
kiwi_plugin_api = "0.1"
```

Implement [`KiwiPlugin`] and export the entry symbol:

```rust
use kiwi_plugin_api::{declare_plugin, KiwiPlugin, PluginApi, PluginResult};

#[derive(Default)]
struct MyPlugin;

impl KiwiPlugin for MyPlugin {
    fn name(&self) -> &'static str {
        "my-plugin"
    }

    fn version(&self) -> &'static str {
        "0.1.0"
    }

    fn register(&self, api: &mut PluginApi<'_>) {
        api.register_command("my-plugin.hello", "My Plugin: Hello", hello);
    }
}

extern "C" fn hello() -> PluginResult {
    PluginResult::Ok
}

declare_plugin!(MyPlugin);
```

Or export [`kiwi_plugin_init`] manually:

```rust
use kiwi_plugin_api::{PluginApi, PluginDescriptor, PluginResult, StaticStr};

fn register(api: &mut PluginApi<'_>) {
    api.register_command("my-plugin.hello", "My Plugin: Hello", hello);
}

extern "C" fn hello() -> PluginResult {
    PluginResult::Ok
}

#[no_mangle]
pub extern "C" fn kiwi_plugin_init() -> PluginDescriptor {
    PluginDescriptor {
        name: StaticStr::from_static("my-plugin"),
        version: StaticStr::from_static("0.1.0"),
        register,
    }
}
```

> **Note:** Plugin crates are separate from the Kiwi workspace. They may use `#[no_mangle]`
> even though the Kiwi workspace forbids `unsafe` internally.

## Version compatibility

- **`API_VERSION`**: Kiwi skips plugins built against an incompatible `kiwi_plugin_api`
  major version.
- **`min_kiwi_version`** (manifest): Kiwi skips plugins that require a newer app version.

Helpers: [`api_version_compatible`], [`kiwi_version_compatible`].

## Plugin status lifecycle

| Status | Meaning |
| --- | --- |
| `Loaded` | Successfully loaded and commands are active in the palette |
| `Disabled` | Present on disk but disabled via `kiwi plugin disable` — not loaded |
| `Failed` | Library found but failed to load (ABI mismatch, missing symbol, etc.) |
| `Incompatible` | `min_kiwi_version` exceeds the running Kiwi version |
| `Missing` | Registered in the registry but the files cannot be found on disk |

## Host integration (Kiwi core)

Kiwi implements [`PluginRegistrar`] to collect [`PluginCommand`] values during load.
Discovery, dynamic loading, registry management, and palette wiring live in the main
`kiwi` crate. See the reference plugin at `plugins/kiwi_plugin_hello/`.

[`KiwiPlugin`]: https://docs.rs/kiwi_plugin_api/latest/kiwi_plugin_api/trait.KiwiPlugin.html
[`kiwi_plugin_init`]: https://docs.rs/kiwi_plugin_api/latest/kiwi_plugin_api/constant.PLUGIN_INIT_SYMBOL.html
[`api_version_compatible`]: https://docs.rs/kiwi_plugin_api/latest/kiwi_plugin_api/fn.api_version_compatible.html
[`kiwi_version_compatible`]: https://docs.rs/kiwi_plugin_api/latest/kiwi_plugin_api/fn.kiwi_version_compatible.html
[`PluginRegistrar`]: https://docs.rs/kiwi_plugin_api/latest/kiwi_plugin_api/trait.PluginRegistrar.html
[`PluginCommand`]: https://docs.rs/kiwi_plugin_api/latest/kiwi_plugin_api/struct.PluginCommand.html
