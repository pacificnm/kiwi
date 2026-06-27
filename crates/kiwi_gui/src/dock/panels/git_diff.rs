//! Unified diff dock panel (SPEC-012 / SPEC-022).

use egui::{Color32, FontId, RichText, Ui};
use kiwi_core::diff::{DiffLine, DiffLineKind, DiffSource};
use kiwi_core::events::AppCommand;
use kiwi_core::theme::SemanticRole;

use super::layout::render_virtual_rows;
use crate::dock::context::PanelContext;

const ROW_HEIGHT: f32 = 16.0;
const TOOLBAR_HEIGHT: f32 = 22.0;
const APPROX_CHAR_WIDTH: f32 = 7.0;

pub fn render(ui: &mut Ui, ctx: &mut PanelContext<'_>) {
    render_toolbar(ui, ctx);
    ui.separator();
    render_diff_content(ui, ctx);
    ui.add_space(4.0);
    render_footer(ui, ctx);
}

/// Keyboard shortcuts when the Diff dock tab is focused.
pub fn keyboard_action(
    ctx: &egui::Context,
    state: &kiwi_core::state::AppState,
) -> Option<AppCommand> {
    if ctx.wants_keyboard_input() || state.palette.open {
        return None;
    }

    if ctx.input(|input| input.modifiers.any()) {
        return None;
    }

    if ctx.input(|input| input.key_pressed(egui::Key::ArrowDown)) {
        return Some(AppCommand::DiffScroll(1));
    }
    if ctx.input(|input| input.key_pressed(egui::Key::ArrowUp)) {
        return Some(AppCommand::DiffScroll(-1));
    }
    if ctx.input(|input| input.key_pressed(egui::Key::PageDown)) {
        return Some(AppCommand::DiffPageScroll(1));
    }
    if ctx.input(|input| input.key_pressed(egui::Key::PageUp)) {
        return Some(AppCommand::DiffPageScroll(-1));
    }
    if ctx.input(|input| input.key_pressed(egui::Key::S)) {
        return Some(AppCommand::DiffToggleSource);
    }
    if ctx.input(|input| input.key_pressed(egui::Key::N)) {
        return Some(AppCommand::DiffNextFile);
    }
    if ctx.input(|input| input.key_pressed(egui::Key::P)) {
        return Some(AppCommand::DiffPrevFile);
    }

    None
}

fn render_toolbar(ui: &mut Ui, ctx: &mut PanelContext<'_>) {
    ui.horizontal(|ui| {
        ui.set_min_height(TOOLBAR_HEIGHT);
        let source_label = match ctx.state.diff.source {
            DiffSource::Unstaged => "Unstaged",
            DiffSource::Staged => "Staged",
        };
        if ui.button(format!("View: {source_label} (S)")).clicked() {
            let _ = (ctx.dispatch)(AppCommand::DiffToggleSource);
        }
        if ui.button("Previous (P)").clicked() {
            let _ = (ctx.dispatch)(AppCommand::DiffPrevFile);
        }
        if ui.button("Next (N)").clicked() {
            let _ = (ctx.dispatch)(AppCommand::DiffNextFile);
        }
    });
}

fn render_footer(ui: &mut Ui, ctx: &PanelContext<'_>) {
    ui.label(
        RichText::new(format_diff_status(&ctx.state.diff))
            .color(ctx.theme.role(SemanticRole::Muted)),
    );
}

fn render_diff_content(ui: &mut Ui, ctx: &mut PanelContext<'_>) {
    let text_cols = text_cols_for(ui);
    ctx.state.viewport.preview_cols = text_cols;

    if let Some((text, role)) = diff_message(ctx) {
        ui.label(RichText::new(text).color(ctx.theme.role(role)));
        ctx.state.viewport.preview_rows = 1;
        return;
    }

    let lines = ctx.state.diff.lines.clone();
    if lines.is_empty() {
        ctx.state.viewport.preview_rows = 1;
        return;
    }

    let gutter = gutter_width(&lines);
    let text_width = text_cols.saturating_sub(gutter).max(1);
    let word_wrap = ctx.state.config.diff.word_wrap;
    let horizontal_offset = if word_wrap {
        0
    } else {
        ctx.state.diff.horizontal_scroll_offset
    };
    let mono = FontId::monospace(ui.style().text_styles[&egui::TextStyle::Monospace].size);

    let mut scroll_offset = ctx.state.diff.scroll_offset;
    let viewport_rows = render_virtual_rows(
        ui,
        ROW_HEIGHT,
        lines.len(),
        &mut scroll_offset,
        |ui, line_index| {
            let line = &lines[line_index];
            ui.horizontal(|ui| {
                ui.set_min_height(ROW_HEIGHT);
                if gutter > 0 {
                    ui.label(
                        RichText::new(format_gutter(line.old_lineno, line.new_lineno, &lines))
                            .font(mono.clone())
                            .color(ctx.theme.role(SemanticRole::Muted)),
                    );
                }
                let display = format_line_content(line, text_width, horizontal_offset, word_wrap);
                ui.label(
                    RichText::new(display)
                        .font(mono.clone())
                        .color(color_for_line_kind(ctx, line.kind)),
                );
            });
        },
    );
    ctx.state.diff.scroll_offset = scroll_offset;
    ctx.state.viewport.preview_rows = viewport_rows;
    ctx.state.diff.clamp_scroll_to_viewport(viewport_rows);
}

fn diff_message(ctx: &PanelContext<'_>) -> Option<(String, SemanticRole)> {
    if ctx.state.diff.selected_path.is_none() {
        return Some((
            "Select a changed file from the Git Status panel".to_string(),
            SemanticRole::Muted,
        ));
    }
    if ctx.state.diff.loading && ctx.state.diff.lines.is_empty() {
        return Some(("Loading…".to_string(), SemanticRole::Muted));
    }
    if let Some(error) = &ctx.state.diff.error {
        return Some((error.clone(), SemanticRole::AgentError));
    }
    if ctx.state.diff.is_binary {
        return Some(("Binary diff not supported".to_string(), SemanticRole::Muted));
    }
    if ctx.state.diff.lines.is_empty() {
        return Some((
            empty_diff_message(ctx.state.diff.source).to_string(),
            SemanticRole::Muted,
        ));
    }
    None
}

fn empty_diff_message(source: DiffSource) -> &'static str {
    match source {
        DiffSource::Staged => {
            "No staged changes for this file (S toggles view; use shell to git add)"
        }
        DiffSource::Unstaged => "No unstaged changes for this file (S toggles view)",
    }
}

pub(crate) fn format_diff_status(diff: &kiwi_core::state::DiffState) -> String {
    if diff.loading && diff.lines.is_empty() {
        return "Loading…".to_string();
    }

    let path = diff.selected_path.as_deref().unwrap_or("No file selected");
    let source = match diff.source {
        DiffSource::Unstaged => "view: unstaged",
        DiffSource::Staged => "view: staged",
    };

    if diff.is_binary {
        return format!("{path} | {source} | binary");
    }

    if diff.error.is_some() {
        return path.to_string();
    }

    if diff.loading {
        return format!("{path} | {source} | reloading…");
    }

    format!("{path} | {source} | {} lines", diff.lines.len())
}

fn color_for_line_kind(ctx: &PanelContext<'_>, kind: DiffLineKind) -> Color32 {
    match kind {
        DiffLineKind::Addition => ctx.theme.role(SemanticRole::GitAdded),
        DiffLineKind::Deletion => ctx.theme.role(SemanticRole::GitDeleted),
        DiffLineKind::Context | DiffLineKind::Header => ctx.theme.role(SemanticRole::Muted),
    }
}

fn gutter_width(lines: &[DiffLine]) -> usize {
    let old_width = max_lineno_digits(lines.iter().filter_map(|line| line.old_lineno));
    let new_width = max_lineno_digits(lines.iter().filter_map(|line| line.new_lineno));
    if old_width == 0 && new_width == 0 {
        return 0;
    }
    old_width + 1 + new_width + 2
}

fn max_lineno_digits(values: impl Iterator<Item = u32>) -> usize {
    values
        .map(|value| value.ilog10() as usize + 1)
        .max()
        .unwrap_or(0)
}

fn format_gutter(old_lineno: Option<u32>, new_lineno: Option<u32>, lines: &[DiffLine]) -> String {
    let old_width = max_lineno_digits(lines.iter().filter_map(|line| line.old_lineno));
    let new_width = max_lineno_digits(lines.iter().filter_map(|line| line.new_lineno));
    let old = old_lineno
        .map(|value| format!("{value:>old_width$}"))
        .unwrap_or_else(|| " ".repeat(old_width));
    let new = new_lineno
        .map(|value| format!("{value:>new_width$}"))
        .unwrap_or_else(|| " ".repeat(new_width));
    format!("{old} {new} ")
}

fn format_line_content(
    line: &DiffLine,
    text_width: usize,
    horizontal_offset: usize,
    word_wrap: bool,
) -> String {
    if word_wrap {
        return truncate_line(&line.content, text_width);
    }

    let sliced = slice_line(&line.content, horizontal_offset, text_width);
    truncate_line(&sliced, text_width)
}

fn slice_line(text: &str, offset: usize, width: usize) -> String {
    if offset == 0 {
        return text.to_string();
    }

    let chars: Vec<char> = text.chars().skip(offset).collect();
    if chars.len() <= width {
        return chars.into_iter().collect();
    }

    chars[..width].iter().collect()
}

fn truncate_line(text: &str, max_width: usize) -> String {
    if max_width == 0 {
        return String::new();
    }

    let chars: Vec<char> = text.chars().collect();
    if chars.len() <= max_width {
        return text.to_string();
    }

    if max_width <= 1 {
        return "…".to_string();
    }

    chars[..max_width - 1].iter().collect::<String>() + "…"
}

fn text_cols_for(ui: &Ui) -> usize {
    let width = ui.available_width().max(APPROX_CHAR_WIDTH);
    usize::try_from((width / APPROX_CHAR_WIDTH).floor() as i64)
        .unwrap_or(80)
        .max(1)
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use kiwi_core::config::ResolvedConfig;
    use kiwi_core::state::{AppState, DiffState, ViewportMetrics};
    use kiwi_core::theme::{load_theme_with_capabilities, SemanticRole, TerminalCapabilities};

    use super::*;
    use crate::theme::GuiTheme;

    fn test_panel() -> (AppState, GuiTheme) {
        let config = ResolvedConfig::default();
        let theme_palette =
            load_theme_with_capabilities(&config.theme, TerminalCapabilities::TrueColor)
                .expect("theme");
        let state = AppState::from_startup(
            PathBuf::from("/tmp/kiwi"),
            false,
            config.clone(),
            theme_palette,
            TerminalCapabilities::TrueColor,
            ViewportMetrics::default(),
        );
        let gui_theme = GuiTheme::from_palette(&state.theme, &config.gui);
        (state, gui_theme)
    }

    #[test]
    fn color_for_addition_uses_git_added_role() {
        let (state, theme) = test_panel();
        let mut noop = |_cmd: kiwi_core::events::AppCommand| false;
        let mut state = state;
        let ctx = PanelContext {
            state: &mut state,
            theme: &theme,
            dispatch: &mut noop,
        };
        assert_eq!(
            color_for_line_kind(&ctx, DiffLineKind::Addition),
            theme.role(SemanticRole::GitAdded)
        );
        assert_eq!(
            color_for_line_kind(&ctx, DiffLineKind::Deletion),
            theme.role(SemanticRole::GitDeleted)
        );
    }

    #[test]
    fn empty_staged_diff_shows_hint() {
        assert!(empty_diff_message(DiffSource::Staged).contains("No staged changes"));
    }

    #[test]
    fn format_diff_status_includes_line_count() {
        let diff = DiffState {
            selected_path: Some("src/main.rs".to_string()),
            source: DiffSource::Unstaged,
            lines: vec![DiffLine {
                kind: DiffLineKind::Context,
                content: " line".to_string(),
                old_lineno: Some(1),
                new_lineno: Some(1),
            }],
            ..DiffState::default()
        };
        let status = format_diff_status(&diff);
        assert!(status.contains("src/main.rs"));
        assert!(status.contains("1 lines"));
    }

    #[test]
    fn gutter_width_zero_when_no_line_numbers() {
        assert_eq!(
            gutter_width(&[DiffLine {
                kind: DiffLineKind::Header,
                content: "@@".to_string(),
                old_lineno: None,
                new_lineno: None,
            }]),
            0
        );
    }
}
