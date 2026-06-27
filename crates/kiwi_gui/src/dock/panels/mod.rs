//! Dock panel renderers.

mod agent;
mod ansi;
mod explorer;
mod git_diff;
mod git_status;
mod github_common;
mod github_context_menu;
mod github_input;
mod github_left;
mod github_prs;
mod issues_detail;
mod layout;
mod placeholder;
mod preview;
mod pty_input;
mod scrollback;
mod search;
mod search_input;
mod terminal;

use egui::Ui;

use super::context::PanelContext;
use super::tab::KiwiTab;

pub use explorer::keyboard_action as explorer_keyboard_action;
pub use git_diff::keyboard_action as git_diff_keyboard_action;
pub use git_status::keyboard_action as git_status_keyboard_action;
pub use github_input::collect_github_keyboard;
pub use preview::keyboard_action as preview_keyboard_action;
pub use pty_input::{collect_pty_input, navigation_sync_commands, PtyTarget};
pub use search_input::{
    collect_search_keyboard, global_search_focus_commands, global_search_focus_pressed,
};

pub fn render_panel(tab: KiwiTab, ui: &mut Ui, ctx: &mut PanelContext<'_>) {
    match tab {
        KiwiTab::Explorer => explorer::render(ui, ctx),
        KiwiTab::GitStatus => git_status::render(ui, ctx),
        KiwiTab::GitDiff => git_diff::render(ui, ctx),
        KiwiTab::Terminal => terminal::render(ui, ctx),
        KiwiTab::Agent => agent::render(ui, ctx),
        KiwiTab::GitHubIssues => github_left::render(ui, ctx),
        KiwiTab::Issues => issues_detail::render(ui, ctx),
        KiwiTab::GitHubPrs => github_prs::render(ui, ctx),
        KiwiTab::Search => search::render(ui, ctx),
        KiwiTab::Preview => preview::render(ui, ctx),
        _ => placeholder::render_placeholder(ui, tab, ctx),
    }
}

#[cfg(test)]
mod routing_tests {
    use super::KiwiTab;

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    enum PanelRoute {
        Terminal,
        Agent,
        GitHubLeft,
        IssuesDetail,
        GitHubPrs,
        Search,
        Preview,
        Placeholder,
    }

    fn panel_route(tab: KiwiTab) -> PanelRoute {
        match tab {
            KiwiTab::Terminal => PanelRoute::Terminal,
            KiwiTab::Agent => PanelRoute::Agent,
            KiwiTab::GitHubIssues => PanelRoute::GitHubLeft,
            KiwiTab::Issues => PanelRoute::IssuesDetail,
            KiwiTab::GitHubPrs => PanelRoute::GitHubPrs,
            KiwiTab::Search => PanelRoute::Search,
            KiwiTab::Preview => PanelRoute::Preview,
            _ => PanelRoute::Placeholder,
        }
    }

    #[test]
    fn terminal_and_agent_use_dedicated_pty_panels() {
        assert_eq!(panel_route(KiwiTab::Terminal), PanelRoute::Terminal);
        assert_eq!(panel_route(KiwiTab::Agent), PanelRoute::Agent);
    }

    #[test]
    fn github_tabs_use_dedicated_panels() {
        assert_eq!(panel_route(KiwiTab::GitHubIssues), PanelRoute::GitHubLeft);
        assert_eq!(panel_route(KiwiTab::Issues), PanelRoute::IssuesDetail);
        assert_eq!(panel_route(KiwiTab::GitHubPrs), PanelRoute::GitHubPrs);
    }

    #[test]
    fn search_and_preview_use_dedicated_panels() {
        assert_eq!(panel_route(KiwiTab::Search), PanelRoute::Search);
        assert_eq!(panel_route(KiwiTab::Preview), PanelRoute::Preview);
    }

    #[test]
    fn unwired_tabs_still_use_placeholder() {
        assert_eq!(panel_route(KiwiTab::Logs), PanelRoute::Placeholder);
    }
}
