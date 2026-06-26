pub use kiwi_core::theme::TerminalCapabilities;

pub fn detect_terminal_capabilities() -> TerminalCapabilities {
    match crossterm::style::available_color_count() {
        u16::MAX => TerminalCapabilities::TrueColor,
        count if count >= 256 => TerminalCapabilities::Colors256,
        _ => TerminalCapabilities::Ansi16,
    }
}
