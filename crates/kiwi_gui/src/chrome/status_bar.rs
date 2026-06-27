//! Bottom status bar with repo, branch, git, and agent segments (SPEC-019).

use egui::{Align, Layout, RichText};
use kiwi_core::agent::AgentStatus;
use kiwi_core::state::AppState;
use kiwi_core::status_bar::{compute_status_bar, fit_status_bar_segments, BRAND, SEPARATOR};
use kiwi_core::theme::SemanticRole;

use crate::theme::GuiTheme;

const STATUS_BAR_HEIGHT: f32 = 24.0;

pub fn render_status_bar(ctx: &egui::Context, theme: &GuiTheme, state: &AppState) {
    egui::TopBottomPanel::bottom("status_bar")
        .min_height(STATUS_BAR_HEIGHT)
        .max_height(STATUS_BAR_HEIGHT)
        .show(ctx, |ui| {
            let snapshot = compute_status_bar(state);
            let width = ui.available_width().max(0.0) as u16;
            let segments = fit_status_bar_segments(&snapshot, width);

            ui.with_layout(Layout::left_to_right(Align::Center), |ui| {
                ui.spacing_mut().item_spacing.x = 0.0;
                render_label(ui, theme, BRAND, SemanticRole::Fg);

                if let Some(repo) = segments.repo {
                    render_separator(ui, theme);
                    render_label(ui, theme, &repo, SemanticRole::Fg);
                }

                render_separator(ui, theme);
                render_label(ui, theme, &segments.root, SemanticRole::Fg);

                render_separator(ui, theme);
                render_label(ui, theme, &segments.branch, SemanticRole::Fg);

                render_separator(ui, theme);
                let agent_role = agent_semantic_role(&snapshot);
                render_label(ui, theme, &segments.agent, agent_role);

                render_separator(ui, theme);
                let git_role = if snapshot.git_modified {
                    SemanticRole::GitModified
                } else {
                    SemanticRole::Fg
                };
                render_label(ui, theme, &segments.git, git_role);

                if let Some(issue) = segments.issue {
                    render_separator(ui, theme);
                    render_label(ui, theme, &issue, SemanticRole::IssueOpen);
                }
            });
        });
}

fn agent_semantic_role(snapshot: &kiwi_core::status_bar::StatusBarSnapshot) -> SemanticRole {
    match snapshot.agent_status {
        AgentStatus::Idle if snapshot.agent_running => SemanticRole::AgentExecuting,
        AgentStatus::Idle => SemanticRole::Muted,
        status => status.semantic_role(),
    }
}

fn render_separator(ui: &mut egui::Ui, theme: &GuiTheme) {
    let muted = theme.role(SemanticRole::Muted);
    ui.label(RichText::new(SEPARATOR).color(muted));
}

fn render_label(ui: &mut egui::Ui, theme: &GuiTheme, text: &str, role: SemanticRole) {
    ui.label(RichText::new(text).color(theme.role(role)));
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use kiwi_core::config::ResolvedConfig;
    use kiwi_core::state::{AppState, ViewportMetrics};
    use kiwi_core::status_bar::format_status_line;
    use kiwi_core::theme::{load_theme_with_capabilities, TerminalCapabilities};

    use super::*;

    fn test_theme() -> GuiTheme {
        let palette = load_theme_with_capabilities(
            &ResolvedConfig::default().theme,
            TerminalCapabilities::TrueColor,
        )
        .expect("theme");
        GuiTheme::from_palette(&palette, &ResolvedConfig::default().gui)
    }

    fn test_state() -> AppState {
        AppState::from_startup(
            PathBuf::from("/tmp/kiwi"),
            true,
            ResolvedConfig::default(),
            load_theme_with_capabilities(
                &ResolvedConfig::default().theme,
                TerminalCapabilities::TrueColor,
            )
            .expect("theme"),
            TerminalCapabilities::TrueColor,
            ViewportMetrics::default(),
        )
    }

    #[test]
    fn status_bar_line_format_matches_core_helper() {
        let state = test_state();
        let snapshot = compute_status_bar(&state);
        let line = format_status_line(&snapshot, 120);
        assert!(line.contains("Root: kiwi"));
        assert!(line.starts_with("Kiwi"));
    }

    #[test]
    fn gui_theme_provides_status_bar_roles() {
        let theme = test_theme();
        assert_ne!(
            theme.role(SemanticRole::Fg),
            theme.role(SemanticRole::Muted)
        );
    }
}
