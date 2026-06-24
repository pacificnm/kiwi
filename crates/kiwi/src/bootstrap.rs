use std::path::PathBuf;

use crate::cli::Cli;
use crate::config::{load_config, ConfigError, ResolvedConfig};

pub struct StartupContext {
    #[allow(dead_code)]
    pub repo_root: PathBuf,
    #[allow(dead_code)]
    pub config: ResolvedConfig,
}

pub fn init(cli: &Cli) -> Result<StartupContext, ConfigError> {
    let repo_root = cli.path.clone();
    let config = load_config(cli, &repo_root)?;
    Ok(StartupContext { repo_root, config })
}
