//! Output bottom panel.

use egui::{RichText, Ui};

/// Renders build/output stream placeholder.
pub fn show(ui: &mut Ui) {
    ui.label(
        RichText::new("Build output will appear here.")
            .monospace()
            .size(12.0),
    );
}
