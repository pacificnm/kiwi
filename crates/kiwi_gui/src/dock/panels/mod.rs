//! Dock panel renderers.

mod placeholder;

use egui::Ui;

use super::context::PanelContext;
use super::tab::KiwiTab;

pub use placeholder::render_placeholder;

pub fn render_panel(tab: KiwiTab, ui: &mut Ui, ctx: &PanelContext<'_>) {
    render_placeholder(ui, tab, ctx);
}
