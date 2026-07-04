//! Terminal bottom panel.

use egui::{RichText, Ui};

/// Renders the integrated terminal placeholder.
pub fn show(ui: &mut Ui) {
    ui.label(
        RichText::new(
            "$ cargo run\n   Compiling kiwi v0.1.0\n    Finished dev [unoptimized + debuginfo]",
        )
        .monospace()
        .size(12.0),
    );
}
