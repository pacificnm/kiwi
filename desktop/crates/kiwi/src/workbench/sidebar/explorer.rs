//! Explorer sidebar — project file tree.

use egui::{RichText, Ui};

/// Renders the file explorer tree placeholder.
pub fn show(ui: &mut Ui) {
    ui.label(RichText::new("Project Tree").weak().size(11.0));
    ui.add_space(4.0);
    for line in [
        "▸ src",
        "  ▸ gui",
        "  ▸ core",
        "  Cargo.toml",
        "  README.md",
    ] {
        ui.label(RichText::new(line).size(13.0).monospace());
    }
}
