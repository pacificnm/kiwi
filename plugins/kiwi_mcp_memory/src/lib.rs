//! Kiwi plugin for project memory search (SPEC-020 cdylib target).
//!
//! The real work is done by the `kiwi-mcp-memory` binary (MCP server).
//! This cdylib registers palette commands so Kiwi users can discover
//! the memory tools from the command palette.

use kiwi_plugin_api::{declare_plugin, KiwiPlugin, PluginApi, PluginResult};

#[derive(Default)]
struct MemoryPlugin;

impl KiwiPlugin for MemoryPlugin {
    fn name(&self) -> &'static str {
        "kiwi-mcp-memory"
    }

    fn version(&self) -> &'static str {
        "0.1.0"
    }

    fn register(&self, api: &mut PluginApi<'_>) {
        api.register_command(
            "memory.index",
            "Memory: Index Project Docs",
            index_project,
        );
        api.register_command(
            "memory.search",
            "Memory: Search Project Memory",
            search_memory,
        );
    }
}

// Stubs — will be wired to the binary's stdin when the plugin API
// gains event subscription capabilities (SPEC-020 Phase 2).
extern "C" fn index_project() -> PluginResult {
    PluginResult::Ok
}

extern "C" fn search_memory() -> PluginResult {
    PluginResult::Ok
}

declare_plugin!(MemoryPlugin);
