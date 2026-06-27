//! Main eframe application shell (SPEC-021 / SPEC-022).

use std::time::{Duration, Instant};

use kiwi_core::state::ReduceView;
use kiwi_core::workspace::{try_merge_save_gui, try_save_from_reduce_view, GuiWorkspaceSnapshot};

use crate::chrome::{render_menu_bar, render_reset_layout_modal, render_status_bar};
use crate::dock::{restore_dock, snapshot_from_dock, DockShell, PanelContext};
use crate::runtime::GuiRuntime;
use crate::theme::GuiTheme;

/// GUI shell with egui_dock tab panels.
pub struct KiwiApp {
    runtime: GuiRuntime,
    gui_theme: GuiTheme,
    dock: DockShell,
    reset_layout_prompt: bool,
    workspace_last_saved: Instant,
}

impl KiwiApp {
    #[must_use]
    pub fn new(
        _cc: &eframe::CreationContext<'_>,
        runtime: GuiRuntime,
        gui_snapshot: Option<GuiWorkspaceSnapshot>,
    ) -> Self {
        let gui_theme = GuiTheme::from_palette(&runtime.state.theme, &runtime.state.config.gui);
        let dock = gui_snapshot
            .as_ref()
            .map(restore_dock)
            .map(DockShell::with_state)
            .unwrap_or_default();
        Self {
            runtime,
            gui_theme,
            dock,
            reset_layout_prompt: false,
            workspace_last_saved: Instant::now(),
        }
    }

    fn save_workspace(&mut self) {
        let persist = self.runtime.state.config.workspace.persist;
        let repo_root = self.runtime.state.repo_root.clone();
        try_save_from_reduce_view(&ReduceView::from_app_state(&mut self.runtime.state));
        try_merge_save_gui(
            &repo_root,
            persist,
            &snapshot_from_dock(self.dock.dock_state()),
        );
        self.workspace_last_saved = Instant::now();
    }

    fn poll_workspace_save(&mut self) {
        if !self.runtime.state.config.workspace.persist {
            return;
        }
        let interval = Duration::from_secs(self.runtime.state.config.workspace.save_interval_secs);
        if self.workspace_last_saved.elapsed() >= interval {
            self.save_workspace();
        }
    }
}

impl eframe::App for KiwiApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let (should_quit, event_count) = self.runtime.process_pending_events();
        if should_quit {
            self.save_workspace();
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            return;
        }

        self.gui_theme.apply_to_context(ctx);

        let menu_action = render_menu_bar(ctx, &mut self.dock);
        if menu_action.reset_layout_requested {
            self.reset_layout_prompt = true;
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            self.dock.render(
                ui,
                PanelContext {
                    state: &self.runtime.state,
                    theme: &self.gui_theme,
                },
            );
        });

        render_status_bar(ctx, &self.gui_theme, &self.runtime.state);
        render_reset_layout_modal(ctx, &mut self.reset_layout_prompt, &mut self.dock);

        if ctx.input(|input| input.key_pressed(egui::Key::Q) && input.modifiers.command) {
            self.save_workspace();
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            return;
        }

        self.poll_workspace_save();

        if event_count > 0 || self.runtime.state.dirty {
            ctx.request_repaint();
        }
        self.runtime.state.mark_clean();
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        self.save_workspace();
    }
}
