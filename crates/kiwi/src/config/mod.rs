mod error;
mod loader;
mod types;
mod writer;

pub use error::ConfigError;
pub use loader::load_config;
pub use types::{
    default_plugins_directory, AgentSettings, EditorSettings, MouseMode, MouseSettings,
    PluginsSettings, ResolvedConfig, ShellSettings, ThemeSettings,
};
pub use writer::{persist_user_theme, project_has_theme_override};
