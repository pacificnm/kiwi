//! Search dock panel — query, mode toggle, and virtualized results (#192 / SPEC-007).

use egui::{RichText, Ui};
use kiwi_core::events::AppCommand;
use kiwi_core::search::{SearchMode, SearchResult};
use kiwi_core::theme::SemanticRole;

use super::layout::render_virtual_rows;
use crate::dock::context::PanelContext;
use crate::dock::tab::KiwiTab;

const ROW_HEIGHT: f32 = 18.0;
const SEARCH_QUERY_ID: &str = "search_query_input";

pub fn render(ui: &mut Ui, ctx: &mut PanelContext<'_>) {
    render_header(ui, ctx);
    ui.separator();
    render_query_line(ui, ctx);
    ui.add_space(4.0);
    render_results(ui, ctx);
    ui.add_space(4.0);
    render_footer(ui, ctx);
}

fn render_header(ui: &mut Ui, ctx: &mut PanelContext<'_>) {
    ui.horizontal(|ui| {
        ui.label(
            RichText::new(format!("Search: {}", ctx.state.search.mode.label()))
                .color(ctx.theme.role(SemanticRole::Muted))
                .strong(),
        );
        if ui
            .button(format!("Mode: {}", ctx.state.search.mode.label()))
            .on_hover_text("Toggle Files / Content (Ctrl+M)")
            .clicked()
        {
            let next = match ctx.state.search.mode {
                SearchMode::Files => SearchMode::Content,
                SearchMode::Content => SearchMode::Files,
            };
            let _ = (ctx.dispatch)(AppCommand::SearchSetMode(next));
        }
    });
}

fn render_query_line(ui: &mut Ui, ctx: &mut PanelContext<'_>) {
    let mut query = ctx.state.search.query.clone();
    let response = ui.horizontal(|ui| {
        ui.label(RichText::new("/").monospace());
        ui.add(
            egui::TextEdit::singleline(&mut query)
                .id(egui::Id::new(SEARCH_QUERY_ID))
                .desired_width(f32::INFINITY)
                .hint_text("type to search"),
        )
    });
    if ctx.is_dock_tab_focused(KiwiTab::Search) {
        response.inner.request_focus();
    }
    if response.inner.changed() {
        let _ = (ctx.dispatch)(AppCommand::SearchSetQuery(query));
    }
}

fn render_results(ui: &mut Ui, ctx: &mut PanelContext<'_>) {
    let total_rows = ctx.state.search.results.len();
    if total_rows == 0 {
        ui.label(
            RichText::new(empty_results_message(ctx))
                .color(ctx.theme.role(SemanticRole::Muted)),
        );
        ctx.state.viewport.search_rows = 1;
        return;
    }

    let selected = ctx.state.search.selected;
    let mut scroll_offset = ctx.state.search.scroll_offset;
    let viewport_rows = render_virtual_rows(
        ui,
        ROW_HEIGHT,
        total_rows,
        &mut scroll_offset,
        |ui, row_index| {
            render_result_row(ui, ctx, row_index, selected);
        },
    );
    ctx.state.search.scroll_offset = scroll_offset;
    ctx.state.viewport.search_rows = viewport_rows;
}

fn render_result_row(
    ui: &mut Ui,
    ctx: &mut PanelContext<'_>,
    row_index: usize,
    selected: usize,
) {
    let Some(result) = ctx.state.search.results.get(row_index) else {
        return;
    };

    let is_selected = row_index == selected;
    let color = if is_selected {
        ctx.theme.role(SemanticRole::Accent)
    } else {
        ctx.theme.role(SemanticRole::Fg)
    };
    let prefix = if is_selected { "▸ " } else { "  " };
    let label = format!(
        "{prefix}{}",
        result_label(result, ctx.state.search.mode)
    );

    ui.horizontal(|ui| {
        ui.set_min_height(ROW_HEIGHT);
        let response = ui.add(
            egui::Label::new(RichText::new(label).color(color).monospace())
                .sense(egui::Sense::click()),
        );
        if response.clicked() {
            let _ = (ctx.dispatch)(AppCommand::SearchSelect(row_index));
        }
        if response.double_clicked() {
            let _ = (ctx.dispatch)(AppCommand::SearchSelect(row_index));
            let _ = (ctx.dispatch)(AppCommand::PreviewFile {
                path: result.path.clone(),
                line: result.line,
            });
            let _ = (ctx.dispatch)(AppCommand::Navigation(
                kiwi_core::navigation::NavCommand::SelectLeftTab(
                    kiwi_core::navigation::LeftNavTab::Search,
                ),
            ));
        }
    });
}

fn render_footer(ui: &mut Ui, ctx: &PanelContext<'_>) {
    ui.label(
        RichText::new(footer_text(ctx)).color(ctx.theme.role(SemanticRole::Muted)),
    );
}

fn footer_text(ctx: &PanelContext<'_>) -> String {
    if ctx.state.search.running {
        return "Searching…".to_string();
    }
    if let Some(error) = &ctx.state.search.error {
        return error.clone();
    }
    if ctx.state.search.truncated {
        return format!(
            "{} results (truncated) · Enter preview · e editor · Ctrl+M mode",
            ctx.state.search.results.len()
        );
    }
    if ctx.state.search.query.is_empty() {
        return "Type to search · Ctrl+M mode".to_string();
    }
    if ctx.state.search.results.is_empty() {
        return "No results · Ctrl+M mode".to_string();
    }
    format!(
        "{} results · Enter preview · e editor · Ctrl+M mode",
        ctx.state.search.results.len()
    )
}

fn empty_results_message(ctx: &PanelContext<'_>) -> &'static str {
    if ctx.state.search.query.is_empty() {
        "Type to search files or content"
    } else if ctx.state.search.running {
        "Searching…"
    } else {
        "No matches"
    }
}

fn result_label(result: &SearchResult, mode: SearchMode) -> String {
    match mode {
        SearchMode::Files => result.id.clone(),
        SearchMode::Content => {
            if result.preview.is_empty() {
                result.id.clone()
            } else {
                format!("{}  {}", result.id, result.preview)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use kiwi_core::config::ResolvedConfig;
    use kiwi_core::search::{SearchMode, SearchResult};
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
    fn content_result_label_includes_preview_snippet() {
        let result = SearchResult::content(
            PathBuf::from("a.rs"),
            1,
            "hello".to_string(),
        );
        let label = result_label(&result, SearchMode::Content);
        assert!(label.contains("hello"));
    }

    #[test]
    fn footer_shows_truncation_notice() {
        let (mut state, theme) = test_ctx();
        state.search.query = "foo".to_string();
        state.search.truncated = true;
        state.search.results = vec![SearchResult::file(
            PathBuf::from("a.rs"),
            "a.rs".to_string(),
        )];
        let mut noop = |_cmd: AppCommand| false;
        let mut pty_surface = PtySurfaceState::default();
        let ctx = PanelContext {
            state: &mut state,
            theme: &theme,
            dispatch: &mut noop,
            pty_surface: &mut pty_surface,
            focused_dock_tab: None,
        };
        assert!(footer_text(&ctx).contains("truncated"));
    }
}
