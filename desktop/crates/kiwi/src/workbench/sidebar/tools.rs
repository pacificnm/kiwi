//! Tools sidebar.

use egui::{RichText, Ui};

/// Renders the tooling panel placeholder.
pub fn show(ui: &mut Ui) {
    ui.label(RichText::new("Tooling panel.").weak().size(12.0));
}
