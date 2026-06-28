pub mod registry;

pub use registry::{default_registry_path, install_plugin, PluginRegistry, PluginRegistryEntry};

/// Scan `search_dirs` for subdirectories containing `plugin.toml` and return
/// an `AvailablePlugin` per discovery, cross-referenced against `registry`.
#[must_use]
pub fn scan_available_plugins(
    search_dirs: &[&std::path::Path],
    registry: &PluginRegistry,
) -> Vec<crate::state::AvailablePlugin> {
    let mut found = Vec::new();
    for dir in search_dirs {
        let Ok(read) = std::fs::read_dir(dir) else { continue };
        for item in read.flatten() {
            let plugin_dir = item.path();
            if !plugin_dir.is_dir() {
                continue;
            }
            let manifest_path = plugin_dir.join("plugin.toml");
            let Ok(content) = std::fs::read_to_string(&manifest_path) else { continue };
            let Ok(manifest) = toml::from_str::<kiwi_plugin_api::PluginManifest>(&content) else {
                continue
            };
            let reg_entry = registry.get(&manifest.name);
            let (agent_command, agent_args) = manifest
                .agent
                .as_ref()
                .map(|a| (Some(a.command.clone()), a.args.clone()))
                .unwrap_or((None, Vec::new()));
            found.push(crate::state::AvailablePlugin {
                display_name: manifest.effective_display_name().to_string(),
                version: manifest.version.clone(),
                description: manifest.description.clone().unwrap_or_default(),
                author: manifest.author.clone().unwrap_or_default(),
                source_path: plugin_dir,
                installed: reg_entry.is_some(),
                enabled: reg_entry.map_or(false, |e| e.enabled),
                name: manifest.name,
                agent_command,
                agent_args,
            });
        }
    }
    found.sort_by(|a, b| a.name.cmp(&b.name));
    found
}
