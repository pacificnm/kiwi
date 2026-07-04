//! Logs bottom panel.

use egui::{RichText, Ui};

/// Renders application logs placeholder.
pub fn show(ui: &mut Ui) {
    ui.label(
        RichText::new("[info] Kiwi workbench ready.")
            .monospace()
            .size(12.0),
    );
}
