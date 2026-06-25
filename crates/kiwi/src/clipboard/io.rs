/// Long-lived clipboard handle for the application session.
///
/// On Linux (X11/Wayland) the clipboard owner must stay alive to serve paste
/// requests. Creating and dropping a new `arboard::Clipboard` on every copy also
/// prints a stderr warning that corrupts the alternate-screen TUI.
pub struct ClipboardService {
    backend: Option<arboard::Clipboard>,
}

impl ClipboardService {
    #[must_use]
    pub fn new() -> Self {
        Self {
            backend: arboard::Clipboard::new().ok(),
        }
    }

    #[cfg_attr(not(test), allow(dead_code))]
    #[must_use]
    pub fn is_available(&self) -> bool {
        self.backend.is_some()
    }

    pub fn read_text(&mut self) -> Result<String, String> {
        let backend = self
            .backend
            .as_mut()
            .ok_or_else(|| "Clipboard unavailable".to_string())?;
        backend.get_text().map_err(|err| err.to_string())
    }

    pub fn write_text(&mut self, text: &str) -> Result<(), String> {
        let backend = self
            .backend
            .as_mut()
            .ok_or_else(|| "Clipboard unavailable".to_string())?;
        backend
            .set_text(text.to_string())
            .map_err(|err| err.to_string())
    }
}

impl Default for ClipboardService {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip_clipboard_when_available() {
        let mut clipboard = ClipboardService::new();
        if !clipboard.is_available() {
            return;
        }
        if clipboard.write_text("kiwi clipboard test").is_err() {
            return;
        }
        let Ok(text) = clipboard.read_text() else {
            return;
        };
        assert_eq!(text, "kiwi clipboard test");
    }
}
