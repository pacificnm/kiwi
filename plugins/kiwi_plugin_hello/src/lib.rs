//! Sample Kiwi plugin for SPEC-020 / issue #70.
//!
//! Registers one palette command: **Hello Plugin: Greet** (`hello.greet`).

use kiwi_plugin_api::{declare_plugin, KiwiPlugin, PluginApi, PluginResult};

#[derive(Default)]
struct HelloPlugin;

impl KiwiPlugin for HelloPlugin {
    fn name(&self) -> &'static str {
        "hello"
    }

    fn version(&self) -> &'static str {
        "0.1.0"
    }

    fn register(&self, api: &mut PluginApi<'_>) {
        api.register_command("hello.greet", "Hello Plugin: Greet", greet);
    }
}

extern "C" fn greet() -> PluginResult {
    PluginResult::Ok
}

declare_plugin!(HelloPlugin);
