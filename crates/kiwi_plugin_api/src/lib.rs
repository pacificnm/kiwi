//! Stable plugin interface for Kiwi dynamic-library extensions.
//!
//! See [SPEC-020](https://github.com/pacificnm/kiwi/blob/main/docs/specs/SPEC-020-plugin-framework.md)
//! and [ADR-018](https://github.com/pacificnm/kiwi/blob/main/docs/architecture/adr/ADR-018-plugin-architecture.md).

mod api;
mod manifest;
mod version;

pub use api::{
    KiwiPlugin, PluginApi, PluginCommand, PluginDescriptor, PluginInitFn, PluginRegisterFn,
    PluginRegistrar, PluginResult, StaticStr,
};
pub use manifest::{AgentPluginConfig, PluginCapabilities, PluginManifest};
pub use version::{
    api_version_compatible, kiwi_version_compatible, API_VERSION, DEFAULT_PLUGIN_INIT_SYMBOL,
    PLUGIN_INIT_SYMBOL,
};

/// Re-export for plugin authors building `plugin.toml` manifests.
pub use serde;
