//! [`egui_dock::TabViewer`] implementation for [`KiwiTab`].

use egui::{Ui, WidgetText};
use egui_dock::TabViewer;

use super::context::PanelContext;
use super::panels::render_panel;
use super::tab::KiwiTab;

pub struct KiwiTabViewer<'a> {
    pub ctx: PanelContext<'a>,
}

impl TabViewer for KiwiTabViewer<'_> {
    type Tab = KiwiTab;

    fn title(&mut self, tab: &mut Self::Tab) -> WidgetText {
        tab.title().into()
    }

    fn ui(&mut self, ui: &mut Ui, tab: &mut Self::Tab) {
        render_panel(*tab, ui, &self.ctx);
    }

    fn closeable(&mut self, tab: &mut Self::Tab) -> bool {
        tab.closable()
    }
}
