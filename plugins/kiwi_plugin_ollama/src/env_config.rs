//! Load repo `.env` and configure MCP embedding defaults.

use std::path::Path;

/// Load key=value pairs from a `.env` file without overwriting existing env vars.
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

/// Prefer OpenAI embeddings when an API key is available (matches indexed tables).
pub fn configure_mcp_embed_env() {
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

pub fn load_repo_env(repo: &Path) {
    load_dotenv(&repo.join(".env"));
    configure_mcp_embed_env();
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn load_dotenv_sets_unset_variables() {
        let dir = std::env::temp_dir().join(format!(
            "kiwi-env-test-{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).expect("dir");
        let env_path = dir.join(".env");
        fs::write(&env_path, "KIWI_ENV_TEST_KEY=from-dotenv\n").expect("write");

        let key = "KIWI_ENV_TEST_KEY";
        std::env::remove_var(key);
        load_dotenv(&env_path);
        assert_eq!(
            std::env::var(key).expect("loaded"),
            "from-dotenv"
        );
        std::env::remove_var(key);
        let _ = fs::remove_dir_all(&dir);
    }
}
