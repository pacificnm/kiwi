//! Git bottom panel.

use egui::{RichText, Ui};

/// Renders git operations log placeholder.
pub fn show(ui: &mut Ui) {
    ui.label(
        RichText::new("On branch main\nnothing to commit, working tree clean")
            .monospace()
            .size(12.0),
    );
}
