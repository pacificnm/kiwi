use std::path::PathBuf;

use clap::{Parser, Subcommand};

/// Terminal-native AI development workspace.
#[derive(Debug, Clone, PartialEq, Eq, Parser)]
#[command(name = "kiwi", version, about)]
pub struct Cli {
    /// Repository or workspace root directory (ignored when a subcommand is given).
    #[arg(value_name = "PATH", default_value = ".")]
    pub path: PathBuf,

    /// Path to an alternate config file (overrides default config locations).
    #[arg(long, value_name = "FILE")]
    pub config: Option<PathBuf>,

    /// Theme name override.
    #[arg(long, value_name = "NAME")]
    pub theme: Option<String>,

    /// Left navigation pane width in percent (10–50).
    #[arg(long, value_name = "PERCENT")]
    pub left_width: Option<u8>,

    #[command(subcommand)]
    pub command: Option<CliCommand>,
}

/// Top-level kiwi subcommands (bypass the TUI).
#[derive(Debug, Clone, PartialEq, Eq, Subcommand)]
pub enum CliCommand {
    /// Manage installed plugins.
    #[command(subcommand)]
    Plugin(PluginSubcommand),
}

/// `kiwi plugin <subcommand>`
#[derive(Debug, Clone, PartialEq, Eq, Subcommand)]
pub enum PluginSubcommand {
    /// List all plugins in the registry.
    List,
    /// Show details for a specific plugin.
    Info {
        /// Plugin name as it appears in the registry.
        name: String,
    },
    /// Enable a plugin (takes effect on next kiwi restart).
    Enable {
        /// Plugin name.
        name: String,
    },
    /// Disable a plugin (takes effect on next kiwi restart).
    Disable {
        /// Plugin name.
        name: String,
    },
    /// Install a plugin from a local directory.
    Install {
        /// Path to a directory containing `plugin.toml` and the shared library.
        path: PathBuf,
    },
    /// Remove an installed plugin (files and registry entry).
    Remove {
        /// Plugin name.
        name: String,
    },
    /// Remove and reinstall a plugin from a local directory.
    Reinstall {
        /// Path to a directory containing `plugin.toml`.
        path: PathBuf,
    },
    /// Reload and re-save the registry from disk (normalises the file).
    Reload,
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

    use super::{Cli, CliCommand, PluginSubcommand};

    #[test]
    fn defaults_path_to_current_directory() {
        let cli = Cli::parse_from(["kiwi"]);
        assert_eq!(cli.path, PathBuf::from("."));
        assert!(cli.config.is_none());
        assert!(cli.theme.is_none());
        assert!(cli.left_width.is_none());
        assert!(cli.command.is_none());
    }

    #[test]
    fn parses_positional_path() {
        let cli = Cli::parse_from(["kiwi", "/tmp/my-repo"]);
        assert_eq!(cli.path, PathBuf::from("/tmp/my-repo"));
        assert!(cli.command.is_none());
    }

    #[test]
    fn parses_config_theme_and_left_width_flags() {
        let cli = Cli::parse_from([
            "kiwi",
            "--config",
            "/tmp/kiwi.toml",
            "--theme",
            "dracula",
            "--left-width",
            "40",
            ".",
        ]);
        assert_eq!(cli.config, Some(PathBuf::from("/tmp/kiwi.toml")));
        assert_eq!(cli.theme.as_deref(), Some("dracula"));
        assert_eq!(cli.left_width, Some(40));
        assert_eq!(cli.path, PathBuf::from("."));
    }

    #[test]
    fn parses_plugin_list_subcommand() {
        let cli = Cli::parse_from(["kiwi", "plugin", "list"]);
        assert!(matches!(
            cli.command,
            Some(CliCommand::Plugin(PluginSubcommand::List))
        ));
    }

    #[test]
    fn parses_plugin_enable_with_name() {
        let cli = Cli::parse_from(["kiwi", "plugin", "enable", "hello"]);
        assert!(matches!(
            cli.command,
            Some(CliCommand::Plugin(PluginSubcommand::Enable { name })) if name == "hello"
        ));
    }

    #[test]
    fn parses_plugin_install_with_path() {
        let cli = Cli::parse_from(["kiwi", "plugin", "install", "/tmp/myplugin"]);
        assert!(matches!(
            cli.command,
            Some(CliCommand::Plugin(PluginSubcommand::Install { path })) if path == PathBuf::from("/tmp/myplugin")
        ));
    }
}
