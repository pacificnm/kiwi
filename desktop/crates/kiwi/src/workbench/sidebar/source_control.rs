//! Source control sidebar.

use egui::{RichText, Ui};

/// Renders the git / source control panel placeholder.
pub fn show(ui: &mut Ui) {
    placeholder(ui, "No repository changes.");
}

fn placeholder(ui: &mut Ui, message: &str) {
    ui.label(RichText::new(message).weak().size(12.0));
}
