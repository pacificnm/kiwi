//! Extensions sidebar.

use egui::{RichText, Ui};

/// Renders the extensions panel placeholder.
pub fn show(ui: &mut Ui) {
    ui.label(
        RichText::new("No extensions installed.")
            .weak()
            .size(12.0),
    );
}
