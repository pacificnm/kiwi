//! [`egui_dock::TabViewer`] implementation for [`KiwiTab`].

use egui::{Align, Id, Ui, WidgetText};
use egui_dock::{TabContextMenuOptions, TabContextMenuResponse, TabViewer};

use super::context::PanelContext;
use super::panels::context_menu::{menu_action, render_menu_shell};
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
        tab.is_closeable()
    }

    fn collapseable(&mut self, tab: &mut Self::Tab) -> bool {
        tab.shows_collapse_button()
    }

    /// Use egui_dock's tab [`ScrollArea`] for all panels. Nested scroll areas break scrolling.
    fn scroll_bars(&self, _tab: &Self::Tab) -> [bool; 2] {
        [true, true]
    }

    fn tab_context_menu(
        &mut self,
        ui: &mut Ui,
        tab: &mut Self::Tab,
        surface: egui_dock::SurfaceIndex,
        node: egui_dock::NodeIndex,
        options: TabContextMenuOptions<'_>,
    ) -> TabContextMenuResponse {
        let mut response = TabContextMenuResponse::default();
        let title = tab.title().to_string();
        render_menu_shell(ui, self.ctx.theme, &title, |ui| {
            self.context_menu(ui, tab, surface, node);
            if options.show_eject
                && menu_action(ui, self.ctx.theme, "⤢", options.eject_label)
            {
                response.eject = true;
                ui.close_menu();
            }
            if options.show_close
                && menu_action(ui, self.ctx.theme, "✕", options.close_label)
            {
                response.close = true;
                ui.close_menu();
            }
        });
        response
    }
}

impl KiwiTabViewer<'_> {
    fn context_menu(
        &mut self,
        ui: &mut Ui,
        tab: &mut KiwiTab,
        _surface: egui_dock::SurfaceIndex,
        _node: egui_dock::NodeIndex,
    ) {
        use kiwi_core::events::AppCommand;
        use kiwi_core::github::GitHubLeftPane;

        if *tab != KiwiTab::GitHubIssues {
            return;
        }
        if self.ctx.state.github.left_pane != GitHubLeftPane::Issues || !self.ctx.state.github.auth_ok
        {
            return;
        }
        if menu_action(ui, self.ctx.theme, "＋", "New Issue") {
            let _ = (self.ctx.dispatch)(AppCommand::GitHubIssueCreateOpen);
            ui.close_menu();
        }
        ui.separator();
    }
}

/// egui_dock can leave a stale scroll offset so content sits below the tab top (blank gap).
/// Only correct near the top of the scroll area — at the bottom, content.min.y legitimately
/// sits below clip.min.y and forcing scroll-to-top causes a visible bounce.
fn fix_blank_gap_at_top(ui: &mut Ui) {
    let clip = ui.clip_rect();
    let content = ui.min_rect();
    let scroll_y = (clip.top() - ui.max_rect().top()).max(0.0);
    if scroll_y < 1.0 && content.min.y > clip.min.y + 0.5 {
        ui.scroll_to_rect(content, Some(Align::TOP));
    }
}
