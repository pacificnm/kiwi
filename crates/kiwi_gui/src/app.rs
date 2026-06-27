//! Main eframe application shell (SPEC-021 / SPEC-022).

use std::time::{Duration, Instant};

use kiwi_core::events::{AppCommand, AppEvent};
use kiwi_core::state::ReduceView;
use kiwi_core::workspace::{try_merge_save_gui, try_save_from_reduce_view, GuiWorkspaceSnapshot};

use crate::chrome::{
    palette_keyboard_action, palette_open_shortcut_action, render_about_modal,
    render_command_palette, render_menu_bar, render_reset_layout_modal, render_shortcuts_modal,
    render_status_bar,
};
use crate::dock::{
    explorer_keyboard_action, git_diff_keyboard_action, git_status_keyboard_action, restore_dock,
    snapshot_from_dock, DockShell, KiwiTab, PanelContext,
};
use crate::navigation_bridge::sync_dock_from_navigation;
use crate::runtime::GuiRuntime;
use crate::theme::GuiTheme;

/// GUI shell with egui_dock tab panels.
pub struct KiwiApp {
    runtime: GuiRuntime,
    gui_theme: GuiTheme,
    dock: DockShell,
    reset_layout_prompt: bool,
    shortcuts_help_open: bool,
    about_open: bool,
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
        let mut app = Self {
            runtime,
            gui_theme,
            dock,
            reset_layout_prompt: false,
            shortcuts_help_open: false,
            about_open: false,
            workspace_last_saved: Instant::now(),
        };
        app.sync_dock();
        app
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

    fn dispatch_command(&mut self, command: AppCommand) -> bool {
        let quit = self.runtime.dispatch_command(command);
        self.sync_dock();
        quit
    }

    fn sync_dock(&mut self) {
        let nav = self.runtime.state.navigation.clone();
        let gh_pane = self.runtime.state.github.left_pane;
        sync_dock_from_navigation(&mut self.dock, &nav, gh_pane);
    }

    fn handle_input_shortcuts(&mut self, ctx: &egui::Context) -> bool {
        if self.runtime.state.palette.open {
            let prompt_mode = self.runtime.state.palette.prompt.is_some();
            let input_empty = self.runtime.state.palette.input.is_empty();
            if let Some(command) = palette_keyboard_action(ctx, prompt_mode, input_empty) {
                return self.dispatch_command(command);
            }
        } else if let Some(command) = palette_open_shortcut_action(ctx) {
            let _ = self.dispatch_command(command);
            return false;
        }

        if ctx.input(|input| input.key_pressed(egui::Key::Q) && input.modifiers.command) {
            return self.dispatch_command(AppCommand::Quit);
        }

        if ctx.input(|input| input.key_pressed(egui::Key::F5)) {
            return self.runtime.dispatch(AppEvent::GitRefreshRequested);
        }

        if !self.runtime.state.palette.open {
            match self.dock.focused_tab() {
                Some(KiwiTab::Explorer) => {
                    if let Some(command) = explorer_keyboard_action(ctx, &self.runtime.state) {
                        return self.dispatch_command(command);
                    }
                }
                Some(KiwiTab::GitStatus) => {
                    if let Some(command) = git_status_keyboard_action(ctx, &self.runtime.state) {
                        return self.dispatch_command(command);
                    }
                }
                Some(KiwiTab::GitDiff) => {
                    if let Some(command) = git_diff_keyboard_action(ctx, &self.runtime.state) {
                        return self.dispatch_command(command);
                    }
                }
                _ => {}
            }
        }

        false
    }

    fn close_window(&mut self, ctx: &egui::Context) {
        self.save_workspace();
        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
    }
}

impl eframe::App for KiwiApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let (should_quit, event_count) = self.runtime.process_pending_events();
        if event_count > 0 {
            self.sync_dock();
        }
        if should_quit {
            self.close_window(ctx);
            return;
        }

        if self.handle_input_shortcuts(ctx) {
            self.close_window(ctx);
            return;
        }

        self.gui_theme.apply_to_context(ctx);

        let menu_action = render_menu_bar(ctx, &mut self.dock);
        if menu_action.reset_layout_requested {
            self.reset_layout_prompt = true;
        }
        if menu_action.command_palette_requested {
            let _ = self.dispatch_command(AppCommand::PaletteOpen);
        }
        if menu_action.git_refresh_requested {
            let _ = self.runtime.dispatch(AppEvent::GitRefreshRequested);
        }
        if menu_action.shortcuts_help_requested {
            self.shortcuts_help_open = true;
        }
        if menu_action.about_requested {
            self.about_open = true;
        }
        if menu_action.quit_requested && self.dispatch_command(AppCommand::Quit) {
            self.close_window(ctx);
            return;
        }

        render_status_bar(ctx, &self.gui_theme, &self.runtime.state);

        egui::CentralPanel::default().show(ctx, |ui| {
            let mut pending_commands = Vec::new();
            let mut dispatch = |command: AppCommand| {
                pending_commands.push(command);
                false
            };
            self.dock.render(
                ui,
                PanelContext {
                    state: &mut self.runtime.state,
                    theme: &self.gui_theme,
                    dispatch: &mut dispatch,
                },
            );
            for command in pending_commands {
                if self.dispatch_command(command) {
                    self.close_window(ctx);
                    return;
                }
            }
        });

        render_reset_layout_modal(ctx, &mut self.reset_layout_prompt, &mut self.dock);
        render_shortcuts_modal(ctx, &mut self.shortcuts_help_open);
        render_about_modal(ctx, &mut self.about_open);

        if let Some(command) = render_command_palette(ctx, &self.gui_theme, &mut self.runtime.state)
        {
            if self.dispatch_command(command) {
                self.close_window(ctx);
                return;
            }
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
