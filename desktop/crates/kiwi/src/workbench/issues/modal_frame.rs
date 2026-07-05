//! Shared modal window helpers for GitHub issue dialogs.

use egui::{Align2, Context, RichText, Ui, Window};

/// Default modal width for issue metadata dialogs.
pub const MODAL_WIDTH: f32 = 480.0;

/// Renders a centered modal window; returns false when the user closes it.
pub fn centered_window(
    ctx: &Context,
    title: &str,
    open: &mut bool,
    add_body: impl FnOnce(&mut Ui),
) {
    if !*open {
        return;
    }

    Window::new(title)
        .collapsible(false)
        .resizable(true)
        .default_width(MODAL_WIDTH)
        .min_width(MODAL_WIDTH)
        .anchor(Align2::CENTER_CENTER, [0.0, 0.0])
        .open(open)
        .show(ctx, |ui| {
            ui.set_min_width(MODAL_WIDTH - 24.0);
            add_body(ui);
        });
}

/// Shows an error line when present.
pub fn error_line(ui: &mut Ui, error: &Option<String>) {
    if let Some(error) = error.as_ref() {
        ui.add_space(6.0);
        ui.label(
            RichText::new(error)
                .color(ui.visuals().error_fg_color)
                .size(12.0),
        );
    }
}

/// Parses a GitHub label color field.
pub fn parse_label_color(color: &str) -> Result<String, String> {
    let normalized = color.trim().trim_start_matches('#');
    if normalized.len() != 6 || !normalized.chars().all(|ch| ch.is_ascii_hexdigit()) {
        return Err("Color must be a 6-digit hex value (e.g. d73a4a)".into());
    }
    Ok(normalized.to_ascii_lowercase())
}

/// Parses optional milestone due date (`YYYY-MM-DD`).
pub fn parse_due_on(input: &str) -> Result<Option<String>, String> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Ok(None);
    }
    if trimmed.len() == 10 && trimmed.chars().nth(4) == Some('-') {
        return Ok(Some(format!("{trimmed}T12:00:00Z")));
    }
    Err("Due date must be YYYY-MM-DD or empty".into())
}

/// Formats milestone due date for editing.
pub fn format_due_on(due_on: &Option<String>) -> String {
    due_on
        .as_deref()
        .and_then(|value| value.get(0..10))
        .unwrap_or("")
        .to_string()
}
