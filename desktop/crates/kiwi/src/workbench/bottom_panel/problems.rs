//! Problems bottom panel.

use egui::{RichText, Ui};

/// Renders compiler/linter problems placeholder.
pub fn show(ui: &mut Ui) {
    ui.label(
        RichText::new("No problems detected.")
            .monospace()
            .size(12.0),
    );
}
