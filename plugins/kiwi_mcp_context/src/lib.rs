use kiwi_plugin_api::{declare_plugin, KiwiPlugin, PluginApi, PluginResult};

#[derive(Default)]
struct ContextPlugin;

impl KiwiPlugin for ContextPlugin {
    fn name(&self) -> &'static str {
        "kiwi-mcp-context"
    }

    fn version(&self) -> &'static str {
        "0.1.0"
    }

    fn register(&self, api: &mut PluginApi<'_>) {
        api.register_command(
            "context.save",
            "Context Memory: Save Entry",
            save_context,
        );
        api.register_command(
            "context.search",
            "Context Memory: Search",
            search_context,
        );
        api.register_command(
            "context.list",
            "Context Memory: List Recent",
            list_context,
        );
    }
}

extern "C" fn save_context() -> PluginResult {
    PluginResult::Ok
}

extern "C" fn search_context() -> PluginResult {
    PluginResult::Ok
}

extern "C" fn list_context() -> PluginResult {
    PluginResult::Ok
}

declare_plugin!(ContextPlugin);
