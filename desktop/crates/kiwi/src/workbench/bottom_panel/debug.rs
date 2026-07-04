//! Debug bottom panel.

use egui::{RichText, Ui};

/// Renders debugger console placeholder.
pub fn show(ui: &mut Ui) {
    ui.label(
        RichText::new("Debugger not attached.")
            .monospace()
            .size(12.0),
    );
}
