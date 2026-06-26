//! Pre-eframe startup: repo validation, config, and theme (SPEC-021 G1 / #181).

use std::fmt;
use std::path::{Path, PathBuf};

use kiwi_core::config::{load_config_with_home, ConfigError, ConfigLoadOptions, ResolvedConfig};
use kiwi_core::repo::{resolve_repo_root, warn_if_not_git_repo, RepoError};
use kiwi_core::theme::{load_theme, ThemeError, ThemePalette};

use crate::cli::Cli;

pub struct GuiBootstrapContext {
    pub repo_root: PathBuf,
    pub is_git_repo: bool,
    pub config: ResolvedConfig,
    pub theme: ThemePalette,
}

#[derive(Debug)]
pub enum BootstrapError {
    Repo(RepoError),
    Config(ConfigError),
    Theme(ThemeError),
}

impl fmt::Display for BootstrapError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Repo(err) => write!(f, "{err}"),
            Self::Config(err) => write!(f, "{err}"),
            Self::Theme(err) => write!(f, "{err}"),
        }
    }
}

impl std::error::Error for BootstrapError {}

pub fn init(cli: &Cli) -> Result<GuiBootstrapContext, BootstrapError> {
    init_with_home(cli, home_dir())
}

fn home_dir() -> Option<PathBuf> {
    std::env::var_os("HOME").map(PathBuf::from)
}

pub(crate) fn init_with_home(
    cli: &Cli,
    home: Option<PathBuf>,
) -> Result<GuiBootstrapContext, BootstrapError> {
    let repo = resolve_repo_root(&cli.path).map_err(BootstrapError::Repo)?;
    warn_if_not_git_repo(&repo);

    let options = ConfigLoadOptions::from(cli);
    let config =
        load_config_with_home(&options, &repo.path, home).map_err(BootstrapError::Config)?;
    let theme = load_theme(&config.theme).map_err(BootstrapError::Theme)?;

    Ok(GuiBootstrapContext {
        repo_root: repo.path,
        is_git_repo: repo.is_git_repo,
        config,
        theme,
    })
}

#[must_use]
pub fn window_title(repo_root: &Path) -> String {
    let name = repo_root
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or(".");
    format!("Kiwi — {name}")
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;

    use clap::Parser;

    use kiwi_core::config::load_config_with_home;

    use super::{init_with_home, window_title, BootstrapError};
    use crate::cli::Cli;

    struct TestHome {
        home: PathBuf,
    }

    impl TestHome {
        fn new(name: &str) -> Self {
            let home = std::env::temp_dir().join(format!("kiwi-gui-bootstrap-{name}"));
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

    struct TempRepo {
        path: PathBuf,
    }

    impl TempRepo {
        fn new(name: &str) -> Self {
            let path = std::env::temp_dir().join(format!("kiwi-gui-bootstrap-repo-{name}"));
            let _ = fs::remove_dir_all(&path);
            fs::create_dir_all(&path).expect("create temp repo");
            Self { path }
        }
    }

    impl Drop for TempRepo {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.path);
        }
    }

    #[test]
    fn window_title_uses_directory_name() {
        assert_eq!(
            window_title(std::path::Path::new("/tmp/my-repo")),
            "Kiwi — my-repo"
        );
    }

    #[test]
    fn init_loads_project_config_and_theme_override() {
        let home = TestHome::new("init-config");
        let repo = TempRepo::new("init-config");
        fs::write(
            repo.path.join(".kiwi.toml"),
            "[theme]\nname = \"project-theme\"\n",
        )
        .expect("write project config");

        let cli = Cli::parse_from([
            "kiwi-gui",
            "--theme",
            "dracula",
            repo.path.to_str().expect("utf8 path"),
        ]);

        let ctx = init_with_home(&cli, Some(home.home.clone())).expect("bootstrap should succeed");

        assert_eq!(ctx.config.theme.name, "dracula");
        assert_eq!(ctx.theme.name, "dracula");
        assert_eq!(
            ctx.repo_root,
            fs::canonicalize(&repo.path).expect("canonicalize")
        );
    }

    #[test]
    fn init_rejects_missing_repo_path() {
        let missing = std::env::temp_dir().join("kiwi-gui-bootstrap-missing");
        let _ = fs::remove_dir_all(&missing);
        let cli = Cli::parse_from(["kiwi-gui", missing.to_str().expect("utf8 path")]);

        let err = init_with_home(&cli, None);
        assert!(matches!(err, Err(BootstrapError::Repo(_))));
    }

    #[test]
    fn config_options_from_cli_match_core_loader() {
        let home = TestHome::new("options");
        let repo = TempRepo::new("options");
        let cli = Cli::parse_from([
            "kiwi-gui",
            "--theme",
            "kiwi-light",
            repo.path.to_str().expect("utf8 path"),
        ]);

        let config = load_config_with_home(
            &kiwi_core::config::ConfigLoadOptions::from(&cli),
            &repo.path,
            Some(home.home.clone()),
        )
        .expect("load config");

        assert_eq!(config.theme.name, "kiwi-light");
    }

    #[test]
    fn init_uses_isolated_home_for_defaults() {
        let home = TestHome::new("default-home");
        let repo = TempRepo::new("default-home");
        let cli = Cli::parse_from(["kiwi-gui", repo.path.to_str().expect("utf8 path")]);
        let ctx = init_with_home(&cli, Some(home.home.clone())).expect("bootstrap");
        assert_eq!(ctx.theme.name, "kiwi-dark");
    }
}
