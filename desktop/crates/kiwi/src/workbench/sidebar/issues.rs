//! Issues sidebar.

use egui::{RichText, Ui};

/// Renders the issues panel placeholder.
pub fn show(ui: &mut Ui) {
    ui.label(RichText::new("No issues.").weak().size(12.0));
}
