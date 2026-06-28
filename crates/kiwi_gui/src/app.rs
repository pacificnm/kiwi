//! Main eframe application shell (SPEC-021 / SPEC-022).

use std::time::{Duration, Instant};

use kiwi_core::events::{AppCommand, AppEvent};
use kiwi_core::navigation::FocusTarget;
use kiwi_core::state::ReduceView;
use kiwi_core::status_bar::{compute_status_bar, StatusBarSnapshot};
use kiwi_core::workspace::{try_merge_save_gui, try_save_from_reduce_view, GuiWorkspaceSnapshot};

use crate::chrome::{
    palette_keyboard_action, palette_open_shortcut_action, render_about_modal,
    render_command_palette, render_menu_bar, render_reset_layout_modal, render_shortcuts_modal,
    render_status_bar,
};
use crate::dock::{
    collect_github_keyboard, collect_pty_input, collect_search_keyboard, explorer_keyboard_action,
    git_diff_keyboard_action, git_status_keyboard_action,
    global_search_focus_commands, global_search_focus_pressed, navigation_sync_commands,
    preview_keyboard_action, restore_dock, snapshot_from_dock, DockShell, KiwiTab, PanelContext,
    PtySurfaceState, PtyTarget,
};
use crate::navigation_bridge::{navigation_commands_for_dock_tab, sync_dock_from_navigation};
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
    last_shell_interrupt: Option<Instant>,
    shutdown_completed: bool,
    status_bar: StatusBarSnapshot,
}

impl KiwiApp {
    #[must_use]
    pub fn new(
        cc: &eframe::CreationContext<'_>,
        runtime: GuiRuntime,
        gui_snapshot: Option<GuiWorkspaceSnapshot>,
    ) -> Self {
        let mut gui_theme =
            GuiTheme::from_palette(&runtime.state.theme, &runtime.state.config.gui);
        gui_theme.apply_to_context(&cc.egui_ctx);
        let dock = gui_snapshot
            .as_ref()
            .map(restore_dock)
            .map(DockShell::with_state)
            .unwrap_or_default();
        let status_bar = compute_status_bar(&runtime.state);
        let mut app = Self {
            runtime,
            gui_theme,
            dock,
            reset_layout_prompt: false,
            shortcuts_help_open: false,
            about_open: false,
            workspace_last_saved: Instant::now(),
            last_shell_interrupt: None,
            shutdown_completed: false,
            status_bar,
        };
        app.sync_dock();
        app
    }

    fn save_workspace(&mut self) {
        let persist = self.runtime.state.config.workspace.persist;
        let repo_root = self.runtime.state.repo_root.clone();
        {
            let mut view = ReduceView::from_app_state(&mut self.runtime.state);
            try_save_from_reduce_view(&mut view);
        }
        try_merge_save_gui(
            &repo_root,
            persist,
            &snapshot_from_dock(self.dock.dock_state()),
            &mut self.runtime.state.logs,
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
        let nav_before = self.runtime.state.navigation.clone();
        let quit = self.runtime.dispatch_command(command);
        if self.runtime.state.navigation != nav_before {
            self.sync_dock();
        }
        quit
    }

    fn sync_dock(&mut self) {
        let nav = self.runtime.state.navigation.clone();
        let gh_pane = self.runtime.state.github.left_pane;
        sync_dock_from_navigation(&mut self.dock, &nav, gh_pane);
    }

    fn resolve_pty_target(&self, pty_surface: &PtySurfaceState) -> Option<(PtyTarget, bool)> {
        if pty_surface.shell_keyboard_focus {
            return Some((PtyTarget::Shell, true));
        }
        if pty_surface.agent_keyboard_focus {
            return Some((PtyTarget::Agent, true));
        }
        match self.dock.focused_tab() {
            Some(KiwiTab::Terminal) => Some((PtyTarget::Shell, true)),
            Some(KiwiTab::Agent) => Some((PtyTarget::Agent, true)),
            _ if self.runtime.state.navigation.focus == FocusTarget::Shell => {
                Some((PtyTarget::Shell, true))
            }
            _ => None,
        }
    }

    fn handle_pty_input(&mut self, ctx: &egui::Context, pty_surface: &PtySurfaceState) -> bool {
        let Some((target, accept_keyboard)) = self.resolve_pty_target(pty_surface) else {
            return false;
        };

        for command in navigation_sync_commands(
            &self.runtime.state,
            match target {
                PtyTarget::Shell => KiwiTab::Terminal,
                PtyTarget::Agent => KiwiTab::Agent,
            },
        ) {
            let _ = self.runtime.dispatch_command(command);
        }

        let outcome = collect_pty_input(
            ctx,
            &self.runtime.state,
            target,
            &mut self.last_shell_interrupt,
            accept_keyboard,
        );

        if let Some(text) = outcome.copy_to_clipboard {
            ctx.copy_text(text);
        }

        for command in outcome.commands {
            if self.dispatch_command(command) {
                return true;
            }
        }
        false
    }

    fn handle_search_input(&mut self, ctx: &egui::Context) -> bool {
        if self.dock.focused_tab() != Some(KiwiTab::Search) {
            return false;
        }

        for command in collect_search_keyboard(ctx, &self.runtime.state) {
            if self.dispatch_command(command) {
                return true;
            }
        }
        false
    }

    fn handle_github_input(&mut self, ctx: &egui::Context) -> bool {
        let Some(tab) = self.dock.focused_tab() else {
            return false;
        };
        if !matches!(
            tab,
            KiwiTab::GitHubIssues | KiwiTab::Issues | KiwiTab::GitHubPrs
        ) {
            return false;
        }

        for command in collect_github_keyboard(ctx, tab, &self.runtime.state) {
            if self.dispatch_command(command) {
                return true;
            }
        }
        false
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
            match self.dock.focused_tab() {
                Some(KiwiTab::GitHubIssues) | Some(KiwiTab::Issues) | Some(KiwiTab::GitHubPrs) => {
                    let _ = self.dispatch_command(AppCommand::GitHubRefresh);
                }
                Some(KiwiTab::Search) => {
                    if !self.runtime.state.search.query.is_empty() {
                        let _ = self.dispatch_command(AppCommand::SearchExecute);
                    }
                }
                // PTY panels encode F5 as \x1b[15~ via collect_pty_input; skip git refresh
                // so htop/vim/man receive F5 without also triggering a git status reload (#281).
                Some(KiwiTab::Terminal) | Some(KiwiTab::Agent) => {}
                _ => {
                    let _ = self.runtime.dispatch(AppEvent::GitRefreshRequested);
                }
            }
        }

        if global_search_focus_pressed(ctx) {
            for command in global_search_focus_commands() {
                let _ = self.dispatch_command(command);
            }
            return false;
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
                Some(KiwiTab::Preview) => {
                    if let Some(command) = preview_keyboard_action(ctx, &self.runtime.state) {
                        return self.dispatch_command(command);
                    }
                }
                _ => {}
            }
        }

        false
    }

    fn close_window(&mut self, ctx: &egui::Context) {
        self.runtime.shutdown();
        self.save_workspace();
        self.shutdown_completed = true;
        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
    }
}

impl eframe::App for KiwiApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let nav_before = self.runtime.state.navigation.clone();
        let dock_snapshot = snapshot_from_dock(self.dock.dock_state());
        let (should_quit, event_count) = self.runtime.process_pending_events(Some(dock_snapshot));
        if self.runtime.state.navigation != nav_before {
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

        if self.gui_theme.needs_apply {
            self.gui_theme.apply_to_context(ctx);
        }

        if self.runtime.state.dirty {
            self.status_bar = compute_status_bar(&self.runtime.state);
        }

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
        for tab in menu_action.tabs_opened {
            for command in navigation_commands_for_dock_tab(tab) {
                let _ = self.dispatch_command(command);
            }
        }
        if menu_action.quit_requested && self.dispatch_command(AppCommand::Quit) {
            self.close_window(ctx);
            return;
        }

        render_status_bar(ctx, &self.gui_theme, &self.status_bar);

        let mut should_close = false;
        egui::CentralPanel::default().show(ctx, |ui| {
            let mut pending_commands = Vec::new();
            let mut dispatch = |command: AppCommand| {
                pending_commands.push(command);
                false
            };
            let mut pty_surface = PtySurfaceState::default();
            self.dock.render(
                ui,
                PanelContext {
                    state: &mut self.runtime.state,
                    theme: &self.gui_theme,
                    dispatch: &mut dispatch,
                    pty_surface: &mut pty_surface,
                    focused_dock_tab: None,
                },
            );
            for command in pending_commands {
                if self.dispatch_command(command) {
                    should_close = true;
                    return;
                }
            }
            self.runtime.sync_pty_resize_from_viewport();
            if self.handle_github_input(ctx) {
                should_close = true;
                return;
            }
            if self.handle_search_input(ctx) {
                should_close = true;
                return;
            }
            if self.handle_pty_input(ctx, &pty_surface) {
                should_close = true;
            }
        });
        if should_close {
            self.close_window(ctx);
            return;
        }

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

        if self.runtime.search_debounce_pending() {
            ctx.request_repaint();
        }
        if self.runtime.poll_search_debounce() {
            let _ = self.runtime.dispatch_command(AppCommand::SearchExecute);
        }

        if event_count > 0 || self.runtime.state.dirty {
            ctx.request_repaint();
        }
        self.runtime.state.mark_clean();
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        if !self.shutdown_completed {
            self.runtime.shutdown();
            self.save_workspace();
        }
    }
}
