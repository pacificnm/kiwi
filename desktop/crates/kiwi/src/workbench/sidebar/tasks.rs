//! Tasks sidebar.

use egui::{RichText, Ui};

/// Renders the tasks panel placeholder.
pub fn show(ui: &mut Ui) {
    ui.label(RichText::new("No tasks.").weak().size(12.0));
}
