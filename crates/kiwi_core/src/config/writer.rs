use std::fs;
use std::path::{Path, PathBuf};

use super::error::ConfigError;

pub fn user_config_path(home: &Path) -> PathBuf {
    home.join(".config/kiwi/config.toml")
}

pub fn project_has_theme_override(repo_root: &Path) -> bool {
    let path = repo_root.join(".kiwi.toml");
    if !path.is_file() {
        return false;
    }

    let Ok(content) = fs::read_to_string(&path) else {
        return false;
    };
    let Ok(raw) = toml::from_str::<toml::Value>(&content) else {
        return false;
    };

    raw.get("theme")
        .and_then(|theme| theme.get("name"))
        .and_then(|name| name.as_str())
        .is_some_and(|name| !name.trim().is_empty())
}

pub fn persist_user_theme(home: &Path, name: &str) -> Result<(), ConfigError> {
    let path = user_config_path(home);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|err| ConfigError::io(path.clone(), err))?;
    }

    let mut doc = if path.is_file() {
        let content =
            fs::read_to_string(&path).map_err(|err| ConfigError::io(path.clone(), err))?;
        toml::from_str(&content).map_err(|err| ConfigError::parse(path.clone(), &content, err))?
    } else {
        toml::Value::Table(toml::map::Map::new())
    };

    let Some(table) = doc.as_table_mut() else {
        return Err(ConfigError::validation("config root must be a table"));
    };

    let theme = table
        .entry("theme")
        .or_insert_with(|| toml::Value::Table(toml::map::Map::new()));
    let Some(theme_table) = theme.as_table_mut() else {
        return Err(ConfigError::validation("[theme] must be a table"));
    };

    theme_table.insert("name".to_string(), toml::Value::String(name.to_string()));
    theme_table.remove("custom");

    let serialized =
        toml::to_string_pretty(&doc).map_err(|err| ConfigError::validation(err.to_string()))?;
    fs::write(&path, serialized).map_err(|err| ConfigError::io(path, err))?;
    Ok(())
}

/// Persist API-mode agent settings to the user config.
///
/// Writes `[agent] active` and `[agent.providers.<provider>]` so each provider
/// keeps its own key/model/url and switching providers never clobbers another's settings.
/// Also writes the legacy `mode`/`provider`/`model` flat fields for backward compat.
pub fn persist_user_agent_mode(
    home: &Path,
    provider: &str,
    model: &str,
    api_key_env: Option<&str>,
    api_url: Option<&str>,
) -> Result<(), ConfigError> {
    let path = user_config_path(home);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|err| ConfigError::io(path.clone(), err))?;
    }

    let mut doc = if path.is_file() {
        let content =
            fs::read_to_string(&path).map_err(|err| ConfigError::io(path.clone(), err))?;
        toml::from_str(&content).map_err(|err| ConfigError::parse(path.clone(), &content, err))?
    } else {
        toml::Value::Table(toml::map::Map::new())
    };

    let Some(root) = doc.as_table_mut() else {
        return Err(ConfigError::validation("config root must be a table"));
    };

    let agent = root
        .entry("agent")
        .or_insert_with(|| toml::Value::Table(toml::map::Map::new()));
    let Some(agent_table) = agent.as_table_mut() else {
        return Err(ConfigError::validation("[agent] must be a table"));
    };

    // Top-level agent fields.
    agent_table.insert("mode".to_string(),     toml::Value::String("api".to_string()));
    agent_table.insert("active".to_string(),   toml::Value::String(provider.to_string()));
    // Legacy flat fields so old readers still work.
    agent_table.insert("provider".to_string(), toml::Value::String(provider.to_string()));
    agent_table.insert("model".to_string(),    toml::Value::String(model.to_string()));

    // Per-provider sub-table: [agent.providers.<provider>]
    let providers = agent_table
        .entry("providers")
        .or_insert_with(|| toml::Value::Table(toml::map::Map::new()));
    let Some(providers_table) = providers.as_table_mut() else {
        return Err(ConfigError::validation("[agent.providers] must be a table"));
    };
    let provider_entry = providers_table
        .entry(provider.to_string())
        .or_insert_with(|| toml::Value::Table(toml::map::Map::new()));
    let Some(p) = provider_entry.as_table_mut() else {
        return Err(ConfigError::validation("provider entry must be a table"));
    };
    p.insert("model".to_string(), toml::Value::String(model.to_string()));
    if let Some(env) = api_key_env {
        p.insert("api_key_env".to_string(), toml::Value::String(env.to_string()));
    }
    if let Some(url) = api_url {
        p.insert("api_url".to_string(), toml::Value::String(url.to_string()));
    }

    let serialized =
        toml::to_string_pretty(&doc).map_err(|err| ConfigError::validation(err.to_string()))?;
    fs::write(&path, serialized).map_err(|err| ConfigError::io(path, err))?;
    Ok(())
}

/// Persist `[agent] command` and `args` to the user config without touching other settings.
pub fn persist_user_agent(home: &Path, command: &str, args: &[String]) -> Result<(), ConfigError> {
    let path = user_config_path(home);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|err| ConfigError::io(path.clone(), err))?;
    }

    let mut doc = if path.is_file() {
        let content =
            fs::read_to_string(&path).map_err(|err| ConfigError::io(path.clone(), err))?;
        toml::from_str(&content).map_err(|err| ConfigError::parse(path.clone(), &content, err))?
    } else {
        toml::Value::Table(toml::map::Map::new())
    };

    let Some(table) = doc.as_table_mut() else {
        return Err(ConfigError::validation("config root must be a table"));
    };

    let agent = table
        .entry("agent")
        .or_insert_with(|| toml::Value::Table(toml::map::Map::new()));
    let Some(agent_table) = agent.as_table_mut() else {
        return Err(ConfigError::validation("[agent] must be a table"));
    };

    agent_table.insert("command".to_string(), toml::Value::String(command.to_string()));
    agent_table.insert(
        "args".to_string(),
        toml::Value::Array(
            args.iter()
                .map(|a| toml::Value::String(a.clone()))
                .collect(),
        ),
    );

    let serialized =
        toml::to_string_pretty(&doc).map_err(|err| ConfigError::validation(err.to_string()))?;
    fs::write(&path, serialized).map_err(|err| ConfigError::io(path, err))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;

    struct TestHome {
        home: PathBuf,
    }

    impl TestHome {
        fn new(name: &str) -> Self {
            let home = std::env::temp_dir().join(format!("kiwi-config-writer-{name}"));
            let _ = fs::remove_dir_all(&home);
            fs::create_dir_all(&home).expect("create temp home");
            Self { home }
        }
    }

    impl Drop for TestHome {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.home);
        }
    }

    #[test]
    fn persist_user_theme_creates_file_when_missing() {
        let home = TestHome::new("create");
        persist_user_theme(&home.home, "dracula").expect("persist");

        let path = user_config_path(&home.home);
        let content = fs::read_to_string(path).expect("read config");
        assert!(content.contains("name = \"dracula\""));
    }

    #[test]
    fn persist_user_theme_updates_existing_without_clobbering_other_sections() {
        let home = TestHome::new("merge");
        let config_dir = home.home.join(".config/kiwi");
        fs::create_dir_all(&config_dir).expect("create config dir");
        fs::write(
            config_dir.join("config.toml"),
            "[editor]\ncommand = \"nvim\"\n[theme]\nname = \"kiwi-dark\"\n",
        )
        .expect("write config");

        persist_user_theme(&home.home, "nord").expect("persist");

        let content = fs::read_to_string(user_config_path(&home.home)).expect("read config");
        assert!(content.contains("command = \"nvim\""));
        assert!(content.contains("name = \"nord\""));
    }

    #[test]
    fn project_has_theme_override_detects_project_theme_name() {
        let repo = std::env::temp_dir().join("kiwi-config-writer-repo");
        let _ = fs::remove_dir_all(&repo);
        fs::create_dir_all(&repo).expect("create repo");
        fs::write(
            repo.join(".kiwi.toml"),
            "[theme]\nname = \"project-theme\"\n",
        )
        .expect("write project config");

        assert!(project_has_theme_override(&repo));

        let _ = fs::remove_dir_all(repo);
    }
}
