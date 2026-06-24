use std::fs;
use std::path::{Path, PathBuf};

use crate::cli::Cli;

use super::error::ConfigError;
use super::types::{RawConfig, ResolvedConfig};

pub fn load_config(cli: &Cli, repo_root: &Path) -> Result<ResolvedConfig, ConfigError> {
    load_config_with_home(cli, repo_root, home_dir())
}

pub(crate) fn load_config_with_home(
    cli: &Cli,
    repo_root: &Path,
    home: Option<PathBuf>,
) -> Result<ResolvedConfig, ConfigError> {
    let mut resolved = ResolvedConfig::default();

    if let Some(path) = resolve_user_config_path(cli, home.as_deref())? {
        if path.exists() {
            let raw = read_config_file(&path)?;
            raw.apply_to(&mut resolved, home.as_deref());
        } else if cli.config.is_some() {
            return Err(ConfigError::io(
                path,
                std::io::Error::from(std::io::ErrorKind::NotFound),
            ));
        }
    }

    let project_path = repo_root.join(".kiwi.toml");
    if project_path.exists() {
        let raw = read_config_file(&project_path)?;
        raw.apply_to(&mut resolved, home.as_deref());
    }

    apply_cli_overrides(cli, &mut resolved);
    validate(&resolved)?;

    Ok(resolved)
}

fn resolve_user_config_path(
    cli: &Cli,
    home: Option<&Path>,
) -> Result<Option<PathBuf>, ConfigError> {
    if let Some(path) = &cli.config {
        return Ok(Some(path.clone()));
    }
    Ok(home.map(|dir| dir.join(".config/kiwi/config.toml")))
}

fn home_dir() -> Option<PathBuf> {
    std::env::var_os("HOME").map(PathBuf::from)
}

fn read_config_file(path: &Path) -> Result<RawConfig, ConfigError> {
    let content = fs::read_to_string(path).map_err(|e| ConfigError::io(path.to_path_buf(), e))?;
    toml::from_str(&content).map_err(|e| ConfigError::parse(path.to_path_buf(), &content, e))
}

fn apply_cli_overrides(cli: &Cli, resolved: &mut ResolvedConfig) {
    if let Some(theme) = &cli.theme {
        resolved.theme.name = theme.clone();
    }
    if let Some(left_width) = cli.left_width {
        resolved.app.left_width = left_width;
    }
}

fn validate(config: &ResolvedConfig) -> Result<(), ConfigError> {
    if config.theme.name.trim().is_empty() {
        return Err(ConfigError::validation("theme.name must not be empty"));
    }

    if !(10..=50).contains(&config.app.left_width) {
        return Err(ConfigError::validation(
            "app.left_width must be between 10 and 50",
        ));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::{Path, PathBuf};

    use clap::Parser;

    use crate::cli::Cli;
    use crate::config::types::expand_tilde;

    use super::load_config_with_home;
    use super::read_config_file;

    struct TestHome {
        home: PathBuf,
    }

    impl TestHome {
        fn new(name: &str) -> Self {
            let home = std::env::temp_dir().join(format!("kiwi-config-test-{name}"));
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
    fn defaults_apply_when_no_files_exist() {
        let home = TestHome::new("defaults");
        let cli = Cli::parse_from(["kiwi", "/tmp/repo"]);
        let config = load_config_with_home(&cli, Path::new("/tmp/repo"), Some(home.home.clone()))
            .expect("load config");

        assert_eq!(config.app.left_width, 30);
        assert_eq!(config.theme.name, "kiwi-dark");
        assert_eq!(config.editor.command, "nvim");
    }

    #[test]
    fn project_config_overrides_user_config() {
        let home = TestHome::new("project-over-user");
        let user_dir = home.home.join(".config/kiwi");
        fs::create_dir_all(&user_dir).expect("create user config dir");
        fs::write(
            user_dir.join("config.toml"),
            "[editor]\ncommand = \"vim\"\n[theme]\nname = \"user-theme\"\n",
        )
        .expect("write user config");

        let repo = std::env::temp_dir().join("kiwi-config-test-repo-project");
        let _ = fs::remove_dir_all(&repo);
        fs::create_dir_all(&repo).expect("create repo");
        fs::write(
            repo.join(".kiwi.toml"),
            "[editor]\ncommand = \"nvim\"\n[theme]\nname = \"project-theme\"\n",
        )
        .expect("write project config");

        let cli = Cli::parse_from(["kiwi", repo.to_str().expect("utf8 path")]);
        let config =
            load_config_with_home(&cli, &repo, Some(home.home.clone())).expect("load config");

        assert_eq!(config.editor.command, "nvim");
        assert_eq!(config.theme.name, "project-theme");
    }

    #[test]
    fn cli_overrides_project_config() {
        let home = TestHome::new("cli-over-project");
        let repo = std::env::temp_dir().join("kiwi-config-test-repo-cli");
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
        let config =
            load_config_with_home(&cli, &repo, Some(home.home.clone())).expect("load config");

        assert_eq!(config.theme.name, "dracula");
        assert_eq!(config.app.left_width, 40);
    }

    #[test]
    fn expands_tilde_in_custom_theme_path() {
        let home = TestHome::new("tilde-expand");
        let repo = std::env::temp_dir().join("kiwi-config-test-repo-tilde");
        let _ = fs::remove_dir_all(&repo);
        fs::create_dir_all(&repo).expect("create repo");
        fs::write(
            repo.join(".kiwi.toml"),
            "[theme]\nname = \"custom\"\ncustom = \"~/themes/my.toml\"\n",
        )
        .expect("write project config");

        let cli = Cli::parse_from(["kiwi", repo.to_str().expect("utf8 path")]);
        let config =
            load_config_with_home(&cli, &repo, Some(home.home.clone())).expect("load config");

        assert_eq!(config.theme.custom, Some(home.home.join("themes/my.toml")));
        assert_eq!(
            expand_tilde("~/themes/my.toml", Some(&home.home)),
            home.home.join("themes/my.toml")
        );
    }

    #[test]
    fn invalid_toml_includes_line_number() {
        let path = std::env::temp_dir().join("kiwi-config-invalid.toml");
        fs::write(&path, "left_width = 30\n[app\n").expect("write invalid toml");

        let err = read_config_file(&path).expect_err("invalid toml should fail");
        assert!(err.message.contains("line"));

        let _ = fs::remove_file(path);
    }
}
