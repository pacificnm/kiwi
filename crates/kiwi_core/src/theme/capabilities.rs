#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TerminalCapabilities {
    Ansi16,
    Colors256,
    TrueColor,
}

impl TerminalCapabilities {
    /// Best-effort detection from environment variables (no terminal I/O).
    #[must_use]
    pub fn detect_from_env() -> Self {
        match std::env::var("COLORTERM").as_deref() {
            Ok("truecolor") | Ok("24bit") => Self::TrueColor,
            _ => {
                if std::env::var("TERM")
                    .map(|term| term.contains("256color"))
                    .unwrap_or(false)
                {
                    Self::Colors256
                } else {
                    Self::Ansi16
                }
            }
        }
    }
}
