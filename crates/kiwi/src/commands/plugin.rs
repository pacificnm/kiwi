use kiwi_core::plugins::{install_plugin, reinstall_plugin, remove_plugin};

use crate::cli::PluginSubcommand;
use crate::plugins::{default_registry_path, PluginRegistry};

/// Dispatch a `kiwi plugin` subcommand. Returns an exit code (0 = success).
pub fn run(sub: &PluginSubcommand) -> i32 {
    let registry_path = match default_registry_path() {
        Some(p) => p,
        None => {
            eprintln!("error: could not determine HOME directory");
            return 1;
        }
    };

    match sub {
        PluginSubcommand::List => cmd_list(&registry_path),
        PluginSubcommand::Info { name } => cmd_info(&registry_path, name),
        PluginSubcommand::Enable { name } => cmd_enable(&registry_path, name),
        PluginSubcommand::Disable { name } => cmd_disable(&registry_path, name),
        PluginSubcommand::Install { path } => cmd_install(&registry_path, path),
        PluginSubcommand::Remove { name } => cmd_remove(&registry_path, name),
        PluginSubcommand::Reinstall { path } => cmd_reinstall(&registry_path, path),
        PluginSubcommand::Reload => cmd_reload(&registry_path),
    }
}

fn load_registry(registry_path: &std::path::Path) -> (PluginRegistry, i32) {
    let (registry, warnings) = PluginRegistry::load(registry_path);
    for w in &warnings {
        eprintln!("warning: {w}");
    }
    let code = if warnings.is_empty() { 0 } else { 0 }; // warnings don't fail
    (registry, code)
}

fn save_registry(registry: &PluginRegistry, path: &std::path::Path) -> i32 {
    match registry.save(path) {
        Ok(()) => 0,
        Err(e) => {
            eprintln!("error: {e}");
            1
        }
    }
}

fn cmd_list(registry_path: &std::path::Path) -> i32 {
    let (registry, _) = load_registry(registry_path);
    let entries = registry.entries_sorted();
    if entries.is_empty() {
        println!("No plugins registered. Install one with `kiwi plugin install <path>`.");
        return 0;
    }
    println!("{:<20} {:<10} {:<10} {}", "NAME", "VERSION", "STATUS", "DESCRIPTION");
    println!("{}", "-".repeat(70));
    for entry in entries {
        let status = if entry.enabled { "enabled" } else { "disabled" };
        let display = entry
            .display_name
            .as_deref()
            .unwrap_or(entry.name.as_str());
        println!("{:<20} {:<10} {:<10} {}", entry.name, entry.version, status, display);
    }
    0
}

fn cmd_info(registry_path: &std::path::Path, name: &str) -> i32 {
    let (registry, _) = load_registry(registry_path);
    match registry.get(name) {
        None => {
            eprintln!("error: plugin `{name}` is not in the registry");
            1
        }
        Some(entry) => {
            println!("Name:      {}", entry.name);
            if let Some(ref dn) = entry.display_name {
                println!("Display:   {dn}");
            }
            println!("Version:   {}", entry.version);
            println!("Enabled:   {}", entry.enabled);
            println!("Source:    {}", entry.source);
            println!("Path:      {}", entry.installed_path.display());
            println!("Library:   {}", entry.entry);
            0
        }
    }
}

fn cmd_enable(registry_path: &std::path::Path, name: &str) -> i32 {
    let (mut registry, _) = load_registry(registry_path);
    if !registry.enable(name) {
        eprintln!("error: plugin `{name}` is not in the registry");
        return 1;
    }
    let code = save_registry(&registry, registry_path);
    if code == 0 {
        println!("Plugin `{name}` enabled. Restart kiwi for the change to take effect.");
    }
    code
}

fn cmd_disable(registry_path: &std::path::Path, name: &str) -> i32 {
    let (mut registry, _) = load_registry(registry_path);
    if !registry.disable(name) {
        eprintln!("error: plugin `{name}` is not in the registry");
        return 1;
    }
    let code = save_registry(&registry, registry_path);
    if code == 0 {
        println!("Plugin `{name}` disabled. Restart kiwi for the change to take effect.");
    }
    code
}

fn cmd_install(registry_path: &std::path::Path, src_path: &std::path::Path) -> i32 {
    let result = match install_plugin(src_path) {
        Ok(result) => result,
        Err(e) => {
            eprintln!("error: {e}");
            return 1;
        }
    };

    let (mut registry, _) = load_registry(registry_path);
    registry.register(result.entry.clone());

    let code = save_registry(&registry, registry_path);
    if code == 0 {
        for message in &result.messages {
            println!("{message}");
        }
        println!("Restart kiwi to load the plugin.");
    }
    code
}

fn cmd_remove(registry_path: &std::path::Path, name: &str) -> i32 {
    let disk_result = remove_plugin(name);
    let (mut registry, _) = load_registry(registry_path);
    let registry_entry = registry.remove(name);

    match (disk_result, registry_entry) {
        (Ok(result), _) => {
            let code = save_registry(&registry, registry_path);
            if code == 0 {
                for message in &result.messages {
                    println!("{message}");
                }
                println!("Restart kiwi for the change to take effect.");
            }
            code
        }
        (Err(disk_err), Some(_)) => {
            let code = save_registry(&registry, registry_path);
            if code == 0 {
                eprintln!("warning: {disk_err}");
                println!(
                    "Plugin `{name}` removed from registry (install directory was already missing)."
                );
            }
            code
        }
        (Err(disk_err), None) => {
            eprintln!("error: {disk_err}");
            1
        }
    }
}

fn cmd_reinstall(registry_path: &std::path::Path, src_path: &std::path::Path) -> i32 {
    let result = match reinstall_plugin(src_path) {
        Ok(result) => result,
        Err(e) => {
            eprintln!("error: {e}");
            return 1;
        }
    };

    let (mut registry, _) = load_registry(registry_path);
    registry.register(result.entry.clone());

    let code = save_registry(&registry, registry_path);
    if code == 0 {
        for message in &result.messages {
            println!("{message}");
        }
        println!("Restart kiwi to load the plugin.");
    }
    code
}

fn cmd_reload(registry_path: &std::path::Path) -> i32 {
    let (registry, _) = load_registry(registry_path);
    let code = save_registry(&registry, registry_path);
    if code == 0 {
        println!("Registry reloaded and saved ({} plugin(s)).", registry.len());
    }
    code
}
