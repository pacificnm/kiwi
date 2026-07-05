//! Syntax highlighting for the editor via `egui_extras` + `syntect`.

use std::path::Path;

use egui::{Align, Color32, Frame, Label, Layout, RichText, TextEdit, TextStyle, TextWrapMode, Ui};
use egui_extras::syntax_highlighting::{highlight, CodeTheme};

use crate::theme::PALETTE;

/// Vertical padding inside multiline [`TextEdit`] (matches egui default margin).
const TEXT_EDIT_TOP_MARGIN: f32 = 2.0;

/// Horizontal padding inside the line-number gutter frame.
const GUTTER_FRAME_MARGIN: f32 = 8.0;

/// Syntect language lookup via file extension (see `SyntaxSet::find_syntax_by_extension`).
pub fn language_from_path(rel_path: &str) -> &str {
    Path::new(rel_path)
        .extension()
        .and_then(|value| value.to_str())
        .unwrap_or("txt")
}

/// Number of logical lines in a buffer (minimum 1).
pub fn line_count(content: &str) -> usize {
    content.chars().filter(|&ch| ch == '\n').count() + 1
}

/// Multiline code editor with a line-number gutter and syntax highlighting.
pub fn highlighted_code_editor_with_lines<'text>(
    ui: &mut Ui,
    content: &'text mut String,
    rel_path: &str,
) -> egui::Response {
    let lines = line_count(content);
    let mut editor_response = None;

    ui.horizontal_top(|ui| {
        ui.spacing_mut().item_spacing.x = 0.0;
        line_number_gutter(ui, lines);
        ui.add_space(1.0);
        editor_response = Some(highlighted_code_editor(ui, content, rel_path));
    });

    editor_response.unwrap_or_else(|| ui.response())
}

fn line_number_gutter(ui: &mut Ui, line_count: usize) {
    let width = gutter_width(ui, line_count);
    let color = ui.visuals().weak_text_color();
    let gutter_text = (1..=line_count)
        .map(|line| line.to_string())
        .collect::<Vec<_>>()
        .join("\n");

    ui.allocate_ui_with_layout(
        egui::vec2(width, 0.0),
        Layout::top_down(Align::RIGHT),
        |ui| {
            Frame::new()
                .fill(PALETTE.background_sidebar)
                .inner_margin(egui::Margin {
                    left: GUTTER_FRAME_MARGIN as i8,
                    right: GUTTER_FRAME_MARGIN as i8,
                    top: TEXT_EDIT_TOP_MARGIN as i8,
                    bottom: 0,
                })
                .show(ui, |ui| {
                    ui.add(
                        Label::new(
                            RichText::new(gutter_text)
                                .text_style(TextStyle::Monospace)
                                .color(color),
                        )
                        .wrap_mode(TextWrapMode::Extend),
                    );
                });
        },
    );
}

fn max_line_number_digits(line_count: usize) -> usize {
    line_count.max(1).to_string().len()
}

fn gutter_width(ui: &Ui, line_count: usize) -> f32 {
    let sample = "9".repeat(max_line_number_digits(line_count));
    let font_id = TextStyle::Monospace.resolve(ui.style());
    let text_width = ui.fonts(|fonts| {
        fonts
            .layout_no_wrap(sample, font_id, Color32::PLACEHOLDER)
            .size()
            .x
    });
    text_width + GUTTER_FRAME_MARGIN * 2.0 + 2.0
}

fn highlighted_code_editor<'text>(
    ui: &mut Ui,
    content: &'text mut String,
    rel_path: &str,
) -> egui::Response {
    let language = language_from_path(rel_path);
    let theme = CodeTheme::from_memory(ui.ctx(), ui.style());

    let mut layouter = |ui: &Ui, text: &str, wrap_width: f32| {
        let mut layout_job = highlight(ui.ctx(), ui.style(), &theme, text, language);
        layout_job.wrap.max_width = wrap_width;
        ui.fonts(|fonts| fonts.layout_job(layout_job))
    };

    ui.add(
        TextEdit::multiline(content)
            .code_editor()
            .font(TextStyle::Monospace)
            .desired_width(f32::INFINITY)
            .frame(false)
            .layouter(&mut layouter),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn language_from_rust_source() {
        assert_eq!(language_from_path("src/main.rs"), "rs");
    }

    #[test]
    fn language_from_nested_path() {
        assert_eq!(
            language_from_path("core/crates/nest-file/Cargo.toml"),
            "toml"
        );
    }

    #[test]
    fn language_defaults_without_extension() {
        assert_eq!(language_from_path("Makefile"), "txt");
    }

    #[test]
    fn line_count_empty_is_one() {
        assert_eq!(line_count(""), 1);
    }

    #[test]
    fn line_count_tracks_newlines() {
        assert_eq!(line_count("a"), 1);
        assert_eq!(line_count("a\nb"), 2);
        assert_eq!(line_count("a\nb\n"), 3);
    }

    #[test]
    fn max_line_number_digits_at_boundaries() {
        assert_eq!(max_line_number_digits(99), 2);
        assert_eq!(max_line_number_digits(100), 3);
        assert_eq!(max_line_number_digits(999), 3);
        assert_eq!(max_line_number_digits(1000), 4);
    }
}
