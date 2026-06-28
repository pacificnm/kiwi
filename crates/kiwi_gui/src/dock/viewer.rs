//! [`egui_dock::TabViewer`] implementation for [`KiwiTab`].

use egui::{Align, Id, Ui, WidgetText};
use egui_dock::TabViewer;

use super::context::PanelContext;
use super::panels::render_panel;
use super::tab::KiwiTab;
use crate::navigation_bridge::navigation_commands_for_dock_tab;

pub struct KiwiTabViewer<'a> {
    pub ctx: PanelContext<'a>,
}

impl TabViewer for KiwiTabViewer<'_> {
    type Tab = KiwiTab;

    fn title(&mut self, tab: &mut Self::Tab) -> WidgetText {
        tab.title().into()
    }

    fn id(&mut self, tab: &mut Self::Tab) -> Id {
        Id::new(format!("kiwi_tab_{tab:?}"))
    }

    fn on_tab_button(&mut self, tab: &mut Self::Tab, response: &egui::Response) {
        if response.clicked() {
            for command in navigation_commands_for_dock_tab(*tab) {
                // navigation_commands_for_dock_tab only produces NavCommand variants,
                // which never trigger a quit. The egui_dock callback also returns ()
                // so propagating the quit signal upward is not possible (#283).
                let _ = (self.ctx.dispatch)(command);
            }
        }
    }

    fn ui(&mut self, ui: &mut Ui, tab: &mut Self::Tab) {
        render_panel(*tab, ui, &mut self.ctx);
        fix_blank_gap_at_top(ui);
    }

    fn closeable(&mut self, tab: &mut Self::Tab) -> bool {
        !matches!(tab, KiwiTab::Terminal)
    }

    /// Use egui_dock's tab [`ScrollArea`] for all panels. Nested scroll areas break scrolling.
    fn scroll_bars(&self, _tab: &Self::Tab) -> [bool; 2] {
        [true, true]
    }
}

/// egui_dock can leave a stale scroll offset so content sits below the tab top (blank gap).
fn fix_blank_gap_at_top(ui: &mut Ui) {
    let clip = ui.clip_rect();
    let content = ui.min_rect();
    if content.min.y > clip.min.y + 0.5 {
        ui.scroll_to_rect(content, Some(Align::TOP));
    }
}
