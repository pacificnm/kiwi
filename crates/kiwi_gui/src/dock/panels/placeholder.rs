//! Placeholder panel bodies for dock tabs not yet implemented (SPEC-022 G4).
//!
//! [`KiwiTab::Terminal`] and [`KiwiTab::Agent`] render via [`super::terminal`] and
//! [`super::agent`] — they must not use this module.

use egui::Ui;
use kiwi_core::theme::SemanticRole;

use crate::dock::context::PanelContext;
use crate::dock::tab::KiwiTab;

pub fn render_placeholder(ui: &mut Ui, tab: KiwiTab, ctx: &mut PanelContext<'_>) {
    // Safety net: PTY tabs must never show the generic stub (see `render_panel`).
    match tab {
        KiwiTab::Terminal => return super::terminal::render(ui, ctx),
        KiwiTab::Agent => return super::agent::render(ui, ctx),
        _ => {}
    }

    ui.heading(tab.title());
    ui.separator();
    ui.label("Panel content arrives in a later milestone.");
    if let Some(hint) = state_hint(tab, ctx) {
        ui.add_space(8.0);
        ui.colored_label(ctx.theme.role(SemanticRole::Muted), hint);
    }
}

fn state_hint(tab: KiwiTab, ctx: &mut PanelContext<'_>) -> Option<String> {
    match tab {
        KiwiTab::Explorer => Some(format!("Root: {}", ctx.state.file_tree.root.display())),
        KiwiTab::GitStatus => {
            let count = ctx.state.git.changed_count();
            Some(format!("{count} changed file(s)"))
        }
        KiwiTab::GitDiff => ctx
            .state
            .diff
            .selected_path
            .clone()
            .map(|path| format!("Selected: {path}")),
        KiwiTab::GitHubIssues => Some(format!("{} issue(s) loaded", ctx.state.github.issues.len())),
        KiwiTab::GitHubPrs => Some(format!("{} PR(s) loaded", ctx.state.github.prs.len())),
        KiwiTab::Preview => ctx
            .state
            .preview
            .path
            .clone()
            .map(|path| format!("File: {}", path.display())),
        KiwiTab::Search => Some(format!(
            "Query: {}",
            if ctx.state.search.query.is_empty() {
                "(empty)".to_string()
            } else {
                ctx.state.search.query.clone()
            }
        )),
        KiwiTab::Terminal | KiwiTab::Agent => None,
        KiwiTab::Logs => Some(format!("{} log entries", ctx.state.logs.entries.len())),
        KiwiTab::Config => Some(format!("Theme: {}", ctx.state.theme.name)),
        KiwiTab::GitLog => None,
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use kiwi_core::config::ResolvedConfig;
    use kiwi_core::state::{AppState, ViewportMetrics};
    use kiwi_core::theme::{load_theme_with_capabilities, TerminalCapabilities};

    use super::*;
    use crate::dock::PtySurfaceState;
    use crate::theme::GuiTheme;

    fn test_context() -> (AppState, GuiTheme) {
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
    fn placeholder_redirects_terminal_and_agent_tabs() {
        let (mut state, theme) = test_context();
        let mut noop = |_cmd: kiwi_core::events::AppCommand| false;
        let mut pty_surface = PtySurfaceState::default();
        let mut ctx = PanelContext {
            state: &mut state,
            theme: &theme,
            dispatch: &mut noop,
            pty_surface: &mut pty_surface,
        };
        assert!(state_hint(KiwiTab::Terminal, &mut ctx).is_none());
        assert!(state_hint(KiwiTab::Agent, &mut ctx).is_none());
    }

    #[test]
    fn explorer_hint_includes_root_path() {
        let (mut state, theme) = test_context();
        let mut noop = |_cmd: kiwi_core::events::AppCommand| false;
        let mut pty_surface = PtySurfaceState::default();
        let mut ctx = PanelContext {
            state: &mut state,
            theme: &theme,
            dispatch: &mut noop,
            pty_surface: &mut pty_surface,
        };
        let hint = state_hint(KiwiTab::Explorer, &mut ctx).expect("hint");
        assert!(hint.contains("Root:"));
    }

    #[test]
    fn every_tab_variant_has_placeholder_hint_handler() {
        let (mut state, theme) = test_context();
        let mut noop = |_cmd: kiwi_core::events::AppCommand| false;
        let mut pty_surface = PtySurfaceState::default();
        let mut ctx = PanelContext {
            state: &mut state,
            theme: &theme,
            dispatch: &mut noop,
            pty_surface: &mut pty_surface,
        };
        for tab in KiwiTab::all_variants() {
            if matches!(tab, KiwiTab::Terminal | KiwiTab::Agent) {
                continue;
            }
            let _ = state_hint(*tab, &mut ctx);
        }
    }
}
