//! Cursor Dark theme tokens and Nest theme definition.

use egui::Color32;
use nest_design::theme::{ThemeDefinition, ThemeId, ThemeMode};
use nest_design::tokens::{
    ColorToken, ColorTokens, RadiusTokens, SpacingTokens, StatusTokens, TypographyStyle,
    TypographyTokens,
};

/// Cursor Dark semantic palette (IDE region fills and text roles).
#[derive(Debug, Clone, Copy)]
#[allow(dead_code)] // full token set; some roles are used via egui or workbench panels
pub struct CursorDarkPalette {
    pub background_default: Color32,
    pub background_elevated: Color32,
    pub background_panel: Color32,
    pub background_sidebar: Color32,
    pub background_editor: Color32,
    pub background_hover: Color32,
    pub background_active: Color32,
    pub border_default: Color32,
    pub border_subtle: Color32,
    pub text_primary: Color32,
    pub text_secondary: Color32,
    pub text_muted: Color32,
    pub text_disabled: Color32,
    pub accent_primary: Color32,
    pub accent_hover: Color32,
    pub accent_active: Color32,
    pub success: Color32,
    pub warning: Color32,
    pub error: Color32,
    pub info: Color32,
}

impl CursorDarkPalette {
    /// Cursor Dark theme palette.
    pub const fn new() -> Self {
        Self {
            background_default: hex("#1B1F23"),
            background_elevated: hex("#23272E"),
            background_panel: hex("#252B32"),
            background_sidebar: hex("#1E2228"),
            background_editor: hex("#1B1F23"),
            background_hover: hex("#2A3038"),
            background_active: hex("#313842"),
            border_default: hex("#313842"),
            border_subtle: hex("#2A3038"),
            text_primary: hex("#CCCCCC"),
            text_secondary: hex("#9DA5B4"),
            text_muted: hex("#6E7681"),
            text_disabled: hex("#4F5964"),
            accent_primary: hex("#4F8EF7"),
            accent_hover: hex("#6AA3FF"),
            accent_active: hex("#3B7CE6"),
            success: hex("#3FB950"),
            warning: hex("#D29922"),
            error: hex("#F85149"),
            info: hex("#58A6FF"),
        }
    }
}

/// Shared Cursor Dark palette instance.
pub const PALETTE: CursorDarkPalette = CursorDarkPalette::new();

/// Returns the Nest [`ThemeDefinition`] for Cursor Dark.
pub fn definition() -> ThemeDefinition {
    ThemeDefinition {
        id: ThemeId::new("cursor-dark"),
        mode: ThemeMode::Dark,
        colors: ColorTokens {
            background: color("#1B1F23"),
            foreground: color("#CCCCCC"),
            primary: color("#4F8EF7"),
            secondary: color("#9DA5B4"),
            border: color("#313842"),
            surface: color("#252B32"),
            accent: Some(color("#4F8EF7")),
            muted: Some(color("#6E7681")),
        },
        spacing: SpacingTokens {
            xs: 4.0,
            sm: 8.0,
            md: 16.0,
            lg: 24.0,
            xl: 32.0,
            xxl: Some(48.0),
        },
        radius: RadiusTokens {
            sm: 4.0,
            md: 6.0,
            lg: 8.0,
            full: Some(9999.0),
        },
        typography: TypographyTokens {
            body: TypographyStyle {
                font_family: "Inter".to_string(),
                size: 13.0,
                line_height: 18.0,
                weight: 400,
            },
            heading: TypographyStyle {
                font_family: "Inter".to_string(),
                size: 18.0,
                line_height: 24.0,
                weight: 500,
            },
            caption: Some(TypographyStyle {
                font_family: "Inter".to_string(),
                size: 11.0,
                line_height: 14.0,
                weight: 400,
            }),
            mono: Some(TypographyStyle {
                font_family: "JetBrains Mono".to_string(),
                size: 13.0,
                line_height: 18.0,
                weight: 400,
            }),
        },
        status: StatusTokens {
            success: color("#3FB950"),
            warning: color("#D29922"),
            error: color("#F85149"),
            info: color("#58A6FF"),
        },
    }
}

const fn hex(value: &str) -> Color32 {
    let bytes = value.as_bytes();
    let r = hex_byte(bytes[1], bytes[2]);
    let g = hex_byte(bytes[3], bytes[4]);
    let b = hex_byte(bytes[5], bytes[6]);
    Color32::from_rgb(r, g, b)
}

const fn hex_byte(high: u8, low: u8) -> u8 {
    (hex_nibble(high) << 4) | hex_nibble(low)
}

const fn hex_nibble(byte: u8) -> u8 {
    match byte {
        b'0'..=b'9' => byte - b'0',
        b'a'..=b'f' => byte - b'a' + 10,
        b'A'..=b'F' => byte - b'A' + 10,
        _ => 0,
    }
}

fn color(value: &str) -> ColorToken {
    ColorToken::new(value).expect("cursor-dark theme colors are valid")
}
