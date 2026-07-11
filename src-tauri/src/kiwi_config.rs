//! Raw read/write access to Kiwi's own `config.toml`, for the "Kiwi Config"
//! Settings item — the file's contents *are* Kiwi's default settings (agent,
//! ai, project, swift, …), so this exposes it directly rather than
//! duplicating each field as its own typed setting.

use std::fs;
use std::path::PathBuf;

use nest_error::{NestError, NestResult};

use crate::config_host::{kiwi_home_config_path, resolve_config_path};

/// Resolves the config file this editor should read/write — the same file
/// [`crate::config_host::resolve_config_path`] found at startup, or the
/// default installed location (`~/.config/kiwi/config.toml`) if none exists
/// yet (first run).
pub fn path() -> PathBuf {
    resolve_config_path().unwrap_or_else(kiwi_home_config_path)
}

/// Absolute form of [`path`], for display in the UI — `resolve_config_path`
/// can return a dev-tree-relative path (`../desktop/config.toml`), which is
/// only meaningful relative to the process's CWD and confusing to show as-is.
pub fn display_path() -> String {
    let resolved = path();
    fs::canonicalize(&resolved)
        .unwrap_or_else(|_| {
            std::env::current_dir()
                .map(|cwd| cwd.join(&resolved))
                .unwrap_or(resolved)
        })
        .display()
        .to_string()
}

/// Reads the config file's raw text, or an empty string if it doesn't exist yet.
pub fn read() -> NestResult<String> {
    let path = path();
    if !path.is_file() {
        return Ok(String::new());
    }
    fs::read_to_string(&path)
        .map_err(|error| NestError::io(format!("failed to read {}: {error}", path.display())))
}

/// Validates `content` as TOML, then writes it to the config file (creating
/// parent directories on first write).
pub fn write(content: &str) -> NestResult<()> {
    toml::from_str::<toml::Value>(content)
        .map_err(|error| NestError::validation(format!("invalid TOML: {error}")))?;

    let path = path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| {
            NestError::io(format!("failed to create {}: {error}", parent.display()))
        })?;
    }
    fs::write(&path, content)
        .map_err(|error| NestError::io(format!("failed to write {}: {error}", path.display())))
}
