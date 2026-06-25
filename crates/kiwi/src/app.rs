use std::io::stdout;
use std::time::{Duration, Instant};

use crossterm::event::{
    self, Event, KeyCode, KeyEventKind, KeyModifiers, MouseButton, MouseEvent, MouseEventKind,
};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;

use crate::agent::{AgentOutputReader, AgentSession};
use crate::bootstrap::StartupContext;
use crate::layout::{agent_pty_size, shell_pty_size, FocusTarget};
use crate::navigation::{map_key, MainTab};
use crate::shell::{encode_key, ShellOutputReader, ShellSession};
use crate::shutdown;
use crate::state::{
    agent_spawn_effects_if_needed, reduce, AppCommand, AppEvent, AppState, EventChannel, SideEffect,
};
use crate::ui::{draw_frame, map_mouse_click, mouse_interactions_enabled};

const SHELL_FORCE_QUIT_WINDOW: Duration = Duration::from_millis(500);

pub struct App {
    state: AppState,
    terminal: crate::terminal::TerminalGuard,
    events: EventChannel,
    shell: Option<ShellSession>,
    shell_io: Option<ShellOutputReader>,
    agent: Option<AgentSession>,
    agent_io: Option<AgentOutputReader>,
    last_shell_interrupt: Option<Instant>,
    last_agent_interrupt: Option<Instant>,
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

        let mut state =
            AppState::from_startup(repo_root.clone(), is_git_repo, config, theme, layout);
        let events = EventChannel::new();
        let (cols, rows) = shell_pty_size(&state.layout.rects);
        let shell = match ShellSession::spawn(&repo_root, &state.config.shell, cols, rows) {
            Ok(session) => {
                state.shell.apply_spawn(
                    &session.spec.command,
                    &session.spec.shell_name,
                    session.pid(),
                    session.cols,
                    session.rows,
                );
                Some(session)
            }
            Err(err) => {
                state.shell.apply_spawn_error(err.to_string());
                None
            }
        };
        let shell_io = shell
            .as_ref()
            .and_then(|session| session.try_clone_reader().ok())
            .map(|reader| ShellOutputReader::spawn(reader, events.sender()));

        let mut app = Self {
            state,
            terminal,
            events,
            shell,
            shell_io,
            agent: None,
            agent_io: None,
            last_shell_interrupt: None,
            last_agent_interrupt: None,
        };
        let spawn_effects = agent_spawn_effects_if_needed(&mut app.state);
        app.execute_effects(spawn_effects);
        app
    }

    fn spawn_agent(&mut self) {
        if self.state.agent.spawned {
            return;
        }

        let (cols, rows) = agent_pty_size(&self.state.layout.rects);
        match AgentSession::spawn(&self.state.repo_root, &self.state.config.agent, cols, rows) {
            Ok(session) => {
                self.state.agent.apply_spawn(
                    &session.spec.command,
                    &session.spec.agent_name,
                    session.pid(),
                    session.cols,
                    session.rows,
                );
                self.agent_io = session
                    .try_clone_reader()
                    .ok()
                    .map(|reader| AgentOutputReader::spawn(reader, self.events.sender()));
                self.agent = Some(session);
            }
            Err(err) => {
                self.state.agent.apply_spawn_error(err.to_string());
            }
        }
        self.state.dirty = true;
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
        shutdown::install_signal_handlers();

        let mut terminal =
            Terminal::new(CrosstermBackend::new(stdout())).expect("create ratatui terminal");

        loop {
            if shutdown::shutdown_requested() {
                break;
            }

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

        self.shutdown(&mut terminal);
    }

    fn shutdown(&mut self, terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>) {
        if let Some(reader) = self.shell_io.take() {
            reader.abandon();
        }
        if let Some(reader) = self.agent_io.take() {
            reader.abandon();
        }

        let _ = terminal.clear();
        let _ = terminal.show_cursor();
        shutdown::cleanup_terminal(&mut self.terminal);

        if let Some(mut shell) = self.shell.take() {
            shell.shutdown();
        }
        if let Some(mut agent) = self.agent.take() {
            agent.shutdown();
        }
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
                SideEffect::SpawnAgent => {
                    self.spawn_agent();
                }
                SideEffect::WriteShell(data) => {
                    if let Some(shell) = self.shell.as_mut() {
                        let _ = shell.write(&data);
                    }
                }
                SideEffect::WriteAgent(data) => {
                    if let Some(agent) = self.agent.as_mut() {
                        let _ = agent.write(&data);
                    }
                }
                SideEffect::ResizeShell { cols, rows } => {
                    if let Some(shell) = self.shell.as_mut() {
                        let _ = shell.resize(cols, rows);
                    }
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

        for command in map_mouse_click(&self.state, mouse.column, mouse.row) {
            if self.dispatch(AppEvent::Command(AppCommand::Navigation(command))) {
                return true;
            }
        }

        false
    }

    fn handle_key(&mut self, key: crossterm::event::KeyEvent) -> bool {
        if self.is_force_quit(key) {
            return self.dispatch(AppEvent::Command(AppCommand::Quit));
        }

        if self.is_focus_cycle_key(key) {
            let command = if key.modifiers.contains(KeyModifiers::SHIFT) {
                crate::navigation::NavCommand::PreviousFocus
            } else {
                crate::navigation::NavCommand::NextFocus
            };
            return self.dispatch(AppEvent::Command(AppCommand::Navigation(command)));
        }

        if self.agent_input_active() {
            return self.handle_agent_key(key);
        }

        if self.state.navigation.focus == FocusTarget::Shell && self.state.shell.running {
            return self.handle_shell_key(key);
        }

        if self.is_global_quit(key) {
            return self.dispatch(AppEvent::Command(AppCommand::Quit));
        }

        if let Some(command) = map_key(key, self.state.navigation.focus) {
            return self.dispatch(AppEvent::Command(AppCommand::Navigation(command)));
        }

        false
    }

    fn is_focus_cycle_key(&self, key: crossterm::event::KeyEvent) -> bool {
        matches!(key.code, KeyCode::Tab)
    }

    fn is_force_quit(&self, key: crossterm::event::KeyEvent) -> bool {
        key.modifiers.contains(KeyModifiers::CONTROL)
            && matches!(key.code, KeyCode::Char('q' | 'Q'))
    }

    fn is_global_quit(&self, key: crossterm::event::KeyEvent) -> bool {
        matches!(key.code, KeyCode::Char('q'))
            || (key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL))
    }

    fn agent_input_active(&self) -> bool {
        self.state.navigation.focus == FocusTarget::Main
            && self.state.navigation.main_tab == MainTab::Agent
            && self.state.agent.running
    }

    fn handle_agent_key(&mut self, key: crossterm::event::KeyEvent) -> bool {
        if key.modifiers.contains(KeyModifiers::CONTROL)
            && matches!(key.code, KeyCode::Char('c' | 'C'))
        {
            let now = Instant::now();
            if self
                .last_agent_interrupt
                .is_some_and(|earlier| now.duration_since(earlier) <= SHELL_FORCE_QUIT_WINDOW)
            {
                return self.dispatch(AppEvent::Command(AppCommand::Quit));
            }
            self.last_agent_interrupt = Some(now);
        } else {
            self.last_agent_interrupt = None;
        }

        match key.code {
            KeyCode::PageUp => self.dispatch(AppEvent::Command(AppCommand::AgentScroll(-1))),
            KeyCode::PageDown => self.dispatch(AppEvent::Command(AppCommand::AgentScroll(1))),
            _ => {
                if let Some(bytes) = encode_key(key) {
                    self.dispatch(AppEvent::Command(AppCommand::AgentWrite(bytes)))
                } else {
                    false
                }
            }
        }
    }

    fn handle_shell_key(&mut self, key: crossterm::event::KeyEvent) -> bool {
        if key.modifiers.contains(KeyModifiers::CONTROL)
            && matches!(key.code, KeyCode::Char('c' | 'C'))
        {
            let now = Instant::now();
            if self
                .last_shell_interrupt
                .is_some_and(|earlier| now.duration_since(earlier) <= SHELL_FORCE_QUIT_WINDOW)
            {
                return self.dispatch(AppEvent::Command(AppCommand::Quit));
            }
            self.last_shell_interrupt = Some(now);
        } else {
            self.last_shell_interrupt = None;
        }

        match key.code {
            KeyCode::PageUp => self.dispatch(AppEvent::Command(AppCommand::ShellScroll(-1))),
            KeyCode::PageDown => self.dispatch(AppEvent::Command(AppCommand::ShellScroll(1))),
            _ => {
                if let Some(bytes) = encode_key(key) {
                    self.dispatch(AppEvent::Command(AppCommand::ShellWrite(bytes)))
                } else {
                    false
                }
            }
        }
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
    fn app_spawns_agent_when_agent_tab_is_active_at_startup() {
        let mut context = test_context();
        if std::path::Path::new("/bin/bash").exists()
            || std::path::Path::new("/usr/bin/bash").exists()
        {
            context.config.agent.command = "bash".to_string();
            let app = App::new(context);
            assert!(app.state().agent.spawned);
            assert!(app.state().agent.running);
            assert!(app.state().agent.child_pid.is_some());
        }
    }

    #[test]
    fn app_spawns_shell_at_startup() {
        let app = App::new(test_context());
        assert!(app.state().shell.running);
        assert!(app.state().shell.child_pid.is_some());
        assert!(!app.state().shell.shell_name.is_empty());
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
    fn tab_cycles_focus_when_agent_input_is_active() {
        use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers};

        use crate::layout::FocusTarget;
        use crate::navigation::{MainTab, NavCommand};
        use crate::state::AppCommand;

        let mut app = App::new(test_context());
        app.state_mut().navigation.main_tab = MainTab::Agent;
        app.state_mut().agent.running = true;
        app.event_sender()
            .send(AppEvent::Command(AppCommand::Navigation(
                NavCommand::SetFocus(FocusTarget::Main),
            )))
            .expect("send");
        app.process_pending_events();

        let tab = KeyEvent {
            code: KeyCode::Tab,
            modifiers: KeyModifiers::empty(),
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        };
        assert!(!app.dispatch_key(tab));
        assert_eq!(app.state().navigation.focus, FocusTarget::CommandPalette);
    }

    #[test]
    fn agent_focus_forwards_keys_instead_of_quitting() {
        use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers};

        use crate::layout::FocusTarget;
        use crate::navigation::{MainTab, NavCommand};
        use crate::state::AppCommand;

        let mut app = App::new(test_context());
        app.state_mut().navigation.main_tab = MainTab::Agent;
        app.state_mut().agent.running = true;
        app.event_sender()
            .send(AppEvent::Command(AppCommand::Navigation(
                NavCommand::SetFocus(FocusTarget::Main),
            )))
            .expect("send");
        app.process_pending_events();

        let key = KeyEvent {
            code: KeyCode::Char('q'),
            modifiers: KeyModifiers::empty(),
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        };
        assert!(!app.dispatch_key(key));

        let ctrl_c = KeyEvent {
            code: KeyCode::Char('c'),
            modifiers: KeyModifiers::CONTROL,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        };
        assert!(!app.dispatch_key(ctrl_c));
        assert!(app.dispatch_key(ctrl_c));
    }

    #[test]
    fn ctrl_q_quits_from_shell_focus() {
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

        let key = KeyEvent {
            code: KeyCode::Char('q'),
            modifiers: KeyModifiers::CONTROL,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        };
        assert!(app.dispatch_key(key));
    }

    #[test]
    fn mouse_click_on_main_tab_returns_focus_to_main() {
        use crossterm::event::{KeyModifiers, MouseButton, MouseEvent, MouseEventKind};

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

        let main_tabs = app.state().layout.rects.main_tabs;
        let mouse = MouseEvent {
            kind: MouseEventKind::Down(MouseButton::Left),
            column: main_tabs.x + 8,
            row: main_tabs.y,
            modifiers: KeyModifiers::empty(),
        };

        assert!(!app.dispatch_mouse(mouse));
        assert_eq!(app.state().navigation.focus, FocusTarget::Main);
    }

    #[test]
    fn shell_focus_forwards_keys_instead_of_quitting() {
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

        let key = KeyEvent {
            code: KeyCode::Char('q'),
            modifiers: KeyModifiers::empty(),
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        };
        assert!(!app.dispatch_key(key));

        let ctrl_c = KeyEvent {
            code: KeyCode::Char('c'),
            modifiers: KeyModifiers::CONTROL,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        };
        assert!(!app.dispatch_key(ctrl_c));
        assert!(app.dispatch_key(ctrl_c));
    }

    #[test]
    fn mouse_click_on_shell_pane_focuses_shell() {
        use crossterm::event::{KeyModifiers, MouseButton, MouseEvent, MouseEventKind};

        use crate::layout::FocusTarget;

        let mut app = App::new(test_context());
        let shell = app.state().layout.rects.shell;
        let mouse = MouseEvent {
            kind: MouseEventKind::Down(MouseButton::Left),
            column: shell.x + 2,
            row: shell.y + 2,
            modifiers: KeyModifiers::empty(),
        };

        assert!(!app.dispatch_mouse(mouse));
        assert_eq!(app.state().navigation.focus, FocusTarget::Shell);
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
