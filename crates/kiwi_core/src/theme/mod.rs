mod capabilities;
mod color;
mod error;
mod loader;
mod palette;
mod roles;

pub use capabilities::TerminalCapabilities;
pub use color::ResolvedColor;
pub use error::ThemeError;
pub use loader::{load_theme, load_theme_with_capabilities, ThemeDefinition, BUILTIN_THEME_NAMES};
pub use palette::{RoleStyle, ThemePalette};
pub use roles::SemanticRole;
