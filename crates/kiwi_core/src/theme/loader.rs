use std::collections::HashMap;
use std::path::{Path, PathBuf};

use serde::Deserialize;

use crate::config::ThemeSettings;

use super::capabilities::TerminalCapabilities;
use super::color::ColorValue;
use super::error::ThemeError;
use super::palette::{RoleStyle, ThemePalette};
use super::roles::SemanticRole;

const KIWI_DARK_TOML: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../assets/themes/kiwi-dark.toml"
));
const KIWI_LIGHT_TOML: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../assets/themes/kiwi-light.toml"
));
const DRACULA_TOML: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../assets/themes/dracula.toml"
));
const CATPPUCCIN_MOCHA_TOML: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../assets/themes/catppuccin-mocha.toml"
));
const CATPPUCCIN_LATTE_TOML: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../assets/themes/catppuccin-latte.toml"
));
const GRUVBOX_TOML: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../assets/themes/gruvbox.toml"
));
const NORD_TOML: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../assets/themes/nord.toml"
));
const TOKYO_NIGHT_TOML: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../assets/themes/tokyo-night.toml"
));

pub const BUILTIN_THEME_NAMES: &[&str] = &[
    "kiwi-dark",
    "kiwi-light",
    "dracula",
    "catppuccin-mocha",
    "catppuccin-latte",
    "gruvbox",
    "nord",
    "tokyo-night",
];

#[derive(Debug, Clone, Deserialize)]
pub struct ThemeDefinition {
    pub name: String,
    #[serde(default)]
    pub extends: Option<String>,
    #[serde(default)]
    pub colors: HashMap<String, String>,
}

pub fn load_theme(settings: &ThemeSettings) -> Result<ThemePalette, ThemeError> {
    load_theme_with_capabilities(settings, TerminalCapabilities::detect_from_env())
}

pub fn load_theme_with_capabilities(
    settings: &ThemeSettings,
    capabilities: TerminalCapabilities,
) -> Result<ThemePalette, ThemeError> {
    if let Some(path) = &settings.custom {
        let definition = parse_theme_file(
            path,
            std::fs::read_to_string(path).map_err(|err| ThemeError::Parse {
                path: Some(path.to_path_buf()),
                message: err.to_string(),
            })?,
        )?;
        return resolve_definition(definition, capabilities, Some(path.to_path_buf()));
    }

    let toml = builtin_theme_toml(&settings.name)
        .ok_or_else(|| ThemeError::UnknownTheme(settings.name.clone()))?;
    let definition = parse_theme_toml(toml, None)?;
    resolve_definition(definition, capabilities, None)
}

fn builtin_theme_toml(name: &str) -> Option<&'static str> {
    match name {
        "kiwi-dark" => Some(KIWI_DARK_TOML),
        "kiwi-light" => Some(KIWI_LIGHT_TOML),
        "dracula" => Some(DRACULA_TOML),
        "catppuccin-mocha" => Some(CATPPUCCIN_MOCHA_TOML),
        "catppuccin-latte" => Some(CATPPUCCIN_LATTE_TOML),
        "gruvbox" => Some(GRUVBOX_TOML),
        "nord" => Some(NORD_TOML),
        "tokyo-night" => Some(TOKYO_NIGHT_TOML),
        _ => None,
    }
}

fn parse_theme_file(path: &Path, content: String) -> Result<ThemeDefinition, ThemeError> {
    parse_theme_toml(&content, Some(path.to_path_buf()))
}

fn parse_theme_toml(content: &str, path: Option<PathBuf>) -> Result<ThemeDefinition, ThemeError> {
    toml::from_str(content).map_err(|err| ThemeError::Parse {
        path,
        message: err.to_string(),
    })
}

fn resolve_definition(
    definition: ThemeDefinition,
    capabilities: TerminalCapabilities,
    path: Option<PathBuf>,
) -> Result<ThemePalette, ThemeError> {
    let mut colors = base_colors()?;

    if let Some(extends) = &definition.extends {
        let parent =
            builtin_theme_toml(extends).ok_or_else(|| ThemeError::UnknownTheme(extends.clone()))?;
        let parent_definition = parse_theme_toml(parent, path.clone())?;
        colors.extend(parent_definition.colors);
    }

    colors.extend(definition.colors);

    let mut roles = HashMap::new();
    for role in SemanticRole::ALL {
        let value = colors
            .get(role.as_str())
            .ok_or(ThemeError::MissingRole(role))?;
        let color = ColorValue::parse(value, role)?;
        roles.insert(role, role_to_style(role, color.resolve(capabilities)));
    }

    Ok(ThemePalette::new(definition.name, roles))
}

fn base_colors() -> Result<HashMap<String, String>, ThemeError> {
    let base = parse_theme_toml(KIWI_DARK_TOML, None)?;
    Ok(base.colors)
}

fn role_to_style(role: SemanticRole, color: super::color::ResolvedColor) -> RoleStyle {
    match role {
        SemanticRole::Bg | SemanticRole::Selection => RoleStyle {
            fg: None,
            bg: Some(color),
        },
        _ => RoleStyle {
            fg: Some(color),
            bg: None,
        },
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use crate::config::ThemeSettings;

    use super::super::capabilities::TerminalCapabilities;
    use super::super::color::ResolvedColor;
    use super::super::roles::SemanticRole;
    use super::{
        builtin_theme_toml, load_theme_with_capabilities, parse_theme_toml, BUILTIN_THEME_NAMES,
    };

    #[test]
    fn default_kiwi_dark_loads() {
        let settings = ThemeSettings {
            name: "kiwi-dark".to_string(),
            custom: None,
        };
        let palette =
            load_theme_with_capabilities(&settings, TerminalCapabilities::TrueColor).expect("load");

        assert_eq!(palette.name, "kiwi-dark");
        assert_eq!(
            palette.git_modified_style().fg,
            Some(ResolvedColor::Rgb(224, 175, 104))
        );
    }

    #[test]
    fn kiwi_light_loads() {
        let settings = ThemeSettings {
            name: "kiwi-light".to_string(),
            custom: None,
        };
        let palette =
            load_theme_with_capabilities(&settings, TerminalCapabilities::TrueColor).expect("load");

        assert_eq!(palette.name, "kiwi-light");
        assert_eq!(
            palette.get(SemanticRole::Bg).bg,
            Some(ResolvedColor::Rgb(232, 234, 244))
        );
        assert_eq!(
            palette.git_modified_style().fg,
            Some(ResolvedColor::Rgb(184, 132, 31))
        );
    }

    #[test]
    fn builtin_theme_manifest_matches_spec() {
        assert_eq!(BUILTIN_THEME_NAMES.len(), 8);
        for name in BUILTIN_THEME_NAMES {
            assert!(
                builtin_theme_toml(name).is_some(),
                "missing embedded theme for {name}"
            );
        }
    }

    #[test]
    fn each_builtin_theme_loads() {
        for &name in BUILTIN_THEME_NAMES {
            let settings = ThemeSettings {
                name: name.to_string(),
                custom: None,
            };
            load_theme_with_capabilities(&settings, TerminalCapabilities::TrueColor)
                .unwrap_or_else(|err| panic!("failed to load {name}: {err}"));
        }
    }

    #[test]
    fn each_builtin_theme_defines_all_roles() {
        for &name in BUILTIN_THEME_NAMES {
            let toml = builtin_theme_toml(name).expect(name);
            let definition = parse_theme_toml(toml, None).expect(name);
            for role in SemanticRole::ALL {
                assert!(
                    definition.colors.contains_key(role.as_str()),
                    "{name} missing role {}",
                    role.as_str()
                );
            }
        }
    }

    #[test]
    fn custom_theme_can_override_single_role() {
        let dir = std::env::temp_dir().join("kiwi-theme-custom-test");
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
            Some(ResolvedColor::Rgb(255, 0, 255))
        );
        assert_eq!(
            palette.git_modified_style().fg,
            Some(ResolvedColor::Rgb(224, 175, 104))
        );

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn unknown_builtin_theme_errors() {
        let settings = ThemeSettings {
            name: "solarized".to_string(),
            custom: None,
        };
        let err = load_theme_with_capabilities(&settings, TerminalCapabilities::TrueColor)
            .expect_err("unknown theme");
        assert!(err.to_string().contains("unknown theme"));
    }

    #[test]
    fn dracula_loads_with_expected_accent() {
        let settings = ThemeSettings {
            name: "dracula".to_string(),
            custom: None,
        };
        let palette =
            load_theme_with_capabilities(&settings, TerminalCapabilities::TrueColor).expect("load");

        assert_eq!(palette.name, "dracula");
        assert_eq!(
            palette.get(SemanticRole::Accent).fg,
            Some(ResolvedColor::Rgb(189, 147, 249))
        );
    }

    #[test]
    fn custom_theme_can_extend_kiwi_light() {
        let dir = std::env::temp_dir().join("kiwi-theme-light-extends-test");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).expect("create temp dir");
        let path = dir.join("custom.toml");
        fs::write(
            &path,
            "name = \"custom-light\"\nextends = \"kiwi-light\"\n\n[colors]\naccent = \"#ff00ff\"\n",
        )
        .expect("write custom theme");

        let settings = ThemeSettings {
            name: "kiwi-light".to_string(),
            custom: Some(path),
        };
        let palette =
            load_theme_with_capabilities(&settings, TerminalCapabilities::TrueColor).expect("load");

        assert_eq!(
            palette.get(SemanticRole::Accent).fg,
            Some(ResolvedColor::Rgb(255, 0, 255))
        );
        assert_eq!(
            palette.get(SemanticRole::Bg).bg,
            Some(ResolvedColor::Rgb(232, 234, 244))
        );

        let _ = fs::remove_dir_all(dir);
    }
}
