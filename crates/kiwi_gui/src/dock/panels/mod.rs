//! Dock panel renderers.

mod explorer;
mod git_diff;
mod git_status;
mod layout;
mod placeholder;

use egui::Ui;

use super::context::PanelContext;
use super::tab::KiwiTab;

pub use explorer::keyboard_action as explorer_keyboard_action;
pub use git_diff::keyboard_action as git_diff_keyboard_action;
pub use git_status::keyboard_action as git_status_keyboard_action;

pub fn render_panel(tab: KiwiTab, ui: &mut Ui, ctx: &mut PanelContext<'_>) {
    match tab {
        KiwiTab::Explorer => explorer::render(ui, ctx),
        KiwiTab::GitStatus => git_status::render(ui, ctx),
        KiwiTab::GitDiff => git_diff::render(ui, ctx),
        _ => placeholder::render_placeholder(ui, tab, ctx),
    }
}
