use ratatui::style::Color;

use super::capabilities::TerminalCapabilities;
use super::error::ThemeError;
use super::roles::SemanticRole;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorValue {
    Rgb(u8, u8, u8),
    Ansi(u8),
}

impl ColorValue {
    pub fn parse(value: &str, role: SemanticRole) -> Result<Self, ThemeError> {
        let value = value.trim();
        if let Some(hex) = value.strip_prefix('#') {
            return parse_hex(hex, role, value);
        }

        if let Some(index) = value.strip_prefix("ansi_") {
            let index = index
                .parse::<u8>()
                .map_err(|_| ThemeError::invalid_color(role, value))?;
            return Ok(Self::Ansi(index));
        }

        Err(ThemeError::invalid_color(role, value))
    }

    #[must_use]
    pub fn to_ratatui_color(self, capabilities: TerminalCapabilities) -> Color {
        match self {
            Self::Rgb(r, g, b) => match capabilities {
                TerminalCapabilities::TrueColor => Color::Rgb(r, g, b),
                TerminalCapabilities::Colors256 | TerminalCapabilities::Ansi16 => {
                    Color::Indexed(rgb_to_256(r, g, b))
                }
            },
            Self::Ansi(index) => Color::Indexed(index),
        }
    }
}

fn parse_hex(hex: &str, role: SemanticRole, raw: &str) -> Result<ColorValue, ThemeError> {
    let bytes = match hex.len() {
        6 => hex.as_bytes(),
        3 => return Err(ThemeError::invalid_color(role, raw)),
        _ => return Err(ThemeError::invalid_color(role, raw)),
    };

    let r = u8::from_str_radix(std::str::from_utf8(&bytes[0..2]).expect("hex"), 16)
        .map_err(|_| ThemeError::invalid_color(role, raw))?;
    let g = u8::from_str_radix(std::str::from_utf8(&bytes[2..4]).expect("hex"), 16)
        .map_err(|_| ThemeError::invalid_color(role, raw))?;
    let b = u8::from_str_radix(std::str::from_utf8(&bytes[4..6]).expect("hex"), 16)
        .map_err(|_| ThemeError::invalid_color(role, raw))?;

    Ok(ColorValue::Rgb(r, g, b))
}

fn rgb_to_256(r: u8, g: u8, b: u8) -> u8 {
    if r == g && g == b {
        if r < 8 {
            return 16;
        }
        if r > 248 {
            return 231;
        }
        return 232 + (r - 8) / 10;
    }

    16 + 36 * (r / 51) + 6 * (g / 51) + (b / 51)
}

#[cfg(test)]
mod tests {
    use ratatui::style::Color;

    use super::ColorValue;
    use super::TerminalCapabilities;
    use crate::theme::roles::SemanticRole;

    #[test]
    fn parses_hex_for_truecolor() {
        let color = ColorValue::parse("#e0af68", SemanticRole::GitModified).expect("parse hex");
        assert_eq!(
            color.to_ratatui_color(TerminalCapabilities::TrueColor),
            Color::Rgb(224, 175, 104)
        );
    }

    #[test]
    fn parses_ansi_index() {
        let color = ColorValue::parse("ansi_11", SemanticRole::GitModified).expect("parse ansi");
        assert_eq!(
            color.to_ratatui_color(TerminalCapabilities::Ansi16),
            Color::Indexed(11)
        );
    }
}
