pub mod install;
pub mod registry;

pub use install::{
    install_plugin_from_source, install_plugin_from_source_with_progress,
    reinstall_plugin_from_source, reinstall_plugin_from_source_with_progress,
    remove_plugin_from_disk, InstallProgressUpdate, PluginInstallResult, PluginRemoveResult,
};
pub use registry::{
    default_registry_path, install_plugin, reinstall_plugin, remove_plugin, PluginRegistry,
    PluginRegistryEntry,
};

/// If `name` is not in `registry`, reads its `plugin.toml` from
/// `install_dir/<name>/plugin.toml` and registers it as enabled.
/// Returns `true` if a new entry was added.
pub fn ensure_registered(registry: &mut PluginRegistry, name: &str) -> bool {
    if registry.get(name).is_some() {
        return false;
    }
    let home = std::env::var_os("HOME").map(std::path::PathBuf::from);
    let install_dir = crate::config::default_plugins_directory(home.as_deref());
    let plugin_path = install_dir.join(name);
    let manifest_path = plugin_path.join("plugin.toml");
    let Ok(content) = std::fs::read_to_string(&manifest_path) else { return false };
    let Ok(manifest) = toml::from_str::<kiwi_plugin_api::PluginManifest>(&content) else {
        return false;
    };
    registry.register(PluginRegistryEntry {
        name: manifest.name.clone(),
        display_name: manifest.display_name.clone(),
        version: manifest.version.clone(),
        enabled: true,
        installed_path: plugin_path,
        entry: manifest.entry.clone(),
        source: "local".to_string(),
    });
    true
}

/// Scan `search_dirs` for subdirectories containing `plugin.toml` and return
/// an `AvailablePlugin` per discovery, cross-referenced against `registry` and
/// the on-disk install directory at `~/.config/kiwi/plugins/<name>`.
#[must_use]
pub fn scan_available_plugins(
    search_dirs: &[&std::path::Path],
    registry: &PluginRegistry,
) -> Vec<crate::state::AvailablePlugin> {
    let home = std::env::var_os("HOME").map(std::path::PathBuf::from);
    let install_dir = crate::config::default_plugins_directory(home.as_deref());

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
            // A plugin is installed if it's in the registry OR its directory
            // already exists in the user's install location.
            let on_disk = install_dir.join(&manifest.name).exists();
            let installed = reg_entry.is_some() || on_disk;
            let enabled = reg_entry.map_or(on_disk, |e| e.enabled);
            let (agent_command, agent_args, agent_mode, agent_provider, agent_model, agent_api_key_env, agent_api_url) = manifest
                .agent
                .as_ref()
                .map(|a| {
                    let mode_str = match a.mode {
                        kiwi_plugin_api::AgentMode::Api => Some("api".to_string()),
                        kiwi_plugin_api::AgentMode::Pty => None,
                    };
                    let cmd = if a.command.is_empty() { None } else { Some(a.command.clone()) };
                    (cmd, a.args.clone(), mode_str, a.provider.clone(), a.model.clone(), a.api_key_env.clone(), a.api_url.clone())
                })
                .unwrap_or((None, Vec::new(), None, None, None, None, None));
            found.push(crate::state::AvailablePlugin {
                display_name: manifest.effective_display_name().to_string(),
                version: manifest.version.clone(),
                description: manifest.description.clone().unwrap_or_default(),
                author: manifest.author.clone().unwrap_or_default(),
                source_path: plugin_dir,
                installed,
                enabled,
                name: manifest.name,
                agent_command,
                agent_args,
                agent_mode,
                agent_provider,
                agent_model,
                agent_api_key_env,
                agent_api_url,
            });
        }
    }
    found.sort_by(|a, b| a.name.cmp(&b.name));
    found
}
