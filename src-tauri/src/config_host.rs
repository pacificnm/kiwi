//! Kiwi config file locations for dev trees vs installed binaries.

use std::path::{Path, PathBuf};

use nest_config::resolve_search;

/// Environment override for the active `config.toml`.
const CONFIG_ENV: &str = "KIWI_CONFIG";

/// Returns the Kiwi XDG config directory (`~/.config/kiwi` on Linux).
pub fn kiwi_config_dir() -> PathBuf {
    dirs::config_dir()
        .map(|dir| dir.join("kiwi"))
        .or_else(|| {
            std::env::var_os("HOME").map(|home| {
                PathBuf::from(home)
                    .join(".config")
                    .join("kiwi")
            })
        })
        .unwrap_or_else(|| PathBuf::from(".config").join("kiwi"))
}

/// Default installed config path: `~/.config/kiwi/config.toml`.
pub fn kiwi_home_config_path() -> PathBuf {
    kiwi_config_dir().join("config.toml")
}

/// Default MCP config beside the Kiwi config: `~/.config/kiwi/mcp.json`.
pub fn kiwi_home_mcp_path() -> PathBuf {
    kiwi_config_dir().join("mcp.json")
}

/// True when running from the Tauri dev tree (`src-tauri/` with `tauri.conf.json`).
pub fn is_dev_layout() -> bool {
    PathBuf::from("tauri.conf.json").exists()
}

/// Locates Kiwi `config.toml`.
///
/// Precedence:
/// 1. `KIWI_CONFIG` env (explicit file)
/// 2. Dev tree: `../desktop/config.toml` or `desktop/config.toml` (debug / `tauri dev` only)
/// 3. Installed: `~/.config/kiwi/config.toml`
/// 4. `nest_config::resolve_search("kiwi")` (cwd `config.toml`, etc.)
pub fn resolve_config_path() -> Option<PathBuf> {
    if let Some(path) = config_from_env() {
        return Some(path);
    }

    if should_prefer_dev_config() {
        if let Some(path) = dev_config_path() {
            return Some(path);
        }
    }

    let home = kiwi_home_config_path();
    if home.is_file() {
        return Some(home);
    }

    resolve_search("kiwi")
}

/// Resolves a config-relative path (e.g. `mcp.json`, `../../swift/config.toml`).
pub fn resolve_config_relative(config_path: &Path, relative: &str) -> PathBuf {
    let path = Path::new(relative);
    if path.is_absolute() {
        return path.to_path_buf();
    }
    let base = config_path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    base.join(path)
}

fn config_from_env() -> Option<PathBuf> {
    let raw = std::env::var(CONFIG_ENV).ok()?;
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return None;
    }
    let path = PathBuf::from(trimmed);
    path.is_file().then_some(path)
}

fn should_prefer_dev_config() -> bool {
    cfg!(debug_assertions) || is_dev_layout()
}

fn dev_config_path() -> Option<PathBuf> {
    for candidate in ["../desktop/config.toml", "desktop/config.toml"] {
        let path = PathBuf::from(candidate);
        if path.is_file() {
            return Some(path);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn resolve_config_path_prefers_desktop_config_in_dev_layout() {
        let dir = tempdir().unwrap();
        let desktop = dir.path().join("desktop");
        fs::create_dir_all(&desktop).unwrap();
        fs::write(desktop.join("config.toml"), "[project]\nname = \"x\"\n").unwrap();

        let original = std::env::current_dir().unwrap();
        let tauri_dir = dir.path().join("src-tauri");
        fs::create_dir_all(&tauri_dir).unwrap();
        fs::write(tauri_dir.join("tauri.conf.json"), "{}").unwrap();
        std::env::set_current_dir(&tauri_dir).unwrap();

        let path = resolve_config_path().unwrap();
        assert_eq!(path, PathBuf::from("../desktop/config.toml"));

        std::env::set_current_dir(original).unwrap();
    }

    #[test]
    fn resolve_config_path_uses_home_config_when_not_dev_layout() {
        let dir = tempdir().unwrap();
        let home = dir.path().join("home");
        let config = home.join(".config").join("kiwi");
        fs::create_dir_all(&config).unwrap();
        fs::write(config.join("config.toml"), "[project]\nname = \"x\"\n").unwrap();

        let original_home = std::env::var("HOME").ok();
        let original_cwd = std::env::current_dir().unwrap();
        let empty_cwd = dir.path().join("cwd");
        fs::create_dir_all(&empty_cwd).unwrap();

        unsafe {
            std::env::set_var("HOME", &home);
        }
        std::env::set_current_dir(&empty_cwd).unwrap();

        let path = resolve_config_path().unwrap();
        assert_eq!(path, config.join("config.toml"));

        std::env::set_current_dir(original_cwd).unwrap();
        if let Some(value) = original_home {
            unsafe {
                std::env::set_var("HOME", value);
            }
        } else {
            unsafe {
                std::env::remove_var("HOME");
            }
        }
    }

    #[test]
    fn resolve_config_relative_from_home_config_dir() {
        let config = PathBuf::from("/home/user/.config/kiwi/config.toml");
        let mcp = resolve_config_relative(&config, "mcp.json");
        assert_eq!(mcp, PathBuf::from("/home/user/.config/kiwi/mcp.json"));
    }
}
