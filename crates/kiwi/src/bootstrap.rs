use std::fmt;
use std::path::PathBuf;

use crate::cli::Cli;
use crate::config::{load_config, ConfigError, ResolvedConfig};
use crate::repo::{resolve_repo_root, warn_if_not_git_repo, RepoError};
use crate::terminal::{TerminalError, TerminalGuard};
use crate::theme::{load_theme, ThemeError, ThemePalette};

pub struct StartupContext {
    #[allow(dead_code)]
    pub repo_root: PathBuf,
    #[allow(dead_code)]
    pub is_git_repo: bool,
    #[allow(dead_code)]
    pub config: ResolvedConfig,
    #[allow(dead_code)]
    pub theme: ThemePalette,
    pub terminal: TerminalGuard,
}

#[derive(Debug)]
pub enum StartupError {
    Config(ConfigError),
    Repo(RepoError),
    Theme(ThemeError),
    Terminal(TerminalError),
}

impl fmt::Display for StartupError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Config(err) => write!(f, "{err}"),
            Self::Repo(err) => write!(f, "{err}"),
            Self::Theme(err) => write!(f, "{err}"),
            Self::Terminal(err) => write!(f, "{err}"),
        }
    }
}

impl std::error::Error for StartupError {}

pub fn init(cli: &Cli) -> Result<StartupContext, StartupError> {
    let repo = resolve_repo_root(&cli.path).map_err(StartupError::Repo)?;
    warn_if_not_git_repo(&repo);

    let config = load_config(cli, &repo.path).map_err(StartupError::Config)?;
    let theme = load_theme(&config.theme).map_err(StartupError::Theme)?;
    let terminal = TerminalGuard::init(&config.mouse).map_err(StartupError::Terminal)?;

    Ok(StartupContext {
        repo_root: repo.path,
        is_git_repo: repo.is_git_repo,
        config,
        theme,
        terminal,
    })
}
