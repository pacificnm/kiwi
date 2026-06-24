use std::fmt;
use std::path::PathBuf;

use crate::cli::Cli;
use crate::config::{load_config, ConfigError, ResolvedConfig};
use crate::terminal::{TerminalError, TerminalGuard};

pub struct StartupContext {
    #[allow(dead_code)]
    pub repo_root: PathBuf,
    #[allow(dead_code)]
    pub config: ResolvedConfig,
    pub terminal: TerminalGuard,
}

#[derive(Debug)]
pub enum StartupError {
    Config(ConfigError),
    Terminal(TerminalError),
}

impl fmt::Display for StartupError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Config(err) => write!(f, "{err}"),
            Self::Terminal(err) => write!(f, "{err}"),
        }
    }
}

impl std::error::Error for StartupError {}

pub fn init(cli: &Cli) -> Result<StartupContext, StartupError> {
    let repo_root = cli.path.clone();
    let config = load_config(cli, &repo_root).map_err(StartupError::Config)?;
    let terminal = TerminalGuard::init(&config.mouse).map_err(StartupError::Terminal)?;

    Ok(StartupContext {
        repo_root,
        config,
        terminal,
    })
}
