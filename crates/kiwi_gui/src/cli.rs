use std::path::PathBuf;

use clap::Parser;

/// Desktop GUI for the Kiwi AI development workspace.
#[derive(Debug, Clone, PartialEq, Eq, Parser)]
#[command(name = "kiwi-gui", version, about)]
pub struct Cli {
    /// Repository or workspace root directory.
    #[arg(value_name = "PATH", default_value = ".")]
    pub path: PathBuf,

    /// Path to an alternate config file (overrides default config locations).
    #[arg(long, value_name = "FILE")]
    pub config: Option<PathBuf>,

    /// Theme name override.
    #[arg(long, value_name = "NAME")]
    pub theme: Option<String>,
}

impl Cli {
    #[must_use]
    pub fn parse_args() -> Self {
        Self::parse()
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use clap::Parser;

    use super::Cli;

    #[test]
    fn defaults_path_to_current_directory() {
        let cli = Cli::parse_from(["kiwi-gui"]);
        assert_eq!(cli.path, PathBuf::from("."));
        assert!(cli.config.is_none());
        assert!(cli.theme.is_none());
    }

    #[test]
    fn parses_positional_path() {
        let cli = Cli::parse_from(["kiwi-gui", "/tmp/my-repo"]);
        assert_eq!(cli.path, PathBuf::from("/tmp/my-repo"));
    }

    #[test]
    fn parses_config_and_theme_flags() {
        let cli = Cli::parse_from([
            "kiwi-gui",
            "--config",
            "/tmp/kiwi.toml",
            "--theme",
            "dracula",
            ".",
        ]);
        assert_eq!(cli.config, Some(PathBuf::from("/tmp/kiwi.toml")));
        assert_eq!(cli.theme.as_deref(), Some("dracula"));
        assert_eq!(cli.path, PathBuf::from("."));
    }
}
