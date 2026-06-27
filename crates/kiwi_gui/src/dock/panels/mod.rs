//! Dock panel renderers.

mod explorer;
mod placeholder;

use egui::Ui;

use super::context::PanelContext;
use super::tab::KiwiTab;

pub use explorer::keyboard_action as explorer_keyboard_action;

pub fn render_panel(tab: KiwiTab, ui: &mut Ui, ctx: &mut PanelContext<'_>) {
    match tab {
        KiwiTab::Explorer => explorer::render(ui, ctx),
        _ => placeholder::render_placeholder(ui, tab, ctx),
    }
}
