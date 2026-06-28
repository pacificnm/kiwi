use std::fs;
use std::path::PathBuf;

use kiwi_core::config::default_plugins_directory;
use kiwi_plugin_api::PluginManifest;

use crate::cli::PluginSubcommand;
use crate::plugins::{default_registry_path, PluginRegistry, PluginRegistryEntry};

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
    // Read and validate the manifest from the source directory.
    let manifest_path = src_path.join("plugin.toml");
    let manifest_str = match fs::read_to_string(&manifest_path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("error: could not read {}: {e}", manifest_path.display());
            return 1;
        }
    };
    let manifest: PluginManifest = match toml::from_str(&manifest_str) {
        Ok(m) => m,
        Err(e) => {
            eprintln!("error: invalid plugin.toml: {e}");
            return 1;
        }
    };

    let home = match std::env::var_os("HOME") {
        Some(h) => PathBuf::from(h),
        None => {
            eprintln!("error: could not determine HOME directory");
            return 1;
        }
    };
    let plugins_dir = default_plugins_directory(Some(&home));
    let dest_dir = plugins_dir.join(&manifest.name);

    if dest_dir.exists() {
        eprintln!(
            "error: plugin directory already exists at {}. Remove it first or use `kiwi plugin remove {}`.",
            dest_dir.display(),
            manifest.name
        );
        return 1;
    }

    if let Err(e) = copy_dir_recursive(src_path, &dest_dir) {
        eprintln!("error: failed to copy plugin: {e}");
        return 1;
    }

    // Find the shared library filename.
    let ext = if cfg!(target_os = "macos") { "dylib" } else { "so" };
    let lib_filename = find_library_filename(&dest_dir, ext).unwrap_or_else(|| manifest.entry.clone());

    let (mut registry, _) = load_registry(registry_path);
    registry.register(PluginRegistryEntry {
        name: manifest.name.clone(),
        display_name: manifest.display_name.clone(),
        version: manifest.version.clone(),
        enabled: true,
        installed_path: dest_dir.clone(),
        entry: lib_filename,
        source: "local".to_string(),
    });

    let code = save_registry(&registry, registry_path);
    if code == 0 {
        println!(
            "Plugin `{}` installed to {}. Restart kiwi to load it.",
            manifest.name,
            dest_dir.display()
        );
    }
    code
}

fn cmd_remove(registry_path: &std::path::Path, name: &str) -> i32 {
    let (mut registry, _) = load_registry(registry_path);
    match registry.remove(name) {
        None => {
            eprintln!("error: plugin `{name}` is not in the registry");
            1
        }
        Some(entry) => {
            let code = save_registry(&registry, registry_path);
            if code == 0 {
                println!(
                    "Plugin `{name}` removed from registry. Files remain at {}.",
                    entry.installed_path.display()
                );
            }
            code
        }
    }
}

fn cmd_reload(registry_path: &std::path::Path) -> i32 {
    let (registry, _) = load_registry(registry_path);
    let code = save_registry(&registry, registry_path);
    if code == 0 {
        println!("Registry reloaded and saved ({} plugin(s)).", registry.len());
    }
    code
}

fn copy_dir_recursive(src: &std::path::Path, dest: &std::path::Path) -> std::io::Result<()> {
    fs::create_dir_all(dest)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let dest_path = dest.join(entry.file_name());
        if entry.file_type()?.is_dir() {
            copy_dir_recursive(&entry.path(), &dest_path)?;
        } else {
            fs::copy(entry.path(), dest_path)?;
        }
    }
    Ok(())
}

fn find_library_filename(dir: &std::path::Path, ext: &str) -> Option<String> {
    let Ok(entries) = fs::read_dir(dir) else { return None };
    for entry in entries.flatten() {
        let name = entry.file_name();
        let name_str = name.to_string_lossy();
        if name_str.ends_with(&format!(".{ext}")) {
            return Some(name_str.into_owned());
        }
    }
    None
}
