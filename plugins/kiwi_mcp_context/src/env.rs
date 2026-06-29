//! Load repo `.env` and Kiwi Settings API keys when MCP is spawned with `cwd` set.

use std::path::{Path, PathBuf};

const PROVIDER_KEY_VARS: [(&str, &str); 3] = [
    ("OPENAI_API_KEY", "openai"),
    ("ANTHROPIC_API_KEY", "claude"),
    ("CURSOR_API_KEY", "cursor"),
];

fn is_provider_key(key: &str) -> bool {
    PROVIDER_KEY_VARS.iter().any(|(env_var, _)| *env_var == key)
}

fn dotenv_paths() -> Vec<PathBuf> {
    let mut paths = Vec::new();
    if let Ok(cwd) = std::env::current_dir() {
        paths.push(cwd.join(".env"));
    }
    if let Ok(root) = std::env::var("KIWI_REPO_ROOT") {
        paths.push(Path::new(&root).join(".env"));
    }
    paths
}

fn load_dotenv_filtered(path: &Path, provider_keys_only: bool) {
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
        if key.is_empty() {
            continue;
        }
        let is_provider = is_provider_key(key);
        if provider_keys_only {
            if !is_provider || std::env::var(key).is_ok() {
                continue;
            }
        } else if is_provider {
            continue;
        } else if std::env::var(key).is_ok() {
            continue;
        }
        let value = value.trim().trim_matches('"').trim_matches('\'');
        if !value.is_empty() {
            std::env::set_var(key, value);
        }
    }
}

fn kiwi_config_path() -> Option<PathBuf> {
    std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".config/kiwi/config.toml"))
}

fn provider_api_key(config: &toml::Table, provider: &str) -> Option<String> {
    let key = config
        .get("agent")?
        .as_table()?
        .get("providers")?
        .as_table()?
        .get(provider)?
        .as_table()?
        .get("api_key")?
        .as_str()?
        .trim();
    if key.is_empty() {
        None
    } else {
        Some(key.to_owned())
    }
}

/// Read API keys from `~/.config/kiwi/config.toml`, overriding `.env` values.
///
/// Matches `tools/memory_common.py::load_kiwi_config_keys` — Kiwi Settings is the
/// source of truth when keys are rotated in the UI.
pub fn load_kiwi_config_keys() {
    let Some(path) = kiwi_config_path() else {
        return;
    };
    let Ok(content) = std::fs::read_to_string(&path) else {
        return;
    };
    let Ok(config) = content.parse::<toml::Table>() else {
        return;
    };

    for (env_var, provider) in PROVIDER_KEY_VARS {
        if let Some(key) = provider_api_key(&config, provider) {
            std::env::set_var(env_var, key);
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
    for path in dotenv_paths() {
        load_dotenv_filtered(&path, false);
    }
    load_kiwi_config_keys();
    for path in dotenv_paths() {
        load_dotenv_filtered(&path, true);
    }
    configure_mcp_embed_env();
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Mutex, OnceLock};

    fn env_test_lock() -> &'static Mutex<()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(()))
    }

    #[test]
    fn provider_api_key_reads_agent_providers_section() {
        let _guard = env_test_lock().lock().expect("lock");
        let config: toml::Table = r#"
            [agent.providers.openai]
            api_key = "sk-test-openai"
        "#
        .parse()
        .expect("toml");

        assert_eq!(
            provider_api_key(&config, "openai").as_deref(),
            Some("sk-test-openai")
        );
        assert!(provider_api_key(&config, "claude").is_none());
    }

    #[test]
    fn load_kiwi_config_keys_sets_env_from_config_file() {
        let _guard = env_test_lock().lock().expect("lock");
        let dir = std::env::temp_dir().join(format!(
            "kiwi-mcp-env-test-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        let config_dir = dir.join(".config").join("kiwi");
        std::fs::create_dir_all(&config_dir).expect("config dir");
        std::fs::write(
            config_dir.join("config.toml"),
            "[agent.providers.openai]\napi_key = \"sk-from-kiwi-config\"\n",
        )
        .expect("write config");

        let home_key = "HOME";
        let openai_key = "OPENAI_API_KEY";
        let prior_home = std::env::var(home_key).ok();
        let prior_openai = std::env::var(openai_key).ok();

        std::env::set_var(home_key, &dir);
        std::env::set_var(openai_key, "stale-from-env");

        load_kiwi_config_keys();

        assert_eq!(
            std::env::var(openai_key).expect("openai key"),
            "sk-from-kiwi-config"
        );

        match prior_openai {
            Some(value) => std::env::set_var(openai_key, value),
            None => std::env::remove_var(openai_key),
        }
        match prior_home {
            Some(value) => std::env::set_var(home_key, value),
            None => std::env::remove_var(home_key),
        }
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn load_mcp_env_prefers_kiwi_config_over_dotenv_provider_key() {
        let _guard = env_test_lock().lock().expect("lock");
        let dir = std::env::temp_dir().join(format!(
            "kiwi-mcp-env-order-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        let config_dir = dir.join(".config").join("kiwi");
        std::fs::create_dir_all(&config_dir).expect("config dir");
        std::fs::write(
            config_dir.join("config.toml"),
            "[agent.providers.openai]\napi_key = \"sk-from-kiwi-config\"\n",
        )
        .expect("write config");
        std::fs::write(
            dir.join(".env"),
            "OPENAI_API_KEY=sk-stale-from-dotenv\nDATABASE_URL=postgres://example\n",
        )
        .expect("write env");

        let home_key = "HOME";
        let openai_key = "OPENAI_API_KEY";
        let database_key = "DATABASE_URL";
        let prior_home = std::env::var(home_key).ok();
        let prior_openai = std::env::var(openai_key).ok();
        let prior_database = std::env::var(database_key).ok();

        std::env::set_var(home_key, &dir);
        std::env::remove_var(openai_key);
        std::env::remove_var(database_key);
        std::env::set_current_dir(&dir).expect("cwd");

        load_mcp_env();

        assert_eq!(
            std::env::var(openai_key).expect("openai key"),
            "sk-from-kiwi-config"
        );
        assert_eq!(
            std::env::var(database_key).expect("database url"),
            "postgres://example"
        );

        match prior_openai {
            Some(value) => std::env::set_var(openai_key, value),
            None => std::env::remove_var(openai_key),
        }
        match prior_database {
            Some(value) => std::env::set_var(database_key, value),
            None => std::env::remove_var(database_key),
        }
        match prior_home {
            Some(value) => std::env::set_var(home_key, value),
            None => std::env::remove_var(home_key),
        }
        let _ = std::fs::remove_dir_all(&dir);
    }
}
