//! TUI theme adapter over [`kiwi_core::theme`].

pub mod capabilities;
pub mod loader;

use std::collections::HashMap;

use ratatui::style::{Color, Style};

pub use kiwi_core::theme::{
    load_theme_with_capabilities as load_core_theme_with_capabilities, SemanticRole, ThemeError,
    BUILTIN_THEME_NAMES,
};
pub use kiwi_core::theme::{RoleStyle as CoreRoleStyle, ThemePalette as CoreThemePalette};

use crate::config::ThemeSettings;

use capabilities::detect_terminal_capabilities;
pub use capabilities::TerminalCapabilities;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ThemePalette {
    pub name: String,
    roles: HashMap<SemanticRole, Style>,
}

impl ThemePalette {
    #[must_use]
    pub fn get(&self, role: SemanticRole) -> Style {
        self.roles.get(&role).copied().unwrap_or_default()
    }

    fn from_core(core: CoreThemePalette) -> Self {
        let roles = SemanticRole::ALL
            .into_iter()
            .map(|role| (role, core_role_to_style(core.get(role))))
            .collect();
        Self {
            name: core.name,
            roles,
        }
    }
}

pub fn load_theme(settings: &ThemeSettings) -> Result<ThemePalette, ThemeError> {
    load_theme_with_capabilities(settings, detect_terminal_capabilities())
}

pub fn load_theme_with_capabilities(
    settings: &ThemeSettings,
    capabilities: TerminalCapabilities,
) -> Result<ThemePalette, ThemeError> {
    let core = load_core_theme_with_capabilities(settings, capabilities)?;
    Ok(ThemePalette::from_core(core))
}

fn core_role_to_style(style: CoreRoleStyle) -> Style {
    let mut ratatui_style = Style::default();
    if let Some(fg) = style.fg {
        ratatui_style = ratatui_style.fg(resolved_color_to_ratatui(fg));
    }
    if let Some(bg) = style.bg {
        ratatui_style = ratatui_style.bg(resolved_color_to_ratatui(bg));
    }
    ratatui_style
}

fn resolved_color_to_ratatui(color: kiwi_core::theme::ResolvedColor) -> Color {
    match color {
        kiwi_core::theme::ResolvedColor::Rgb(r, g, b) => Color::Rgb(r, g, b),
        kiwi_core::theme::ResolvedColor::Indexed(index) => Color::Indexed(index),
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use ratatui::style::Color;

    use crate::config::ThemeSettings;

    use super::*;

    #[test]
    fn tui_palette_matches_core_kiwi_dark_colors() {
        let settings = ThemeSettings {
            name: "kiwi-dark".to_string(),
            custom: None,
        };
        let palette =
            load_theme_with_capabilities(&settings, TerminalCapabilities::TrueColor).expect("load");

        assert_eq!(palette.name, "kiwi-dark");
        assert_eq!(
            palette.get(SemanticRole::GitModified).fg,
            Some(Color::Rgb(224, 175, 104))
        );
    }

    #[test]
    fn tui_palette_loads_custom_theme() {
        let dir = std::env::temp_dir().join("kiwi-tui-theme-custom-test");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).expect("create temp dir");
        let path = dir.join("custom.toml");
        fs::write(
            &path,
            "name = \"custom\"\nextends = \"kiwi-dark\"\n\n[colors]\naccent = \"#ff00ff\"\n",
        )
        .expect("write custom theme");

        let settings = ThemeSettings {
            name: "kiwi-dark".to_string(),
            custom: Some(path),
        };
        let palette =
            load_theme_with_capabilities(&settings, TerminalCapabilities::TrueColor).expect("load");

        assert_eq!(
            palette.get(SemanticRole::Accent).fg,
            Some(Color::Rgb(255, 0, 255))
        );

        let _ = fs::remove_dir_all(dir);
    }
}
