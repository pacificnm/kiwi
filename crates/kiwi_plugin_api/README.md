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
# entry = "kiwi_plugin_init"   # optional; this is the default
```

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

## Host integration (Kiwi core)

Kiwi implements [`PluginRegistrar`] to collect [`PluginCommand`] values during load.
Discovery, dynamic loading, and palette wiring are implemented in follow-up issues
(#69, #70).

[`KiwiPlugin`]: https://docs.rs/kiwi_plugin_api/latest/kiwi_plugin_api/trait.KiwiPlugin.html
[`kiwi_plugin_init`]: https://docs.rs/kiwi_plugin_api/latest/kiwi_plugin_api/constant.PLUGIN_INIT_SYMBOL.html
[`api_version_compatible`]: https://docs.rs/kiwi_plugin_api/latest/kiwi_plugin_api/fn.api_version_compatible.html
[`kiwi_version_compatible`]: https://docs.rs/kiwi_plugin_api/latest/kiwi_plugin_api/fn.kiwi_version_compatible.html
[`PluginRegistrar`]: https://docs.rs/kiwi_plugin_api/latest/kiwi_plugin_api/trait.PluginRegistrar.html
[`PluginCommand`]: https://docs.rs/kiwi_plugin_api/latest/kiwi_plugin_api/struct.PluginCommand.html
