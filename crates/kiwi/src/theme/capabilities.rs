#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TerminalCapabilities {
    Ansi16,
    Colors256,
    TrueColor,
}

impl TerminalCapabilities {
    #[must_use]
    pub fn detect() -> Self {
        match crossterm::style::available_color_count() {
            u16::MAX => Self::TrueColor,
            count if count >= 256 => Self::Colors256,
            _ => Self::Ansi16,
        }
    }
}
