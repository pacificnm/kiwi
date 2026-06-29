mod error;
mod loader;
mod types;
mod writer;

pub use error::ConfigError;
pub use loader::{load_config, load_config_with_home, ConfigLoadOptions};
pub use types::{
    default_plugin_bin_directory, default_plugins_directory, expand_tilde, AgentMode,
    AgentSettings, EditorSettings, GuiSettings, MouseMode, MouseSettings, PluginsSettings,
    ResolvedConfig, ShellSettings, ThemeSettings,
};
pub use writer::{
    persist_user_agent, persist_user_agent_mode, persist_user_theme, project_has_theme_override,
    user_config_path,
};
