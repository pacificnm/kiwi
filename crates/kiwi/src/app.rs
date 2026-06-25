use std::io::stdout;
use std::time::Duration;

use crossterm::event::{
    self, Event, KeyCode, KeyEventKind, KeyModifiers, MouseButton, MouseEvent, MouseEventKind,
};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;

use crate::bootstrap::StartupContext;
use crate::navigation::map_key;
use crate::state::{reduce, AppCommand, AppEvent, AppState, EventChannel, SideEffect};
use crate::ui::{draw_frame, map_tab_click, mouse_interactions_enabled};

pub struct App {
    state: AppState,
    terminal: crate::terminal::TerminalGuard,
    events: EventChannel,
}

impl App {
    #[must_use]
    pub fn new(context: StartupContext) -> Self {
        let StartupContext {
            repo_root,
            is_git_repo,
            config,
            theme,
            layout,
            terminal,
        } = context;

        Self {
            state: AppState::from_startup(repo_root, is_git_repo, config, theme, layout),
            terminal,
            events: EventChannel::new(),
        }
    }

    #[must_use]
    #[cfg(test)]
    pub fn state(&self) -> &AppState {
        &self.state
    }

    #[must_use]
    #[cfg(test)]
    pub fn state_mut(&mut self) -> &mut AppState {
        &mut self.state
    }

    #[must_use]
    #[cfg(test)]
    pub fn event_sender(&self) -> crate::state::EventSender {
        self.events.sender()
    }

    pub fn run(&mut self) {
        let mut terminal =
            Terminal::new(CrosstermBackend::new(stdout())).expect("create ratatui terminal");

        loop {
            if self.process_pending_events() {
                break;
            }

            if self.state.dirty {
                terminal
                    .draw(|frame| draw_frame(frame, &self.state))
                    .expect("draw frame");
                self.state.mark_clean();
            }

            if event::poll(Duration::from_millis(100)).expect("poll terminal events")
                && self.handle_terminal_event(event::read().expect("read terminal event"))
            {
                break;
            }
        }

        crate::shutdown::cleanup_terminal(&mut self.terminal);
    }

    fn process_pending_events(&mut self) -> bool {
        let pending = self.events.drain_coalesced();
        for event in pending {
            if self.dispatch(event) {
                return true;
            }
        }
        false
    }

    fn dispatch(&mut self, event: AppEvent) -> bool {
        let effects = reduce(&mut self.state, event);
        self.execute_effects(effects)
    }

    fn execute_effects(&mut self, effects: Vec<SideEffect>) -> bool {
        for effect in effects {
            match effect {
                SideEffect::Quit => return true,
                SideEffect::SpawnGitRefresh => {
                    // Services will enqueue GitStatusUpdated events in later milestones.
                }
                SideEffect::SaveWorkspace | SideEffect::LaunchEditor(_) => {}
            }
        }
        false
    }

    fn handle_terminal_event(&mut self, event: Event) -> bool {
        match event {
            Event::Resize(width, height) => {
                self.dispatch(AppEvent::TerminalResize { width, height })
            }
            Event::Key(key) => {
                if key.kind != KeyEventKind::Press {
                    return false;
                }
                self.handle_key(key)
            }
            Event::Mouse(mouse) => self.handle_mouse(mouse),
            _ => false,
        }
    }

    fn handle_mouse(&mut self, mouse: MouseEvent) -> bool {
        if !mouse_interactions_enabled(&self.state.config.mouse) {
            return false;
        }

        if mouse.modifiers.contains(KeyModifiers::SHIFT) {
            return false;
        }

        if !matches!(mouse.kind, MouseEventKind::Down(MouseButton::Left)) {
            return false;
        }

        if let Some(command) = map_tab_click(&self.state, mouse.column, mouse.row) {
            return self.dispatch(AppEvent::Command(AppCommand::Navigation(command)));
        }

        false
    }

    fn handle_key(&mut self, key: crossterm::event::KeyEvent) -> bool {
        let quit = matches!(key.code, KeyCode::Char('q'))
            || (key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL));

        if quit {
            return self.dispatch(AppEvent::Command(AppCommand::Quit));
        }

        if let Some(command) = map_key(key, self.state.navigation.focus) {
            return self.dispatch(AppEvent::Command(AppCommand::Navigation(command)));
        }

        false
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::bootstrap::StartupContext;
    use crate::config::ResolvedConfig;
    use crate::layout::compute_layout;
    use crate::state::AppEvent;
    use crate::terminal::TerminalGuard;
    use crate::theme::capabilities::TerminalCapabilities;
    use crate::theme::loader::load_theme_with_capabilities;
    use crate::theme::ThemePalette;

    use super::App;

    fn test_palette() -> ThemePalette {
        load_theme_with_capabilities(
            &ResolvedConfig::default().theme,
            TerminalCapabilities::TrueColor,
        )
        .expect("load default theme")
    }

    fn test_context() -> StartupContext {
        StartupContext {
            repo_root: PathBuf::from("."),
            is_git_repo: false,
            config: ResolvedConfig::default(),
            theme: test_palette(),
            layout: compute_layout(120, 40, 30).expect("layout"),
            terminal: TerminalGuard::inactive(),
        }
    }

    #[test]
    fn app_constructs_without_panic() {
        let _app = App::new(test_context());
    }

    #[test]
    fn app_drains_channel_events_into_state() {
        let mut app = App::new(test_context());
        app.event_sender()
            .send(AppEvent::GitRefreshRequested)
            .expect("send");
        app.process_pending_events();
        assert!(app.state().dirty);
    }

    #[test]
    fn shell_focus_ignores_main_tab_shortcuts() {
        use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers};

        use crate::layout::FocusTarget;
        use crate::navigation::NavCommand;
        use crate::state::AppCommand;

        let mut app = App::new(test_context());
        app.event_sender()
            .send(AppEvent::Command(AppCommand::Navigation(
                NavCommand::SetFocus(FocusTarget::Shell),
            )))
            .expect("send");
        app.process_pending_events();
        let before = app.state().navigation.main_tab;

        let key = KeyEvent {
            code: KeyCode::Char('3'),
            modifiers: KeyModifiers::empty(),
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        };
        assert!(!app.dispatch_key(key));
        assert_eq!(app.state().navigation.main_tab, before);
    }

    #[test]
    fn mouse_click_on_main_tab_updates_selection() {
        use crossterm::event::{KeyModifiers, MouseButton, MouseEvent, MouseEventKind};

        let mut app = App::new(test_context());
        let main_tabs = app.state().layout.rects.main_tabs;
        let mouse = MouseEvent {
            kind: MouseEventKind::Down(MouseButton::Left),
            column: main_tabs.x + 8,
            row: main_tabs.y,
            modifiers: KeyModifiers::empty(),
        };

        assert!(!app.dispatch_mouse(mouse));
        assert_eq!(
            app.state().navigation.main_tab,
            crate::navigation::MainTab::Issues
        );
    }

    #[test]
    fn mouse_click_ignored_when_mouse_disabled() {
        use crossterm::event::{KeyModifiers, MouseButton, MouseEvent, MouseEventKind};

        let mut app = App::new(test_context());
        app.state_mut().config.mouse.enabled = false;
        let before = app.state().navigation.left_tab;
        let left_tabs = app.state().layout.rects.left_tabs;
        let mouse = MouseEvent {
            kind: MouseEventKind::Down(MouseButton::Left),
            column: left_tabs.x,
            row: left_tabs.y,
            modifiers: KeyModifiers::empty(),
        };

        assert!(!app.dispatch_mouse(mouse));
        assert_eq!(app.state().navigation.left_tab, before);
    }
}

#[cfg(test)]
impl App {
    fn dispatch_key(&mut self, key: crossterm::event::KeyEvent) -> bool {
        self.handle_key(key)
    }

    fn dispatch_mouse(&mut self, mouse: MouseEvent) -> bool {
        self.handle_mouse(mouse)
    }
}
