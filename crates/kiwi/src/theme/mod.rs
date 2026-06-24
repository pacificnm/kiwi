mod color;
mod error;
pub mod loader;
mod roles;

pub mod capabilities;

pub use error::ThemeError;
pub use loader::{load_theme, ThemePalette};
