//! Search sidebar — project-wide search.

use egui::{RichText, TextEdit, Ui};

/// Renders the search sidebar.
pub fn show(ui: &mut Ui, query: &mut String) {
    use super::SIDEBAR_INNER_WIDTH;

    ui.label("Search");
    ui.add(
        TextEdit::singleline(query)
            .hint_text("Search files…")
            .desired_width(SIDEBAR_INNER_WIDTH),
    );
    ui.add_space(8.0);
    ui.label(
        RichText::new("Type to search the project.")
            .weak()
            .size(12.0),
    );
}
