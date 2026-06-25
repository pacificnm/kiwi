use std::io::stdout;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use crossterm::event::{
    self, Event, KeyCode, KeyEventKind, KeyModifiers, MouseButton, MouseEvent, MouseEventKind,
};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;

use crate::agent::{AgentOutputReader, AgentSession};
use crate::bootstrap::StartupContext;
use crate::clipboard::{
    clipboard_op_from_key, clipboard_shortcut_allowed, ClipboardOp, ClipboardService,
};
use crate::diff::spawn_file_diff_load;
use crate::editor::{
    launch_gui_editor, prepare_editor_launch, resolve_editor_target, run_terminal_editor,
    EditorLaunchMode,
};
use crate::file_tree::spawn_directory_load;
use crate::git::spawn_git_refresh;
use crate::github::{
    spawn_github_auth_check, spawn_github_issue_comment, spawn_github_issue_create_branch,
    spawn_github_issue_detail_load, spawn_github_issue_label_apply, spawn_github_issue_list_load,
    spawn_github_open_browser, spawn_github_pr_create, spawn_github_pr_detail_load,
    spawn_github_pr_list_load, spawn_github_repo_labels_load,
};
use crate::layout::{agent_pty_size, shell_pty_size, FocusTarget};
use crate::navigation::{map_key, LeftNavTab, MainTab, NavCommand};
use crate::preview::spawn_preview_load;
use crate::search::{spawn_search, DebounceTimer, SearchCancelHandle, SearchJob, SearchMode};
use crate::selection::{hit_test_text, SelectionPane};
use crate::shell::{encode_key, ShellOutputReader, ShellSession};
use crate::shutdown;
use crate::state::{
    agent_spawn_effects_if_needed, reduce, AppCommand, AppEvent, AppState, EventChannel, SideEffect,
};
use crate::ui::git_interaction_at;
use crate::ui::{
    draw_frame, file_tree_interaction_at, github_issue_interaction_at, github_pr_interaction_at,
    map_mouse_click, mouse_interactions_enabled, palette_match_at, search_interaction_at,
    DoubleClickTarget, DoubleClickTracker, FileTreeMouseAction,
};
use crate::watcher::RepoWatcher;
use crate::workspace::{load_palette_history, save_palette_history};

const SHELL_FORCE_QUIT_WINDOW: Duration = Duration::from_millis(500);

struct PendingEditorLaunch {
    path: PathBuf,
    line: Option<u32>,
}

pub struct App {
    state: AppState,
    terminal: crate::terminal::TerminalGuard,
    events: EventChannel,
    shell: Option<ShellSession>,
    shell_io: Option<ShellOutputReader>,
    agent: Option<AgentSession>,
    agent_io: Option<AgentOutputReader>,
    last_shell_interrupt: Option<Instant>,
    search_debounce: DebounceTimer,
    search_cancel: SearchCancelHandle,
    search_live_generation: Arc<AtomicU64>,
    pending_editor_launch: Option<PendingEditorLaunch>,
    clipboard: ClipboardService,
    double_click: DoubleClickTracker,
    _repo_watcher: Option<RepoWatcher>,
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
        if state.config.workspace.persist {
            if let Some(history) = load_palette_history(&repo_root) {
                state.palette.history = history;
            }
        }
        let events = EventChannel::new();
        let (repo_watcher, watcher_error) = match RepoWatcher::spawn(
            repo_root.clone(),
            state.config.watcher.debounce_ms,
            events.sender(),
        ) {
            Ok(watcher) => (Some(watcher), None),
            Err(err) => {
                eprintln!("file watcher disabled: {err}");
                (None, Some(err))
            }
        };
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
            search_debounce: DebounceTimer::default(),
            search_cancel: SearchCancelHandle::default(),
            search_live_generation: Arc::new(AtomicU64::new(0)),
            pending_editor_launch: None,
            clipboard: ClipboardService::new(),
            double_click: DoubleClickTracker::default(),
            _repo_watcher: repo_watcher,
        };
        let spawn_effects = agent_spawn_effects_if_needed(&mut app.state);
        app.execute_effects(spawn_effects);
        if let Some(err) = watcher_error {
            app.state.logs.push_info(format!(
                "File watcher disabled: {err}. Use Git: Refresh Status or R in the Git tab."
            ));
            app.state
                .notifications
                .show_toast("File watcher disabled — use manual git refresh");
            app.state.dirty = true;
        }
        if app.state.workspace_meta.is_git_repo {
            app.dispatch(AppEvent::GitRefreshRequested);
        }
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

    fn restart_agent(&mut self) {
        if self.state.navigation.main_tab != MainTab::Agent {
            return;
        }

        if let Some(reader) = self.agent_io.take() {
            reader.abandon();
        }
        if let Some(mut agent) = self.agent.take() {
            agent.shutdown();
        }

        self.state.agent.prepare_restart();
        self.spawn_agent();
    }

    fn poll_agent_exit(&mut self) {
        if !self.state.agent.running {
            return;
        }

        let exit_code = {
            let Some(agent) = self.agent.as_mut() else {
                return;
            };
            agent.poll_exit()
        };

        let Some(code) = exit_code else {
            return;
        };

        if let Some(reader) = self.agent_io.take() {
            reader.abandon();
        }
        self.agent.take();

        self.dispatch(AppEvent::AgentExited(code));
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

            self.poll_agent_exit();

            if self.process_pending_events() {
                break;
            }

            self.flush_pending_editor_launch(&mut terminal);

            self.poll_search_debounce();

            if self.state.dirty {
                terminal
                    .draw(|frame| draw_frame(frame, &self.state))
                    .expect("draw frame");
                self.state.mark_clean();
            }

            let poll_timeout = self
                .search_debounce
                .remaining()
                .unwrap_or(Duration::from_millis(100))
                .min(Duration::from_millis(100));

            if event::poll(poll_timeout).expect("poll terminal events")
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

        if self.state.config.workspace.persist {
            let _ = save_palette_history(&self.state.repo_root, &self.state.palette.history);
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
        let quit = self.execute_effects(effects);
        self.sync_search_debounce();
        quit
    }

    fn sync_search_debounce(&mut self) {
        self.search_live_generation
            .store(self.state.search.generation, Ordering::Relaxed);

        if self.state.search.debounce_scheduled {
            let debounce = Duration::from_millis(self.state.config.search.debounce_ms);
            self.search_debounce.schedule(debounce);
        }
    }

    fn poll_search_debounce(&mut self) {
        if self.search_debounce.poll_ready() {
            self.dispatch(AppEvent::Command(AppCommand::SearchExecute));
        }
    }

    fn flush_pending_editor_launch(
        &mut self,
        tui: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
    ) {
        let Some(pending) = self.pending_editor_launch.take() else {
            return;
        };

        let repo_root = self.state.repo_root.clone();
        let settings = self.state.config.editor.clone();

        let prepared =
            match prepare_editor_launch(&repo_root, &settings, &pending.path, pending.line) {
                Ok(prepared) => prepared,
                Err(err) => {
                    self.dispatch(AppEvent::EditorLaunchFailed {
                        path: pending.path,
                        error: err.user_message(),
                        show_modal: err.is_command_not_found(),
                    });
                    return;
                }
            };

        let launch_result = match prepared.mode {
            EditorLaunchMode::Gui => launch_gui_editor(&prepared),
            EditorLaunchMode::Terminal => {
                let _ = tui.clear();
                if let Err(err) = self.terminal.suspend() {
                    self.dispatch(AppEvent::EditorLaunchFailed {
                        path: prepared.path.clone(),
                        error: err.to_string(),
                        show_modal: false,
                    });
                    return;
                }

                let result = run_terminal_editor(&repo_root, &prepared);

                if let Err(err) = self.terminal.resume() {
                    self.dispatch(AppEvent::EditorLaunchFailed {
                        path: prepared.path.clone(),
                        error: err.to_string(),
                        show_modal: false,
                    });
                }

                let _ = tui.clear();
                self.state.dirty = true;
                result
            }
        };

        match launch_result {
            Ok(result) => {
                self.dispatch(AppEvent::EditorLaunched {
                    path: result.path,
                    command: result.command,
                });
            }
            Err(err) => {
                self.dispatch(AppEvent::EditorLaunchFailed {
                    path: prepared.path,
                    error: err.user_message(),
                    show_modal: err.is_command_not_found(),
                });
            }
        }
    }

    fn execute_effects(&mut self, effects: Vec<SideEffect>) -> bool {
        for effect in effects {
            match effect {
                SideEffect::Quit => return true,
                SideEffect::SpawnGitRefresh => {
                    if self.state.workspace_meta.is_git_repo {
                        spawn_git_refresh(
                            self.state.repo_root.clone(),
                            self.state.config.git.show_untracked,
                            self.events.sender(),
                        );
                    }
                }
                SideEffect::SpawnGitHubRefresh => {
                    // Issue/PR list refresh will enqueue events in later milestones.
                }
                SideEffect::SpawnGitHubAuthCheck => {
                    spawn_github_auth_check(
                        self.state.config.github.command.clone(),
                        self.events.sender(),
                    );
                }
                SideEffect::SpawnGitHubIssueList => {
                    spawn_github_issue_list_load(
                        self.state.repo_root.clone(),
                        self.state.config.github.command.clone(),
                        self.events.sender(),
                    );
                }
                SideEffect::SpawnGitHubPrList => {
                    spawn_github_pr_list_load(
                        self.state.repo_root.clone(),
                        self.state.config.github.command.clone(),
                        self.events.sender(),
                    );
                }
                SideEffect::SpawnGitHubIssueDetail { number } => {
                    spawn_github_issue_detail_load(
                        self.state.repo_root.clone(),
                        self.state.config.github.command.clone(),
                        number,
                        self.events.sender(),
                    );
                }
                SideEffect::SpawnGitHubPrDetail { number } => {
                    spawn_github_pr_detail_load(
                        self.state.repo_root.clone(),
                        self.state.config.github.command.clone(),
                        number,
                        self.events.sender(),
                    );
                }
                SideEffect::SpawnGitHubIssueComment { number, body } => {
                    spawn_github_issue_comment(
                        self.state.repo_root.clone(),
                        self.state.config.github.command.clone(),
                        number,
                        body,
                        self.events.sender(),
                    );
                }
                SideEffect::SpawnGitHubIssueCreateBranch { number } => {
                    spawn_github_issue_create_branch(
                        self.state.repo_root.clone(),
                        self.state.config.github.command.clone(),
                        number,
                        self.events.sender(),
                    );
                }
                SideEffect::SpawnGitHubRepoLabels => {
                    spawn_github_repo_labels_load(
                        self.state.repo_root.clone(),
                        self.state.config.github.command.clone(),
                        self.events.sender(),
                    );
                }
                SideEffect::SpawnGitHubIssueLabelApply { number, labels } => {
                    spawn_github_issue_label_apply(
                        self.state.repo_root.clone(),
                        self.state.config.github.command.clone(),
                        number,
                        labels,
                        self.events.sender(),
                    );
                }
                SideEffect::SpawnGitHubOpenBrowser { target } => {
                    spawn_github_open_browser(
                        self.state.repo_root.clone(),
                        self.state.config.github.command.clone(),
                        target,
                        self.events.sender(),
                    );
                }
                SideEffect::SpawnGitHubPrCreate { request } => {
                    spawn_github_pr_create(
                        self.state.repo_root.clone(),
                        self.state.config.github.command.clone(),
                        request,
                        self.events.sender(),
                    );
                }
                SideEffect::SpawnAgent => {
                    self.spawn_agent();
                }
                SideEffect::RestartAgent => {
                    self.restart_agent();
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
                SideEffect::SaveWorkspace => {}
                SideEffect::LaunchEditor { path, line } => {
                    self.pending_editor_launch = Some(PendingEditorLaunch { path, line });
                }
                SideEffect::SavePaletteHistory => {
                    if self.state.config.workspace.persist {
                        let _ = save_palette_history(
                            &self.state.repo_root,
                            &self.state.palette.history,
                        );
                    }
                }
                SideEffect::LoadDirectoryChildren(path) => {
                    spawn_directory_load(path, self.events.sender());
                }
                SideEffect::LoadPreviewFile(path) => {
                    spawn_preview_load(
                        path,
                        self.state.config.preview.max_size_bytes,
                        self.events.sender(),
                    );
                }
                SideEffect::LoadFileDiff { path, source } => {
                    spawn_file_diff_load(
                        self.state.repo_root.clone(),
                        path,
                        source,
                        self.state.config.diff.context_lines,
                        self.events.sender(),
                    );
                }
                SideEffect::CancelSearch => {
                    self.search_cancel.cancel();
                    self.search_debounce.clear();
                    self.search_live_generation
                        .store(self.state.search.generation, Ordering::Relaxed);
                }
                SideEffect::RunSearch {
                    mode,
                    query,
                    generation,
                } => {
                    self.search_cancel.clear();
                    spawn_search(
                        SearchJob {
                            mode,
                            query,
                            generation,
                            repo_root: self.state.repo_root.clone(),
                            rg_command: self.state.config.search.command.clone(),
                        },
                        self.events.sender(),
                        self.search_live_generation.clone(),
                        self.search_cancel.clone(),
                    );
                }
                SideEffect::CopyToClipboard(text) => {
                    if let Err(err) = self.clipboard.write_text(&text) {
                        self.state
                            .notifications
                            .show_toast(format!("Copy failed: {err}"));
                        self.state.dirty = true;
                    }
                }
                SideEffect::PasteFromClipboard => match self.clipboard.read_text() {
                    Ok(text) => {
                        let _ = self.dispatch(AppEvent::Command(AppCommand::PasteText(text)));
                    }
                    Err(err) => {
                        self.state
                            .notifications
                            .show_toast(format!("Paste failed: {err}"));
                        self.state.dirty = true;
                    }
                },
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
            Event::Paste(text) => self.dispatch(AppEvent::Command(AppCommand::PasteText(text))),
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

        match mouse.kind {
            MouseEventKind::Down(MouseButton::Left) => self.handle_mouse_left_down(mouse),
            MouseEventKind::Drag(MouseButton::Left) => self.handle_mouse_left_drag(mouse),
            MouseEventKind::Up(MouseButton::Left) => {
                self.dispatch(AppEvent::Command(AppCommand::SelectionEnd))
            }
            _ => false,
        }
    }

    fn handle_mouse_left_down(&mut self, mouse: MouseEvent) -> bool {
        if let Some((pane, pos)) = hit_test_text(&self.state, mouse.column, mouse.row) {
            self.apply_selection_focus(pane);
            return self.dispatch(AppEvent::Command(AppCommand::SelectionBegin {
                pane,
                line: pos.line,
                col: pos.col,
            }));
        }

        let _ = self.dispatch(AppEvent::Command(AppCommand::SelectionClear));
        self.handle_mouse_click(mouse)
    }

    fn handle_mouse_left_drag(&mut self, mouse: MouseEvent) -> bool {
        if !self.state.text_selection.dragging {
            return false;
        }

        let Some(active_pane) = self.state.text_selection.pane else {
            return false;
        };

        if let Some((pane, pos)) = hit_test_text(&self.state, mouse.column, mouse.row) {
            if pane == active_pane {
                return self.dispatch(AppEvent::Command(AppCommand::SelectionExtend {
                    line: pos.line,
                    col: pos.col,
                }));
            }
        }

        false
    }

    fn apply_selection_focus(&mut self, pane: SelectionPane) {
        let commands = match pane {
            SelectionPane::Preview => vec![
                NavCommand::SelectMainTab(MainTab::Preview),
                NavCommand::SetFocus(FocusTarget::Main),
            ],
            SelectionPane::IssueDetail => vec![
                NavCommand::SelectMainTab(MainTab::Issues),
                NavCommand::SetFocus(FocusTarget::Main),
            ],
            SelectionPane::PrDetail => vec![
                NavCommand::SelectMainTab(MainTab::Prs),
                NavCommand::SetFocus(FocusTarget::Main),
            ],
            SelectionPane::Agent => vec![
                NavCommand::SelectMainTab(MainTab::Agent),
                NavCommand::SetFocus(FocusTarget::Main),
            ],
            SelectionPane::Shell => vec![NavCommand::SetFocus(FocusTarget::Shell)],
        };

        for command in commands {
            let _ = self.dispatch(AppEvent::Command(AppCommand::Navigation(command)));
        }
    }

    fn handle_mouse_click(&mut self, mouse: MouseEvent) -> bool {
        if let Some(match_index) = palette_match_at(
            &self.state,
            self.state.layout.rects.palette,
            mouse.column,
            mouse.row,
        ) {
            return self.dispatch(AppEvent::Command(AppCommand::PaletteExecuteMatch(
                match_index,
            )));
        }

        if self.state.navigation.left_tab == LeftNavTab::Files {
            if let Some(action) = file_tree_interaction_at(
                &self.state,
                self.state.layout.rects.left_content,
                mouse.column,
                mouse.row,
            ) {
                let _ = self.dispatch(AppEvent::Command(AppCommand::Navigation(
                    crate::navigation::NavCommand::SetFocus(FocusTarget::Left),
                )));
                return match action {
                    FileTreeMouseAction::Select(path) => {
                        if self
                            .double_click
                            .register(DoubleClickTarget::FileTree(path.clone()))
                        {
                            return self.dispatch_file_tree_open(path);
                        }
                        self.dispatch(AppEvent::Command(AppCommand::FileTreeSelect(path)))
                    }
                    FileTreeMouseAction::Expand(path) => {
                        self.dispatch(AppEvent::Command(AppCommand::FileTreeExpand(path)))
                    }
                    FileTreeMouseAction::Collapse(path) => {
                        self.dispatch(AppEvent::Command(AppCommand::FileTreeCollapse(path)))
                    }
                };
            }
        }

        if self.state.navigation.left_tab == LeftNavTab::Search {
            if let Some(index) = search_interaction_at(
                &self.state,
                self.state.layout.rects.left_content,
                mouse.column,
                mouse.row,
            ) {
                let _ = self.dispatch(AppEvent::Command(AppCommand::Navigation(
                    crate::navigation::NavCommand::SetFocus(FocusTarget::Left),
                )));
                if self
                    .double_click
                    .register(DoubleClickTarget::SearchResult(index))
                {
                    let _ = self.dispatch(AppEvent::Command(AppCommand::SearchSelect(index)));
                    return self.dispatch_preview_for_search_index(index);
                }
                return self.dispatch(AppEvent::Command(AppCommand::SearchSelect(index)));
            }
        }

        if self.state.navigation.left_tab == LeftNavTab::Git {
            if let Some(index) = git_interaction_at(
                &self.state,
                self.state.layout.rects.left_content,
                mouse.column,
                mouse.row,
            ) {
                let _ = self.dispatch(AppEvent::Command(AppCommand::Navigation(
                    crate::navigation::NavCommand::SetFocus(FocusTarget::Left),
                )));
                if self
                    .double_click
                    .register(DoubleClickTarget::GitFile(index))
                {
                    let _ = self.dispatch(AppEvent::Command(AppCommand::GitSelect(index)));
                    return self.dispatch(AppEvent::Command(AppCommand::GitOpenSelected));
                }
                return self.dispatch(AppEvent::Command(AppCommand::GitSelect(index)));
            }
        }

        if self.state.navigation.left_tab == LeftNavTab::Gh
            && self.state.github.left_pane == crate::github::GitHubLeftPane::Prs
        {
            if let Some(index) = github_pr_interaction_at(
                &self.state,
                self.state.layout.rects.left_content,
                mouse.column,
                mouse.row,
            ) {
                let _ = self.dispatch(AppEvent::Command(AppCommand::Navigation(
                    crate::navigation::NavCommand::SetFocus(FocusTarget::Left),
                )));
                if self
                    .double_click
                    .register(DoubleClickTarget::GitHubPr(index))
                {
                    let _ = self.dispatch(AppEvent::Command(AppCommand::GitHubSelectPr(index)));
                    return self.dispatch(AppEvent::Command(AppCommand::GitHubOpenSelected));
                }
                return self.dispatch(AppEvent::Command(AppCommand::GitHubSelectPr(index)));
            }
        }

        if self.state.navigation.left_tab == LeftNavTab::Gh
            && self.state.github.left_pane == crate::github::GitHubLeftPane::Issues
        {
            if let Some(index) = github_issue_interaction_at(
                &self.state,
                self.state.layout.rects.left_content,
                mouse.column,
                mouse.row,
            ) {
                let _ = self.dispatch(AppEvent::Command(AppCommand::Navigation(
                    crate::navigation::NavCommand::SetFocus(FocusTarget::Left),
                )));
                if self
                    .double_click
                    .register(DoubleClickTarget::GitHubIssue(index))
                {
                    let _ = self.dispatch(AppEvent::Command(AppCommand::GitHubSelectIssue(index)));
                    return self.dispatch(AppEvent::Command(AppCommand::GitHubOpenSelected));
                }
                return self.dispatch(AppEvent::Command(AppCommand::GitHubSelectIssue(index)));
            }
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

        if self.state.notifications.modal.is_some() {
            if key.code == KeyCode::Esc {
                return self.dispatch(AppEvent::Command(AppCommand::ModalDismiss));
            }
            return false;
        }

        if self.state.github.label_picker.is_some() {
            return self.handle_label_picker_key(key);
        }

        if self.is_palette_open_key(key) {
            return self.dispatch(AppEvent::Command(AppCommand::PaletteOpen));
        }

        if self.is_search_focus_key(key) {
            return self.dispatch_search_focus();
        }

        if let Some(op) = clipboard_op_from_key(key) {
            let shell_focused =
                self.state.navigation.focus == FocusTarget::Shell && self.state.shell.running;
            let shell_has_selection = self.state.text_selection.applies_to(SelectionPane::Shell);
            if clipboard_shortcut_allowed(op, shell_focused, shell_has_selection) {
                return self.dispatch_clipboard_op(op);
            }
        }

        if self.state.palette.open {
            return self.handle_palette_key(key);
        }

        if self.is_agent_restart_key(key) {
            return self.dispatch(AppEvent::Command(AppCommand::AgentRestart));
        }

        if self.is_focus_cycle_key(key) {
            let command = if key.modifiers.contains(KeyModifiers::SHIFT) {
                crate::navigation::NavCommand::PreviousFocus
            } else {
                crate::navigation::NavCommand::NextFocus
            };
            return self.dispatch(AppEvent::Command(AppCommand::Navigation(command)));
        }

        if self.preview_keys_active() && self.handle_preview_key(key) {
            return false;
        }

        if self.diff_keys_active() && self.handle_diff_key(key) {
            return false;
        }

        if self.file_tree_input_active() && self.handle_file_tree_key(key) {
            return false;
        }

        if self.git_input_active() && self.handle_git_key(key) {
            return false;
        }

        if self.github_input_active() && self.handle_github_key(key) {
            return false;
        }

        if self.search_input_active() && self.handle_search_key(key) {
            return false;
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

    fn dispatch_clipboard_op(&mut self, op: ClipboardOp) -> bool {
        let command = match op {
            ClipboardOp::Copy => AppCommand::ClipboardCopy,
            ClipboardOp::Cut => AppCommand::ClipboardCut,
            ClipboardOp::Paste => AppCommand::ClipboardPaste,
        };
        self.dispatch(AppEvent::Command(command))
    }

    fn is_focus_cycle_key(&self, key: crossterm::event::KeyEvent) -> bool {
        matches!(key.code, KeyCode::Tab)
    }

    fn is_palette_open_key(&self, key: crossterm::event::KeyEvent) -> bool {
        key.modifiers.contains(KeyModifiers::CONTROL)
            && matches!(key.code, KeyCode::Char('p' | 'P'))
    }

    fn handle_label_picker_key(&mut self, key: crossterm::event::KeyEvent) -> bool {
        match key.code {
            KeyCode::Esc => self.dispatch(AppEvent::Command(AppCommand::GitHubLabelPickerCancel)),
            KeyCode::Enter => self.dispatch(AppEvent::Command(AppCommand::GitHubLabelPickerApply)),
            KeyCode::Char(' ') => {
                self.dispatch(AppEvent::Command(AppCommand::GitHubLabelPickerToggle))
            }
            KeyCode::Char('j') | KeyCode::Down => {
                self.dispatch(AppEvent::Command(AppCommand::GitHubLabelPickerMove(1)))
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.dispatch(AppEvent::Command(AppCommand::GitHubLabelPickerMove(-1)))
            }
            _ => false,
        }
    }

    fn handle_palette_key(&mut self, key: crossterm::event::KeyEvent) -> bool {
        let prompt_mode = self.state.palette.prompt.is_some();
        match key.code {
            KeyCode::Esc => self.dispatch(AppEvent::Command(AppCommand::PaletteClose)),
            KeyCode::Enter => self.dispatch(AppEvent::Command(AppCommand::PaletteExecuteSelected)),
            KeyCode::Up if !prompt_mode => {
                if self.state.palette.input.is_empty() {
                    self.dispatch(AppEvent::Command(AppCommand::PaletteHistoryUp))
                } else {
                    self.dispatch(AppEvent::Command(AppCommand::PaletteMoveSelection(-1)))
                }
            }
            KeyCode::Down if !prompt_mode => {
                if self.state.palette.input.is_empty() {
                    self.dispatch(AppEvent::Command(AppCommand::PaletteHistoryDown))
                } else {
                    self.dispatch(AppEvent::Command(AppCommand::PaletteMoveSelection(1)))
                }
            }
            KeyCode::Up if prompt_mode => false,
            KeyCode::Down if prompt_mode => false,
            KeyCode::Backspace => self.dispatch(AppEvent::Command(AppCommand::PaletteBackspace)),
            KeyCode::Char(ch) if !key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.dispatch(AppEvent::Command(AppCommand::PaletteAppendChar(ch)))
            }
            _ => false,
        }
    }

    fn file_tree_input_active(&self) -> bool {
        self.state.navigation.focus == FocusTarget::Left
            && self.state.navigation.left_tab == LeftNavTab::Files
    }

    fn search_input_active(&self) -> bool {
        self.state.navigation.focus == FocusTarget::Left
            && self.state.navigation.left_tab == LeftNavTab::Search
    }

    fn git_input_active(&self) -> bool {
        self.state.navigation.focus == FocusTarget::Left
            && self.state.navigation.left_tab == LeftNavTab::Git
    }

    fn github_input_active(&self) -> bool {
        if self.state.palette.open {
            return false;
        }

        (self.state.navigation.focus == FocusTarget::Left
            && self.state.navigation.left_tab == LeftNavTab::Gh)
            || (self.state.navigation.focus == FocusTarget::Main
                && matches!(
                    self.state.navigation.main_tab,
                    MainTab::Issues | MainTab::Prs
                ))
    }

    fn handle_github_key(&mut self, key: crossterm::event::KeyEvent) -> bool {
        if !key.modifiers.is_empty() && key.code != KeyCode::Char('R') {
            return false;
        }

        let gh_left_focused = self.state.navigation.focus == FocusTarget::Left
            && self.state.navigation.left_tab == LeftNavTab::Gh;
        let gh_issues_list_focused =
            gh_left_focused && self.state.github.left_pane == crate::github::GitHubLeftPane::Issues;
        let gh_prs_list_focused =
            gh_left_focused && self.state.github.left_pane == crate::github::GitHubLeftPane::Prs;
        let issues_detail_focused = self.state.navigation.focus == FocusTarget::Main
            && self.state.navigation.main_tab == MainTab::Issues;
        let prs_detail_focused = self.state.navigation.focus == FocusTarget::Main
            && self.state.navigation.main_tab == MainTab::Prs;

        match key.code {
            KeyCode::Char('R') => {
                self.dispatch(AppEvent::Command(AppCommand::GitHubRefresh));
                true
            }
            KeyCode::Char('o') if self.github_input_active() => {
                self.dispatch(AppEvent::Command(AppCommand::GitHubOpenInBrowser));
                true
            }
            KeyCode::Char('i') if gh_left_focused => {
                self.dispatch(AppEvent::Command(AppCommand::GitHubSelectLeftPane(
                    crate::github::GitHubLeftPane::Issues,
                )));
                true
            }
            KeyCode::Char('p') if gh_left_focused => {
                self.dispatch(AppEvent::Command(AppCommand::GitHubSelectLeftPane(
                    crate::github::GitHubLeftPane::Prs,
                )));
                true
            }
            KeyCode::Char('j') if gh_prs_list_focused => {
                self.dispatch(AppEvent::Command(AppCommand::GitHubMovePrSelection(1)));
                true
            }
            KeyCode::Char('k') if gh_prs_list_focused => {
                self.dispatch(AppEvent::Command(AppCommand::GitHubMovePrSelection(-1)));
                true
            }
            KeyCode::Char('j') if gh_issues_list_focused => {
                self.dispatch(AppEvent::Command(AppCommand::GitHubMoveIssueSelection(1)));
                true
            }
            KeyCode::Char('k') if gh_issues_list_focused => {
                self.dispatch(AppEvent::Command(AppCommand::GitHubMoveIssueSelection(-1)));
                true
            }
            KeyCode::Char('j') if issues_detail_focused => {
                self.dispatch(AppEvent::Command(AppCommand::GitHubIssueDetailScroll(1)));
                true
            }
            KeyCode::Char('k') if issues_detail_focused => {
                self.dispatch(AppEvent::Command(AppCommand::GitHubIssueDetailScroll(-1)));
                true
            }
            KeyCode::PageDown if issues_detail_focused => {
                self.dispatch(AppEvent::Command(AppCommand::GitHubIssueDetailPageScroll(
                    1,
                )));
                true
            }
            KeyCode::PageUp if issues_detail_focused => {
                self.dispatch(AppEvent::Command(AppCommand::GitHubIssueDetailPageScroll(
                    -1,
                )));
                true
            }
            KeyCode::Char('j') if prs_detail_focused => {
                self.dispatch(AppEvent::Command(AppCommand::GitHubPrDetailScroll(1)));
                true
            }
            KeyCode::Char('k') if prs_detail_focused => {
                self.dispatch(AppEvent::Command(AppCommand::GitHubPrDetailScroll(-1)));
                true
            }
            KeyCode::PageDown if prs_detail_focused => {
                self.dispatch(AppEvent::Command(AppCommand::GitHubPrDetailPageScroll(1)));
                true
            }
            KeyCode::PageUp if prs_detail_focused => {
                self.dispatch(AppEvent::Command(AppCommand::GitHubPrDetailPageScroll(-1)));
                true
            }
            KeyCode::Enter if gh_issues_list_focused => {
                self.dispatch(AppEvent::Command(AppCommand::GitHubOpenSelected));
                true
            }
            KeyCode::Enter if gh_prs_list_focused => {
                self.dispatch(AppEvent::Command(AppCommand::GitHubOpenSelected));
                true
            }
            _ => false,
        }
    }

    fn handle_git_key(&mut self, key: crossterm::event::KeyEvent) -> bool {
        if !key.modifiers.is_empty() && key.code != KeyCode::Char('R') {
            return false;
        }

        match key.code {
            KeyCode::Char('j') => self.dispatch(AppEvent::Command(AppCommand::GitMoveSelection(1))),
            KeyCode::Char('k') => {
                self.dispatch(AppEvent::Command(AppCommand::GitMoveSelection(-1)))
            }
            KeyCode::Char('R') => self.dispatch(AppEvent::Command(AppCommand::GitRefresh)),
            KeyCode::Enter => self.dispatch(AppEvent::Command(AppCommand::GitOpenSelected)),
            _ => false,
        }
    }

    fn handle_search_key(&mut self, key: crossterm::event::KeyEvent) -> bool {
        if self.is_search_mode_toggle(key) {
            let mode = match self.state.search.mode {
                SearchMode::Files => SearchMode::Content,
                SearchMode::Content => SearchMode::Files,
            };
            return self.dispatch(AppEvent::Command(AppCommand::SearchSetMode(mode)));
        }

        if !key.modifiers.is_empty() {
            return false;
        }

        match key.code {
            KeyCode::Esc => self.dispatch(AppEvent::Command(AppCommand::SearchClear)),
            KeyCode::Backspace => self.dispatch(AppEvent::Command(AppCommand::SearchBackspace)),
            KeyCode::Enter => self.dispatch_preview_from_search_selection(),
            KeyCode::Char('e') => self.dispatch_open_editor(),
            KeyCode::Char('j') => {
                self.dispatch(AppEvent::Command(AppCommand::SearchMoveSelection(1)))
            }
            KeyCode::Char('k') => {
                self.dispatch(AppEvent::Command(AppCommand::SearchMoveSelection(-1)))
            }
            KeyCode::Char('/') => false,
            KeyCode::Char(ch) => self.dispatch(AppEvent::Command(AppCommand::SearchAppendChar(ch))),
            _ => false,
        }
    }

    fn is_search_mode_toggle(&self, key: crossterm::event::KeyEvent) -> bool {
        key.modifiers.contains(KeyModifiers::CONTROL)
            && matches!(key.code, KeyCode::Char('m' | 'M') | KeyCode::Enter)
    }

    fn dispatch_preview_for_search_index(&mut self, index: usize) -> bool {
        let Some(result) = self.state.search.results.get(index) else {
            return false;
        };
        self.dispatch(AppEvent::Command(AppCommand::PreviewFile {
            path: result.path.clone(),
            line: result.line,
        }))
    }

    fn dispatch_file_tree_open(&mut self, path: PathBuf) -> bool {
        let Some(node) = self.state.file_tree.nodes.get(&path) else {
            return false;
        };

        if node.is_dir {
            return self.dispatch(AppEvent::Command(AppCommand::FileTreeExpand(path)));
        }

        let _ = self.dispatch(AppEvent::Command(AppCommand::FileTreeSelect(path.clone())));
        self.dispatch(AppEvent::Command(AppCommand::PreviewFile {
            path,
            line: None,
        }))
    }

    fn dispatch_preview_from_search_selection(&mut self) -> bool {
        self.dispatch_preview_for_search_index(self.state.search.selected)
    }

    fn dispatch_search_focus(&mut self) -> bool {
        if self.search_input_active() {
            return false;
        }

        let _ = self.dispatch(AppEvent::Command(AppCommand::Navigation(
            NavCommand::SelectLeftTab(LeftNavTab::Search),
        )));
        self.dispatch(AppEvent::Command(AppCommand::Navigation(
            NavCommand::SetFocus(FocusTarget::Left),
        )))
    }

    fn is_search_focus_key(&self, key: crossterm::event::KeyEvent) -> bool {
        key.modifiers.is_empty() && matches!(key.code, KeyCode::Char('/'))
    }

    fn dispatch_open_editor(&mut self) -> bool {
        let Some(target) = resolve_editor_target(&self.state) else {
            return false;
        };
        self.dispatch(AppEvent::Command(AppCommand::OpenEditor {
            path: target.path,
            line: target.line,
        }))
    }

    fn preview_keys_active(&self) -> bool {
        self.state.navigation.main_tab == MainTab::Preview && !self.state.palette.open
    }

    fn diff_keys_active(&self) -> bool {
        self.state.navigation.main_tab == MainTab::Diff && !self.state.palette.open
    }

    fn handle_preview_key(&mut self, key: crossterm::event::KeyEvent) -> bool {
        if !key.modifiers.is_empty() {
            return false;
        }

        match key.code {
            KeyCode::Char('j') => {
                self.dispatch(AppEvent::Command(AppCommand::PreviewScroll(1)));
                true
            }
            KeyCode::Char('k') => {
                self.dispatch(AppEvent::Command(AppCommand::PreviewScroll(-1)));
                true
            }
            KeyCode::PageUp => {
                self.dispatch(AppEvent::Command(AppCommand::PreviewPageScroll(-1)));
                true
            }
            KeyCode::PageDown => {
                self.dispatch(AppEvent::Command(AppCommand::PreviewPageScroll(1)));
                true
            }
            KeyCode::Char('e') => self.dispatch_open_editor(),
            _ => false,
        }
    }

    fn handle_diff_key(&mut self, key: crossterm::event::KeyEvent) -> bool {
        if !key.modifiers.is_empty() {
            return false;
        }

        match key.code {
            KeyCode::Char('j') => {
                self.dispatch(AppEvent::Command(AppCommand::DiffScroll(1)));
                true
            }
            KeyCode::Char('k') => {
                self.dispatch(AppEvent::Command(AppCommand::DiffScroll(-1)));
                true
            }
            KeyCode::PageUp => {
                self.dispatch(AppEvent::Command(AppCommand::DiffPageScroll(-1)));
                true
            }
            KeyCode::PageDown => {
                self.dispatch(AppEvent::Command(AppCommand::DiffPageScroll(1)));
                true
            }
            KeyCode::Char('h') => {
                self.dispatch(AppEvent::Command(AppCommand::DiffHorizontalScroll(-4)));
                true
            }
            KeyCode::Char('l') => {
                self.dispatch(AppEvent::Command(AppCommand::DiffHorizontalScroll(4)));
                true
            }
            KeyCode::Char('s') => {
                self.dispatch(AppEvent::Command(AppCommand::DiffToggleSource));
                true
            }
            KeyCode::Char('n') => {
                self.dispatch(AppEvent::Command(AppCommand::DiffNextFile));
                true
            }
            KeyCode::Char('p') => {
                self.dispatch(AppEvent::Command(AppCommand::DiffPrevFile));
                true
            }
            _ => false,
        }
    }

    fn handle_file_tree_key(&mut self, key: crossterm::event::KeyEvent) -> bool {
        if !key.modifiers.is_empty() {
            return false;
        }

        match key.code {
            KeyCode::Char('j') => {
                self.dispatch(AppEvent::Command(AppCommand::FileTreeMoveSelection(1)))
            }
            KeyCode::Char('k') => {
                self.dispatch(AppEvent::Command(AppCommand::FileTreeMoveSelection(-1)))
            }
            KeyCode::Char('l') => {
                let Some(path) = self.state.file_tree.selected.clone() else {
                    return false;
                };
                self.dispatch(AppEvent::Command(AppCommand::FileTreeExpand(path)))
            }
            KeyCode::Char('h') => {
                let Some(path) = self.state.file_tree.selected.clone() else {
                    return false;
                };
                self.dispatch(AppEvent::Command(AppCommand::FileTreeCollapse(path)))
            }
            KeyCode::Char('r') => self.dispatch(AppEvent::Command(AppCommand::FileTreeRefresh)),
            KeyCode::Char('p') => self.dispatch_preview_from_selection(),
            KeyCode::Char('e') => self.dispatch_open_editor(),
            KeyCode::Enter => self.handle_file_tree_enter(),
            _ => false,
        }
    }

    fn handle_file_tree_enter(&mut self) -> bool {
        let Some(path) = self.state.file_tree.selected.clone() else {
            return false;
        };
        let Some(node) = self.state.file_tree.nodes.get(&path) else {
            return false;
        };

        if node.is_dir {
            self.dispatch(AppEvent::Command(AppCommand::FileTreeExpand(path)))
        } else {
            self.dispatch(AppEvent::Command(AppCommand::PreviewFile {
                path,
                line: None,
            }))
        }
    }

    fn dispatch_preview_from_selection(&mut self) -> bool {
        let Some(path) = self.state.file_tree.selected.clone() else {
            return false;
        };
        let Some(node) = self.state.file_tree.nodes.get(&path) else {
            return false;
        };

        if node.is_dir {
            return false;
        }

        self.dispatch(AppEvent::Command(AppCommand::PreviewFile {
            path,
            line: None,
        }))
    }

    fn is_agent_restart_key(&self, key: crossterm::event::KeyEvent) -> bool {
        self.state.navigation.main_tab == MainTab::Agent
            && key.modifiers.contains(KeyModifiers::CONTROL)
            && key.modifiers.contains(KeyModifiers::SHIFT)
            && matches!(key.code, KeyCode::Char('r' | 'R'))
    }

    fn is_force_quit(&self, key: crossterm::event::KeyEvent) -> bool {
        key.modifiers.contains(KeyModifiers::CONTROL)
            && matches!(key.code, KeyCode::Char('q' | 'Q'))
    }

    fn is_global_quit(&self, key: crossterm::event::KeyEvent) -> bool {
        matches!(key.code, KeyCode::Char('q'))
    }

    fn agent_input_active(&self) -> bool {
        self.state.navigation.focus == FocusTarget::Main
            && self.state.navigation.main_tab == MainTab::Agent
            && self.state.agent.running
    }

    fn handle_agent_key(&mut self, key: crossterm::event::KeyEvent) -> bool {
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

        let letter = KeyEvent {
            code: KeyCode::Char('a'),
            modifiers: KeyModifiers::empty(),
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        };
        assert!(!app.dispatch_key(letter));
    }

    #[test]
    fn agent_focus_ctrl_c_does_not_quit() {
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

        let ctrl_c = KeyEvent {
            code: KeyCode::Char('c'),
            modifiers: KeyModifiers::CONTROL,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        };
        assert!(!app.dispatch_key(ctrl_c));
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
    fn agent_restart_shortcut_dispatches_on_agent_tab() {
        use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers};

        use crate::layout::FocusTarget;
        use crate::navigation::{MainTab, NavCommand};
        use crate::state::AppCommand;

        let mut app = App::new(test_context());
        app.state_mut().navigation.main_tab = MainTab::Agent;
        app.event_sender()
            .send(AppEvent::Command(AppCommand::Navigation(
                NavCommand::SetFocus(FocusTarget::Main),
            )))
            .expect("send");
        app.process_pending_events();

        let key = KeyEvent {
            code: KeyCode::Char('R'),
            modifiers: KeyModifiers::CONTROL | KeyModifiers::SHIFT,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        };
        assert!(!app.dispatch_key(key));
        if std::path::Path::new("/bin/bash").exists()
            || std::path::Path::new("/usr/bin/bash").exists()
        {
            assert!(app.state().agent.spawned);
            assert!(app.state().agent.running);
        }
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

    #[test]
    fn file_tree_j_moves_selection_when_left_files_focused() {
        use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers};

        use crate::file_tree::DirectoryEntry;
        use crate::layout::FocusTarget;
        use crate::navigation::NavCommand;
        use crate::state::{AppCommand, AppEvent};

        let mut app = App::new(test_context());
        let root = app.state().file_tree.root.clone();
        app.event_sender()
            .send(AppEvent::Command(AppCommand::Navigation(
                NavCommand::SetFocus(FocusTarget::Left),
            )))
            .expect("send");
        app.event_sender()
            .send(AppEvent::Command(AppCommand::FileTreeExpand(root.clone())))
            .expect("send");
        app.event_sender()
            .send(AppEvent::FileTreeChildrenLoaded {
                parent: root.clone(),
                children: vec![
                    DirectoryEntry {
                        path: root.join("src"),
                        name: "src".to_string(),
                        is_dir: true,
                    },
                    DirectoryEntry {
                        path: root.join("README.md"),
                        name: "README.md".to_string(),
                        is_dir: false,
                    },
                ],
                error: None,
            })
            .expect("send");
        app.process_pending_events();

        let key = KeyEvent {
            code: KeyCode::Char('j'),
            modifiers: KeyModifiers::empty(),
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        };
        assert!(!app.dispatch_key(key));
        assert_eq!(app.state().file_tree.selected, Some(root.join("src")));
    }

    #[test]
    fn double_click_file_tree_file_opens_preview_tab() {
        use crossterm::event::{KeyModifiers, MouseButton, MouseEvent, MouseEventKind};

        use crate::file_tree::DirectoryEntry;
        use crate::navigation::{LeftNavTab, MainTab, NavCommand};
        use crate::state::{AppCommand, AppEvent};

        let mut app = App::new(test_context());
        let root = app.state().file_tree.root.clone();
        app.event_sender()
            .send(AppEvent::Command(AppCommand::Navigation(
                NavCommand::SelectLeftTab(LeftNavTab::Files),
            )))
            .expect("send");
        app.event_sender()
            .send(AppEvent::Command(AppCommand::FileTreeExpand(root.clone())))
            .expect("send");
        app.event_sender()
            .send(AppEvent::FileTreeChildrenLoaded {
                parent: root.clone(),
                children: vec![DirectoryEntry {
                    path: root.join("README.md"),
                    name: "README.md".to_string(),
                    is_dir: false,
                }],
                error: None,
            })
            .expect("send");
        app.process_pending_events();

        let area = app.state().layout.rects.left_content;
        let row = area.y + 2;
        let mouse = MouseEvent {
            kind: MouseEventKind::Down(MouseButton::Left),
            column: area.x + 4,
            row,
            modifiers: KeyModifiers::empty(),
        };

        app.dispatch_mouse(mouse);
        app.dispatch_mouse(mouse);
        assert_eq!(app.state().navigation.main_tab, MainTab::Preview);
        assert_eq!(app.state().file_tree.selected, Some(root.join("README.md")));
        assert_eq!(app.state().preview.path, Some(root.join("README.md")));
    }

    #[test]
    fn double_click_search_result_opens_preview_tab() {
        use std::path::PathBuf;

        use crossterm::event::{KeyModifiers, MouseButton, MouseEvent, MouseEventKind};

        use crate::navigation::{LeftNavTab, MainTab, NavCommand};
        use crate::search::{SearchMode, SearchResult, SearchState};
        use crate::state::{AppCommand, AppEvent};

        let mut app = App::new(test_context());
        app.state_mut().search = SearchState {
            mode: SearchMode::Content,
            query: "main".to_string(),
            results: vec![SearchResult::content(
                PathBuf::from("src/main.rs"),
                12,
                "fn main()".to_string(),
            )],
            ..SearchState::default()
        };
        app.event_sender()
            .send(AppEvent::Command(AppCommand::Navigation(
                NavCommand::SelectLeftTab(LeftNavTab::Search),
            )))
            .expect("send");
        app.process_pending_events();

        let area = app.state().layout.rects.left_content;
        let row = (area.y..area.y.saturating_add(area.height))
            .find(|row| {
                crate::ui::search_interaction_at(app.state(), area, area.x + 2, *row) == Some(0)
            })
            .expect("search result row");
        let mouse = MouseEvent {
            kind: MouseEventKind::Down(MouseButton::Left),
            column: area.x + 2,
            row,
            modifiers: KeyModifiers::empty(),
        };

        app.dispatch_mouse(mouse);
        app.dispatch_mouse(mouse);
        assert_eq!(app.state().navigation.main_tab, MainTab::Preview);
        assert_eq!(app.state().preview.path, Some(PathBuf::from("src/main.rs")));
        assert_eq!(app.state().search.selected, 0);
    }

    #[test]
    fn double_click_github_issue_opens_issues_detail() {
        use crossterm::event::{KeyModifiers, MouseButton, MouseEvent, MouseEventKind};

        use crate::github::{GitHubLeftPane, Issue, IssueState};
        use crate::layout::FocusTarget;
        use crate::navigation::{LeftNavTab, MainTab, NavCommand};
        use crate::state::{AppCommand, AppEvent, GitHubState};
        use crate::ui::github_issue_interaction_at;

        let mut app = App::new(test_context());
        app.state_mut().github = GitHubState {
            auth_checked: true,
            auth_ok: true,
            issues: vec![Issue {
                number: 42,
                title: "Mouse issue".to_string(),
                state: IssueState::Open,
                labels: Vec::new(),
                assignees: Vec::new(),
            }],
            selected_issue: None,
            left_pane: GitHubLeftPane::Issues,
            ..GitHubState::default()
        };
        app.event_sender()
            .send(AppEvent::Command(AppCommand::Navigation(
                NavCommand::SelectLeftTab(LeftNavTab::Gh),
            )))
            .expect("send");
        app.process_pending_events();

        let area = app.state().layout.rects.left_content;
        let row = (area.y..area.y.saturating_add(area.height))
            .find(|row| github_issue_interaction_at(app.state(), area, area.x + 2, *row).is_some())
            .expect("github issue row");
        let mouse = MouseEvent {
            kind: MouseEventKind::Down(MouseButton::Left),
            column: area.x + 2,
            row,
            modifiers: KeyModifiers::empty(),
        };

        app.dispatch_mouse(mouse);
        assert_eq!(app.state().github.selected_issue, Some(42));

        app.dispatch_mouse(mouse);
        assert_eq!(app.state().navigation.main_tab, MainTab::Issues);
        assert_eq!(app.state().navigation.focus, FocusTarget::Main);
        assert!(app.state().github.issue_detail_loading);
    }

    #[test]
    fn double_click_git_file_opens_diff_tab() {
        use crossterm::event::{KeyModifiers, MouseButton, MouseEvent, MouseEventKind};

        use crate::git::{GitFileEntry, GitFileStatus};
        use crate::navigation::{LeftNavTab, MainTab, NavCommand};
        use crate::state::{AppCommand, AppEvent, GitState};
        use crate::ui::git_interaction_at;

        let mut app = App::new(test_context());
        app.state_mut().workspace_meta.is_git_repo = true;
        app.state_mut().git = GitState {
            branch: Some("main".to_string()),
            file_entries: vec![GitFileEntry {
                path: "src/main.rs".to_string(),
                status: GitFileStatus::Modified,
            }],
            ..GitState::default()
        };
        app.event_sender()
            .send(AppEvent::Command(AppCommand::Navigation(
                NavCommand::SelectLeftTab(LeftNavTab::Git),
            )))
            .expect("send");
        app.process_pending_events();

        let area = app.state().layout.rects.left_content;
        let row = (area.y..area.y.saturating_add(area.height))
            .find(|row| git_interaction_at(app.state(), area, area.x + 2, *row).is_some())
            .expect("git file row");
        let mouse = MouseEvent {
            kind: MouseEventKind::Down(MouseButton::Left),
            column: area.x + 2,
            row,
            modifiers: KeyModifiers::empty(),
        };

        app.dispatch_mouse(mouse);
        app.dispatch_mouse(mouse);
        assert_eq!(app.state().navigation.main_tab, MainTab::Diff);
        assert_eq!(
            app.state().diff.selected_path.as_deref(),
            Some("src/main.rs")
        );
        assert_eq!(
            app.state().git.selected_path.as_deref(),
            Some("src/main.rs")
        );
    }

    #[test]
    fn diff_toggle_source_key_works_on_main_diff_tab_with_shell_focus() {
        use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers};

        use crate::diff::DiffSource;
        use crate::layout::FocusTarget;
        use crate::navigation::{LeftNavTab, MainTab};

        let mut app = App::new(test_context());
        app.state_mut().navigation.main_tab = MainTab::Diff;
        app.state_mut().navigation.left_tab = LeftNavTab::Git;
        app.state_mut().navigation.focus = FocusTarget::Shell;
        app.state_mut().diff.selected_path = Some("src/main.rs".to_string());
        app.state_mut().diff.source = DiffSource::Unstaged;

        let key = KeyEvent {
            code: KeyCode::Char('s'),
            modifiers: KeyModifiers::empty(),
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        };

        assert!(!app.dispatch_key(key));
        assert_eq!(app.state().diff.source, DiffSource::Staged);
        assert!(app.state().diff.loading);
    }

    #[test]
    fn diff_toggle_source_key_does_not_quit_app() {
        use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers};

        use crate::diff::DiffSource;
        use crate::navigation::MainTab;

        let mut app = App::new(test_context());
        app.state_mut().navigation.main_tab = MainTab::Diff;
        app.state_mut().diff.selected_path = Some("src/main.rs".to_string());
        app.state_mut().diff.source = DiffSource::Unstaged;

        let key = KeyEvent {
            code: KeyCode::Char('s'),
            modifiers: KeyModifiers::empty(),
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        };

        assert!(!app.dispatch_key(key));
        assert_eq!(app.state().diff.source, DiffSource::Staged);
    }

    #[test]
    fn diff_next_file_key_does_not_quit_app() {
        use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers};

        use crate::git::{GitFileEntry, GitFileStatus};
        use crate::navigation::MainTab;

        let mut app = App::new(test_context());
        app.state_mut().navigation.main_tab = MainTab::Diff;
        app.state_mut().git.file_entries = vec![
            GitFileEntry {
                path: "a.rs".to_string(),
                status: GitFileStatus::Modified,
            },
            GitFileEntry {
                path: "b.rs".to_string(),
                status: GitFileStatus::Modified,
            },
        ];
        app.state_mut().diff.selected_path = Some("a.rs".to_string());

        let key = KeyEvent {
            code: KeyCode::Char('n'),
            modifiers: KeyModifiers::empty(),
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        };

        assert!(!app.dispatch_key(key));
        assert_eq!(app.state().diff.selected_path.as_deref(), Some("b.rs"));
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
