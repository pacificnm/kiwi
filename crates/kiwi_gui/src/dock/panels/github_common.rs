//! Shared GitHub panel helpers for Issues and PRs dock tabs (#191).

use egui::{Color32, RichText, Ui};
use kiwi_core::events::AppCommand;
use kiwi_core::github::{GitHubAuthErrorKind, IssueState, PrState};
use kiwi_core::navigation::{FocusTarget, MainTab, NavCommand};
use kiwi_core::state::AppState;
use kiwi_core::theme::SemanticRole;

use super::layout::render_virtual_rows;
use crate::dock::context::PanelContext;
use crate::dock::tab::KiwiTab;
use crate::theme::GuiTheme;

pub const LIST_ROW_HEIGHT: f32 = 18.0;
pub const DETAIL_ROW_HEIGHT: f32 = 16.0;

/// Dispatch nav sync only when this dock tab has egui_dock focus.
pub fn sync_github_navigation(ctx: &mut PanelContext<'_>, tab: KiwiTab) {
    if !ctx.is_dock_tab_focused(tab) {
        return;
    }
    for command in super::github_input::navigation_sync_commands(ctx.state, tab) {
        let _ = (ctx.dispatch)(command);
    }
}

/// Returns `true` when the panel should stop rendering (auth pending or error).
pub fn render_auth_gate(ui: &mut Ui, ctx: &PanelContext<'_>, tab: KiwiTab) -> bool {
    if let Some(label) = github_status_label(ctx.state, tab) {
        ui.colored_label(ctx.theme.role(SemanticRole::Muted), label);
    }

    if let Some(message) = auth_error_message(ctx.state) {
        ui.add_space(8.0);
        render_auth_error(ui, ctx.theme, message, ctx.state.github.error_kind);
        return true;
    }

    false
}

pub fn github_status_label(state: &AppState, tab: KiwiTab) -> Option<&'static str> {
    if state.github.loading && !state.github.auth_checked {
        return Some("Checking GitHub authentication…");
    }
    if state.github.auth_ok {
        return match tab {
            KiwiTab::GitHubIssues if state.github.issues_loading => Some("Loading issues…"),
            KiwiTab::GitHubPrs if state.github.prs_loading => Some("Loading pull requests…"),
            KiwiTab::Issues if state.github.issue_detail_loading => Some("Loading issue detail…"),
            _ => None,
        };
    }
    if !state.github.auth_checked {
        return Some("Waiting for GitHub auth check…");
    }
    None
}

pub fn auth_error_message(state: &AppState) -> Option<&str> {
    if state.github.auth_ok {
        return None;
    }
    if !state.github.auth_checked && !state.github.loading {
        return None;
    }
    state
        .github
        .error
        .as_deref()
        .filter(|message| !message.is_empty())
}

pub fn render_auth_error(
    ui: &mut Ui,
    theme: &GuiTheme,
    message: &str,
    kind: Option<GitHubAuthErrorKind>,
) {
    let heading = match kind {
        Some(GitHubAuthErrorKind::NotInstalled) => "GitHub CLI required",
        Some(GitHubAuthErrorKind::NotAuthenticated) => "GitHub login required",
        Some(GitHubAuthErrorKind::CommandFailed) | None => "GitHub auth check failed",
    };
    ui.colored_label(
        theme.role(SemanticRole::AgentError),
        RichText::new(heading).strong(),
    );
    ui.add_space(4.0);
    ui.colored_label(theme.role(SemanticRole::Muted), message);
}

#[must_use]
pub fn issue_state_color(theme: &GuiTheme, state: IssueState) -> Color32 {
    match state {
        IssueState::Open => theme.role(SemanticRole::IssueOpen),
        IssueState::Closed => theme.role(SemanticRole::IssueClosed),
    }
}

#[must_use]
pub fn pr_state_color(theme: &GuiTheme, state: PrState) -> Color32 {
    match state {
        PrState::Open => theme.role(SemanticRole::PrOpen),
        PrState::Draft => theme.role(SemanticRole::PrDraft),
        PrState::Merged => theme.role(SemanticRole::PrMerged),
        PrState::Closed => theme.role(SemanticRole::PrClosed),
    }
}

/// Select issue row, switch to Issues main tab, and focus center (opens detail tab).
pub fn select_issue_commands(row_index: usize) -> [AppCommand; 3] {
    [
        AppCommand::GitHubSelectIssue(row_index),
        AppCommand::Navigation(NavCommand::SelectMainTabUnpaired(MainTab::Issues)),
        AppCommand::Navigation(NavCommand::SetFocus(FocusTarget::Main)),
    ]
}

/// Mouse/keyboard list interaction: select only, or select and open the main detail tab.
#[must_use]
pub fn issue_list_click_commands(row_index: usize, open_detail: bool) -> Vec<AppCommand> {
    if open_detail {
        select_issue_commands(row_index).to_vec()
    } else {
        vec![AppCommand::GitHubSelectIssue(row_index)]
    }
}

/// Mouse/keyboard list interaction: select only, or select and open the main detail tab.
#[must_use]
pub fn pr_list_click_commands(row_index: usize, open_detail: bool) -> Vec<AppCommand> {
    if open_detail {
        select_pr_commands(row_index).to_vec()
    } else {
        vec![AppCommand::GitHubSelectPr(row_index)]
    }
}

/// Select branch row, switch to Branches main tab, and focus center (opens detail tab).
pub fn select_branch_commands(row_index: usize) -> [AppCommand; 3] {
    [
        AppCommand::BranchSelect(row_index),
        AppCommand::Navigation(NavCommand::SelectMainTabUnpaired(MainTab::Branches)),
        AppCommand::Navigation(NavCommand::SetFocus(FocusTarget::Main)),
    ]
}

/// Mouse/keyboard list interaction: select only, or select and open the main detail tab.
#[must_use]
pub fn branch_list_click_commands(row_index: usize, open_detail: bool) -> Vec<AppCommand> {
    if open_detail {
        select_branch_commands(row_index).to_vec()
    } else {
        vec![AppCommand::BranchSelect(row_index)]
    }
}

/// Select PR row, switch to PRs main tab, and focus center (opens detail tab).
pub fn select_pr_commands(row_index: usize) -> [AppCommand; 3] {
    [
        AppCommand::GitHubSelectPr(row_index),
        AppCommand::Navigation(NavCommand::SelectMainTabUnpaired(MainTab::Prs)),
        AppCommand::Navigation(NavCommand::SetFocus(FocusTarget::Main)),
    ]
}

pub fn render_detail_lines(
    ui: &mut Ui,
    ctx: &mut PanelContext<'_>,
    lines: &[String],
    scroll_offset: &mut usize,
    loading: bool,
    error: Option<&str>,
    empty_hint: &str,
) -> usize {
    if loading && lines.is_empty() {
        ui.label(
            RichText::new("Loading detail…").color(ctx.theme.role(SemanticRole::Muted)),
        );
        ctx.state.viewport.github_detail_rows = 1;
        return 1;
    }

    if let Some(error) = error {
        ui.colored_label(ctx.theme.role(SemanticRole::AgentError), error);
        ctx.state.viewport.github_detail_rows = 1;
        return 1;
    }

    if lines.is_empty() {
        ui.label(RichText::new(empty_hint).color(ctx.theme.role(SemanticRole::Muted)));
        ctx.state.viewport.github_detail_rows = 1;
        return 1;
    }

    let total_rows = lines.len();
    let mut offset = *scroll_offset;
    let layout = render_virtual_rows(
        ui,
        DETAIL_ROW_HEIGHT,
        total_rows,
        &mut offset,
        |ui, row_index| {
            let text = &lines[row_index];
            let color = ctx.theme.role(SemanticRole::Fg);
            let rich = if row_index == 0 {
                RichText::new(text).color(color).strong()
            } else {
                RichText::new(text).color(color)
            };
            ui.horizontal(|ui| {
                ui.set_min_height(DETAIL_ROW_HEIGHT);
                ui.label(rich);
            });
        },
    );
    *scroll_offset = offset;
    ctx.state.viewport.github_detail_rows = layout.viewport_rows;
    layout.viewport_rows
}

#[cfg(test)]
mod tests {
    use kiwi_core::config::ResolvedConfig;
    use kiwi_core::theme::{load_theme_with_capabilities, TerminalCapabilities};

    use super::*;

    fn make_state() -> kiwi_core::state::AppState {
        use std::path::PathBuf;
        use kiwi_core::state::{AppState, ViewportMetrics};
        use kiwi_core::theme::{load_theme_with_capabilities, TerminalCapabilities};
        AppState::from_startup(
            PathBuf::from("/tmp/kiwi"),
            true,
            ResolvedConfig::default(),
            load_theme_with_capabilities(&ResolvedConfig::default().theme, TerminalCapabilities::TrueColor)
                .expect("theme"),
            TerminalCapabilities::TrueColor,
            ViewportMetrics::default(),
        )
    }

    #[test]
    fn github_status_label_issues_list_loading() {
        let mut state = make_state();
        state.github.auth_ok = true;
        state.github.issues_loading = true;
        assert_eq!(
            github_status_label(&state, KiwiTab::GitHubIssues),
            Some("Loading issues…")
        );
        // Issues detail tab must NOT show the list-loading label.
        assert_ne!(
            github_status_label(&state, KiwiTab::Issues),
            Some("Loading issues…")
        );
    }

    #[test]
    fn github_status_label_issue_detail_loading() {
        let mut state = make_state();
        state.github.auth_ok = true;
        state.github.issue_detail_loading = true;
        assert_eq!(
            github_status_label(&state, KiwiTab::Issues),
            Some("Loading issue detail…")
        );
    }

    #[test]
    fn issue_open_uses_issue_open_role() {
        let config = ResolvedConfig::default();
        let palette = load_theme_with_capabilities(
            &config.theme,
            TerminalCapabilities::TrueColor,
        )
        .expect("theme");
        let theme = crate::theme::GuiTheme::from_palette(&palette, &config.gui);
        let color = issue_state_color(&theme, IssueState::Open);
        assert_eq!(color, theme.role(SemanticRole::IssueOpen));
    }

    #[test]
    fn select_issue_commands_focuses_main_issues_tab() {
        let commands = select_issue_commands(2);
        assert!(commands.iter().any(|cmd| matches!(
            cmd,
            AppCommand::GitHubSelectIssue(2)
        )));
        assert!(commands.iter().any(|cmd| matches!(
            cmd,
            AppCommand::Navigation(NavCommand::SelectMainTabUnpaired(MainTab::Issues))
        )));
        assert!(commands.iter().any(|cmd| matches!(
            cmd,
            AppCommand::Navigation(NavCommand::SetFocus(FocusTarget::Main))
        )));
    }

    #[test]
    fn issue_list_click_select_only_does_not_open_main_tab() {
        let commands = issue_list_click_commands(1, false);
        assert_eq!(commands, vec![AppCommand::GitHubSelectIssue(1)]);
    }

    #[test]
    fn issue_list_click_open_dispatches_full_select_flow() {
        let commands = issue_list_click_commands(1, true);
        assert_eq!(commands.len(), 3);
        assert!(commands.iter().any(|cmd| matches!(
            cmd,
            AppCommand::GitHubSelectIssue(1)
        )));
        assert!(commands.iter().any(|cmd| matches!(
            cmd,
            AppCommand::Navigation(NavCommand::SelectMainTabUnpaired(MainTab::Issues))
        )));
    }
}
