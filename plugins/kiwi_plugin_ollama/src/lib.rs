//! Kiwi plugin for Ollama/qwen2.5-coder (SPEC-020 cdylib target).
//!
//! Registers palette commands that appear in Kiwi's command palette.
//! The actual agent logic lives in the `kiwi-ollama` binary (src/main.rs).

use kiwi_plugin_api::{declare_plugin, KiwiPlugin, PluginApi, PluginResult};

#[derive(Default)]
struct OllamaPlugin;

impl KiwiPlugin for OllamaPlugin {
    fn name(&self) -> &'static str {
        "ollama"
    }

    fn version(&self) -> &'static str {
        "0.1.0"
    }

    fn register(&self, api: &mut PluginApi<'_>) {
        api.register_command("ollama.restart", "Ollama: Restart Agent", restart_agent);
        api.register_command(
            "ollama.clear_context",
            "Ollama: Clear Context",
            clear_context,
        );
    }
}

// Stub — Kiwi's built-in AgentRestart command handles the actual restart.
// These will be wired to agent stdin when the plugin API gains event capabilities.
extern "C" fn restart_agent() -> PluginResult {
    PluginResult::Ok
}

extern "C" fn clear_context() -> PluginResult {
    PluginResult::Ok
}

declare_plugin!(OllamaPlugin);
