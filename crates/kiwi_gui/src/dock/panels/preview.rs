//! Preview dock panel — read-only file content (#192 / SPEC-006).

use egui::{RichText, Ui};
use kiwi_core::editor::resolve_editor_target;
use kiwi_core::events::AppCommand;
use kiwi_core::navigation::{FocusTarget, MainTab, NavCommand};
use kiwi_core::state::ReduceView;
use kiwi_core::theme::SemanticRole;

use super::layout::{render_virtual_rows, truncate_line};
use crate::dock::context::PanelContext;
use crate::dock::tab::KiwiTab;

const ROW_HEIGHT: f32 = 18.0;

pub fn render(ui: &mut Ui, ctx: &mut PanelContext<'_>) {
    if ctx.is_dock_tab_focused(KiwiTab::Preview) {
        sync_preview_navigation(ctx);
    }

    ui.horizontal(|ui| {
        ui.label(
            RichText::new(panel_title(ctx))
                .color(ctx.theme.role(SemanticRole::Muted))
                .strong(),
        );
        if ui.button("Open in editor").clicked() {
            dispatch_open_editor(ctx);
        }
    });
    ui.separator();

    if let Some(message) = blocking_message(ctx) {
        ui.label(
            RichText::new(message.clone()).color(message_color(ctx, &message)),
        );
        ctx.state.viewport.preview_rows = 1;
        render_status_footer(ui, ctx);
        return;
    }

    render_content(ui, ctx);
    render_status_footer(ui, ctx);
}

/// Keyboard shortcuts when the Preview dock tab is focused.
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
        return Some(AppCommand::PreviewScroll(1));
    }
    if ctx.input(|input| input.key_pressed(egui::Key::ArrowUp)) {
        return Some(AppCommand::PreviewScroll(-1));
    }
    if ctx.input(|input| input.key_pressed(egui::Key::PageDown)) {
        return Some(AppCommand::PreviewPageScroll(1));
    }
    if ctx.input(|input| input.key_pressed(egui::Key::PageUp)) {
        return Some(AppCommand::PreviewPageScroll(-1));
    }
    if ctx.input(|input| input.key_pressed(egui::Key::E)) {
        return open_editor_command(state);
    }

    None
}

fn open_editor_command(state: &kiwi_core::state::AppState) -> Option<AppCommand> {
    let path = state.preview.path.clone()?;
    let line = u32::try_from(state.preview.scroll_offset.saturating_add(1)).ok();
    Some(AppCommand::OpenEditor { path, line })
}

fn sync_preview_navigation(ctx: &mut PanelContext<'_>) {
    if ctx.state.navigation.main_tab != MainTab::Preview {
        let _ = (ctx.dispatch)(AppCommand::Navigation(NavCommand::SelectMainTabUnpaired(
            MainTab::Preview,
        )));
    }
    if ctx.state.navigation.focus != FocusTarget::Main {
        let _ = (ctx.dispatch)(AppCommand::Navigation(NavCommand::SetFocus(
            FocusTarget::Main,
        )));
    }
}

fn dispatch_open_editor(ctx: &mut PanelContext<'_>) {
    let view = ReduceView::from_app_state(ctx.state);
    if let Some(target) = resolve_editor_target(&view) {
        let _ = (ctx.dispatch)(AppCommand::OpenEditor {
            path: target.path,
            line: target.line,
        });
    }
}

fn panel_title(ctx: &PanelContext<'_>) -> String {
    ctx.state
        .preview
        .path
        .as_ref()
        .and_then(|path| path.file_name())
        .map(|name| format!("Preview: {}", name.to_string_lossy()))
        .unwrap_or_else(|| "Preview".to_string())
}

fn blocking_message(ctx: &PanelContext<'_>) -> Option<String> {
    let preview = &ctx.state.preview;
    if preview.loading && preview.lines.is_empty() {
        return Some("Loading…".to_string());
    }
    if let Some(error) = preview.load_error.as_deref() {
        return Some(error.to_string());
    }
    if preview.binary {
        return Some(format!("Binary file ({} bytes)", preview.file_size));
    }
    if preview.oversize {
        return Some(format!(
            "File too large to preview ({} bytes)",
            preview.file_size
        ));
    }
    if preview.lines.is_empty() {
        return Some(if preview.path.is_some() {
            "Empty file".to_string()
        } else {
            "Select a file to preview".to_string()
        });
    }
    None
}

fn message_color(ctx: &PanelContext<'_>, message: &str) -> egui::Color32 {
    if ctx.state.preview.load_error.is_some() {
        ctx.theme.role(SemanticRole::AgentError)
    } else {
        let _ = message;
        ctx.theme.role(SemanticRole::Muted)
    }
}

fn render_content(ui: &mut Ui, ctx: &mut PanelContext<'_>) {
    let line_numbers = ctx.state.config.preview.line_numbers;
    let line_count = ctx.state.preview.line_count();
    let gutter_width = if line_numbers {
        line_number_width(line_count).max(4)
    } else {
        0
    };

    let cols = max_text_cols(ui, gutter_width);
    ctx.state.viewport.preview_cols = usize::from(cols);

    let lines = &ctx.state.preview.lines;
    let mut scroll_offset = ctx.state.preview.scroll_offset;
    let viewport_rows = render_virtual_rows(
        ui,
        ROW_HEIGHT,
        lines.len(),
        &mut scroll_offset,
        |ui, row_index| {
            let line_text = lines.get(row_index).map(String::as_str).unwrap_or("");
            render_line_row(ui, ctx, row_index, line_text, gutter_width, cols);
        },
    );
    ctx.state.preview.scroll_offset = scroll_offset;
    ctx.state.viewport.preview_rows = viewport_rows.max(1);
}

fn render_line_row(
    ui: &mut Ui,
    ctx: &PanelContext<'_>,
    line_index: usize,
    line_text: &str,
    gutter_width: usize,
    text_cols: u16,
) {
    ui.horizontal(|ui| {
        ui.set_min_height(ROW_HEIGHT);
        if gutter_width > 0 {
            let number = format!("{:>width$} ", line_index + 1, width = gutter_width - 1);
            ui.label(
                RichText::new(number)
                    .monospace()
                    .color(ctx.theme.role(SemanticRole::Muted)),
            );
        }
        ui.label(
            RichText::new(truncate_line(line_text, usize::from(text_cols)))
                .monospace()
                .color(ctx.theme.role(SemanticRole::Fg)),
        );
    });
}

fn render_status_footer(ui: &mut Ui, ctx: &PanelContext<'_>) {
    ui.add_space(4.0);
    ui.label(
        RichText::new(format_preview_status(ctx)).color(ctx.theme.role(SemanticRole::Muted)),
    );
}

fn format_preview_status(ctx: &PanelContext<'_>) -> String {
    let preview = &ctx.state.preview;
    if preview.loading && preview.lines.is_empty() {
        return "Loading…".to_string();
    }
    if preview.loading {
        let path = preview
            .path_display()
            .unwrap_or_else(|| "No file selected".to_string());
        return format!("{path} | reloading…");
    }

    let path = preview
        .path_display()
        .unwrap_or_else(|| "No file selected".to_string());

    if preview.binary {
        return format!("{path} | binary | {} bytes", preview.file_size);
    }
    if preview.oversize {
        return format!("{path} | too large | {} bytes", preview.file_size);
    }
    if preview.load_error.is_some() {
        return path;
    }

    let encoding = if preview.lossy_utf8 {
        "UTF-8 (lossy)"
    } else {
        "UTF-8"
    };
    let truncated = if preview.truncated {
        " | truncated"
    } else {
        ""
    };
    format!(
        "{path} | {} lines | {encoding}{truncated} · ↑/↓ scroll · e editor",
        preview.line_count()
    )
}

fn max_text_cols(ui: &Ui, gutter_width: usize) -> u16 {
    let clip_width = ui.clip_rect().width().max(1.0);
    let approx_char_width = ui
        .style()
        .text_styles
        .get(&egui::TextStyle::Monospace)
        .map(|font| font.size)
        .unwrap_or(13.0);
    let available = clip_width - gutter_width as f32 * approx_char_width;
    (available / approx_char_width).floor().max(1.0) as u16
}

fn line_number_width(line_count: usize) -> usize {
    let digits = line_count.max(1).ilog10() as usize + 1;
    digits + 1
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use kiwi_core::config::ResolvedConfig;
    use kiwi_core::preview::PreviewState;
    use kiwi_core::state::{AppState, ViewportMetrics};
    use kiwi_core::theme::{load_theme_with_capabilities, TerminalCapabilities};

    use super::*;
    use crate::dock::{PanelContext, PtySurfaceState};
    use crate::theme::GuiTheme;

    fn test_ctx() -> (AppState, GuiTheme) {
        let config = ResolvedConfig::default();
        let theme_palette =
            load_theme_with_capabilities(&config.theme, TerminalCapabilities::TrueColor)
                .expect("theme");
        let state = AppState::from_startup(
            PathBuf::from("/tmp/kiwi"),
            true,
            config.clone(),
            theme_palette,
            TerminalCapabilities::TrueColor,
            ViewportMetrics::default(),
        );
        let gui_theme = GuiTheme::from_palette(&state.theme, &config.gui);
        (state, gui_theme)
    }

    #[test]
    fn status_includes_line_count_and_encoding() {
        let (mut state, theme) = test_ctx();
        state.preview = PreviewState {
            path: Some(PathBuf::from("/tmp/a.rs")),
            lines: vec!["one".to_string(), "two".to_string()],
            ..PreviewState::default()
        };
        let mut noop = |_cmd: AppCommand| false;
        let mut pty_surface = PtySurfaceState::default();
        let ctx = PanelContext {
            state: &mut state,
            theme: &theme,
            dispatch: &mut noop,
            pty_surface: &mut pty_surface,
            focused_dock_tab: None,
        };
        let status = format_preview_status(&ctx);
        assert!(status.contains("2 lines"));
        assert!(status.contains("UTF-8"));
    }

    #[test]
    fn preview_row_height_fits_two_lines_in_view() {
        use crate::dock::panels::layout::row_count_for_height;
        assert_eq!(row_count_for_height(36.0, ROW_HEIGHT), 2);
    }
}
