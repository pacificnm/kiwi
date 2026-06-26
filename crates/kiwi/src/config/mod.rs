//! TUI-facing config API; domain types live in [`kiwi_core::config`].

use std::path::Path;

pub use kiwi_core::config::load_config as load_config_with_options;
pub use kiwi_core::config::{
    persist_user_theme, project_has_theme_override, AgentSettings, ConfigError, ConfigLoadOptions,
    EditorSettings, MouseMode, MouseSettings, PluginsSettings, ResolvedConfig, ShellSettings,
    ThemeSettings,
};

use crate::cli::Cli;

pub fn load_config(cli: &Cli, repo_root: &Path) -> Result<ResolvedConfig, ConfigError> {
    load_config_with_options(&ConfigLoadOptions::from(cli), repo_root)
}

impl From<&Cli> for ConfigLoadOptions {
    fn from(cli: &Cli) -> Self {
        Self {
            config_path: cli.config.clone(),
            theme: cli.theme.clone(),
            left_width: cli.left_width,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::{Path, PathBuf};

    use clap::Parser;

    use super::*;
    use crate::cli::Cli;

    struct TestHome {
        home: PathBuf,
    }

    impl TestHome {
        fn new(name: &str) -> Self {
            let home = std::env::temp_dir().join(format!("kiwi-config-cli-test-{name}"));
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
    fn cli_wrapper_applies_overrides() {
        let home = TestHome::new("cli-wrapper");
        let repo = std::env::temp_dir().join("kiwi-config-cli-test-repo");
        let _ = fs::remove_dir_all(&repo);
        fs::create_dir_all(&repo).expect("create repo");
        fs::write(
            repo.join(".kiwi.toml"),
            "[theme]\nname = \"project-theme\"\n[app]\nleft_width = 25\n",
        )
        .expect("write project config");

        let cli = Cli::parse_from([
            "kiwi",
            "--theme",
            "dracula",
            "--left-width",
            "40",
            repo.to_str().expect("utf8 path"),
        ]);
        let config = kiwi_core::config::load_config_with_home(
            &ConfigLoadOptions::from(&cli),
            &repo,
            Some(home.home.clone()),
        )
        .expect("load config");

        assert_eq!(config.theme.name, "dracula");
        assert_eq!(config.app.left_width, 40);

        let _ = fs::remove_dir_all(repo);
    }

    #[test]
    fn load_config_uses_defaults_without_files() {
        let home = TestHome::new("cli-path");
        let cli = Cli::parse_from(["kiwi", "/tmp/repo"]);
        let config = kiwi_core::config::load_config_with_home(
            &ConfigLoadOptions::from(&cli),
            Path::new("/tmp/repo"),
            Some(home.home.clone()),
        )
        .expect("load config");
        assert_eq!(config.theme.name, "kiwi-dark");
    }
}
