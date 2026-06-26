mod error;
mod loader;
mod types;

pub use error::ConfigError;
pub use loader::load_config;
pub use types::{
    default_plugins_directory, AgentSettings, EditorSettings, MouseMode, MouseSettings,
    PluginsSettings, ResolvedConfig, ShellSettings, ThemeSettings,
};
