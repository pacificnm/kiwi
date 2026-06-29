//! Load repo `.env` when MCP is spawned by Cursor with `cwd` set.

use std::path::Path;

pub fn load_dotenv(path: &Path) {
    let Ok(content) = std::fs::read_to_string(path) else {
        return;
    };

    for raw_line in content.lines() {
        let line = raw_line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        let key = key.trim();
        if key.is_empty() || std::env::var(key).is_ok() {
            continue;
        }
        let value = value.trim().trim_matches('"').trim_matches('\'');
        if !value.is_empty() {
            std::env::set_var(key, value);
        }
    }
}

fn configure_mcp_embed_env() {
    if std::env::var("EMBED_BACKEND").is_ok() {
        return;
    }
    let has_key = std::env::var("OPENAI_API_KEY")
        .ok()
        .is_some_and(|value| !value.trim().is_empty());
    if has_key {
        std::env::set_var("EMBED_BACKEND", "openai");
    }
}

pub fn load_mcp_env() {
    if let Ok(cwd) = std::env::current_dir() {
        load_dotenv(&cwd.join(".env"));
    }
    if let Ok(root) = std::env::var("KIWI_REPO_ROOT") {
        load_dotenv(&Path::new(&root).join(".env"));
    }
    configure_mcp_embed_env();
}
