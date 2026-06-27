//! Git status dock panel (SPEC-008 / SPEC-022).

use egui::{Color32, RichText, Ui};
use kiwi_core::events::AppCommand;
use kiwi_core::git::{
    build_panel_rows, clamp_git_scroll, git_selected_row_index, GitFileStatus, GitPanelRow,
};
use kiwi_core::theme::SemanticRole;

use super::layout::render_virtual_rows;
use crate::dock::context::PanelContext;

const ROW_HEIGHT: f32 = 18.0;

pub fn render(ui: &mut Ui, ctx: &mut PanelContext<'_>) {
    render_branch_line(ui, ctx);
    ui.add_space(4.0);
    render_file_list(ui, ctx);
    ui.add_space(4.0);
    render_footer_line(ui, ctx);
}

/// Keyboard shortcuts when the Git Status dock tab is focused.
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
        return Some(AppCommand::GitMoveSelection(1));
    }
    if ctx.input(|input| input.key_pressed(egui::Key::ArrowUp)) {
        return Some(AppCommand::GitMoveSelection(-1));
    }
    if ctx.input(|input| input.key_pressed(egui::Key::R)) {
        return Some(AppCommand::GitRefresh);
    }
    if ctx.input(|input| input.key_pressed(egui::Key::Enter)) {
        return Some(AppCommand::GitOpenSelected);
    }

    None
}

fn render_branch_line(ui: &mut Ui, ctx: &PanelContext<'_>) {
    let text = branch_line_text(ctx);
    ui.label(
        RichText::new(text)
            .color(ctx.theme.role(SemanticRole::Muted))
            .strong(),
    );
}

fn render_footer_line(ui: &mut Ui, ctx: &PanelContext<'_>) {
    let text = footer_line_text(ctx);
    ui.label(RichText::new(text).color(ctx.theme.role(SemanticRole::Muted)));
}

fn render_file_list(ui: &mut Ui, ctx: &mut PanelContext<'_>) {
    let show_untracked = ctx.state.config.git.show_untracked;
    let rows = build_panel_rows(&ctx.state.git.file_entries, show_untracked);
    let total_rows = rows.len();
    let selected_row = git_selected_row_index(&ctx.state.git, &rows);

    if total_rows == 0 {
        ui.label(RichText::new(empty_list_message(ctx)).color(ctx.theme.role(SemanticRole::Muted)));
        ctx.state.viewport.git_rows = 1;
        return;
    }

    clamp_git_scroll(&mut ctx.state.git, &rows, 1);

    let mut scroll_offset = ctx.state.git.scroll_offset;
    let viewport_rows = render_virtual_rows(
        ui,
        ROW_HEIGHT,
        total_rows,
        &mut scroll_offset,
        |ui, row_index| {
            render_row(ui, ctx, &rows[row_index], row_index, selected_row);
        },
    );
    ctx.state.git.scroll_offset = scroll_offset;
    ctx.state.viewport.git_rows = viewport_rows;
    clamp_git_scroll(&mut ctx.state.git, &rows, viewport_rows);
}

fn render_row(
    ui: &mut Ui,
    ctx: &mut PanelContext<'_>,
    row: &GitPanelRow,
    row_index: usize,
    selected_row: Option<usize>,
) {
    match row {
        GitPanelRow::Header { label, count } => {
            ui.horizontal(|ui| {
                ui.set_min_height(ROW_HEIGHT);
                ui.label(
                    RichText::new(format!("{label} ({count})"))
                        .color(ctx.theme.role(SemanticRole::Muted))
                        .strong(),
                );
            });
        }
        GitPanelRow::File { path, status } => {
            let selected = selected_row == Some(row_index);
            let color = file_row_color(ctx, *status, selected);
            let prefix = if selected { "▸ " } else { "  " };
            let label = format!("{prefix}{} {path}", status.badge());

            ui.horizontal(|ui| {
                ui.set_min_height(ROW_HEIGHT);
                let response = ui.add(
                    egui::Label::new(RichText::new(label).color(color).monospace())
                        .sense(egui::Sense::click()),
                );
                if response.clicked() {
                    let _ = (ctx.dispatch)(AppCommand::GitSelect(row_index));
                }
                if response.double_clicked() {
                    let _ = (ctx.dispatch)(AppCommand::GitSelect(row_index));
                    let _ = (ctx.dispatch)(AppCommand::GitOpenSelected);
                }
            });
        }
    }
}

fn branch_line_text(ctx: &PanelContext<'_>) -> String {
    if !ctx.state.workspace_meta.is_git_repo {
        return "Not a git repository".to_string();
    }
    if ctx.state.git.loading {
        return "Refreshing…".to_string();
    }
    if let Some(error) = &ctx.state.git.error {
        return error.clone();
    }

    let mut parts = Vec::new();
    if let Some(branch) = &ctx.state.git.branch {
        parts.push(branch.clone());
    }
    if ctx.state.git.ahead > 0 || ctx.state.git.behind > 0 {
        parts.push(format!(
            "↑{} ↓{}",
            ctx.state.git.ahead, ctx.state.git.behind
        ));
    }
    if parts.is_empty() {
        "No branch".to_string()
    } else {
        parts.join(" · ")
    }
}

fn footer_line_text(ctx: &PanelContext<'_>) -> String {
    if !ctx.state.workspace_meta.is_git_repo {
        return "Git features disabled".to_string();
    }
    if ctx.state.git.loading {
        return "Refreshing…".to_string();
    }
    if ctx.state.git.file_entries.is_empty() {
        return "Clean working tree · R refresh".to_string();
    }
    "↑/↓ move · Enter diff · R refresh".to_string()
}

fn empty_list_message(ctx: &PanelContext<'_>) -> &'static str {
    if !ctx.state.workspace_meta.is_git_repo {
        "Open a git repository"
    } else if ctx.state.git.loading {
        "Refreshing…"
    } else {
        "Clean working tree"
    }
}

fn file_row_color(ctx: &PanelContext<'_>, status: GitFileStatus, selected: bool) -> Color32 {
    if selected {
        return ctx.theme.role(SemanticRole::Accent);
    }
    ctx.theme.role(status.semantic_role())
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use kiwi_core::config::ResolvedConfig;
    use kiwi_core::git::{GitFileEntry, GitFileStatus};
    use kiwi_core::state::{AppState, GitState, ViewportMetrics};
    use kiwi_core::theme::{load_theme_with_capabilities, SemanticRole, TerminalCapabilities};

    use super::*;
    use crate::dock::{PtySurfaceState};
    use crate::theme::GuiTheme;

    fn test_panel() -> (AppState, GuiTheme) {
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

    fn panel_ctx<'a>(
        state: &'a mut AppState,
        theme: &'a GuiTheme,
        dispatch: &'a mut dyn FnMut(kiwi_core::events::AppCommand) -> bool,
        pty_surface: &'a mut PtySurfaceState,
    ) -> PanelContext<'a> {
        PanelContext {
            state,
            theme,
            dispatch,
            pty_surface,
            focused_dock_tab: None,
        }
    }

    #[test]
    fn branch_line_shows_ahead_behind() {
        let (mut state, theme) = test_panel();
        state.git = GitState {
            branch: Some("main".to_string()),
            ahead: 2,
            behind: 1,
            ..GitState::default()
        };
        let mut noop = |_cmd: kiwi_core::events::AppCommand| false;
        let mut pty_surface = PtySurfaceState::default();
        let ctx = panel_ctx(&mut state, &theme, &mut noop, &mut pty_surface);
        assert!(branch_line_text(&ctx).contains("main"));
        assert!(branch_line_text(&ctx).contains("↑2 ↓1"));
    }

    #[test]
    fn non_git_repo_shows_disabled_message() {
        let (mut state, theme) = test_panel();
        state.workspace_meta.is_git_repo = false;
        let mut noop = |_cmd: kiwi_core::events::AppCommand| false;
        let mut pty_surface = PtySurfaceState::default();
        let ctx = panel_ctx(&mut state, &theme, &mut noop, &mut pty_surface);
        assert_eq!(branch_line_text(&ctx), "Not a git repository");
        assert_eq!(footer_line_text(&ctx), "Git features disabled");
    }

    #[test]
    fn selected_file_uses_accent_color() {
        let (mut state, theme) = test_panel();
        state.git.file_entries = vec![GitFileEntry {
            path: "src/main.rs".to_string(),
            status: GitFileStatus::Modified,
        }];
        let mut noop = |_cmd: kiwi_core::events::AppCommand| false;
        let mut pty_surface = PtySurfaceState::default();
        let ctx = panel_ctx(&mut state, &theme, &mut noop, &mut pty_surface);
        let accent = file_row_color(&ctx, GitFileStatus::Modified, true);
        assert_eq!(accent, theme.role(SemanticRole::Accent));
    }

    #[test]
    fn unselected_file_uses_status_color() {
        let (mut state, theme) = test_panel();
        let mut noop = |_cmd: kiwi_core::events::AppCommand| false;
        let mut pty_surface = PtySurfaceState::default();
        let ctx = panel_ctx(&mut state, &theme, &mut noop, &mut pty_surface);
        let color = file_row_color(&ctx, GitFileStatus::Deleted, false);
        assert_eq!(color, theme.role(SemanticRole::GitDeleted));
    }
}
