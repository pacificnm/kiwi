mod error;
mod loader;
mod types;

pub use error::ConfigError;
pub use loader::load_config;
pub use types::{
    AgentSettings, MouseMode, MouseSettings, ResolvedConfig, ShellSettings, ThemeSettings,
};
