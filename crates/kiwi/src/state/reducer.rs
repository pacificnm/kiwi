use crate::agent::infer_status_from_scrollback;
use crate::commands::{execute_command, refresh_matches};
use crate::layout::{agent_pty_size, compute_layout, shell_pty_size, FocusTarget};
use crate::navigation::{MainTab, NavCommand};

use super::app_state::AppState;
use super::event::{AppCommand, AppEvent, SideEffect};

pub fn reduce(state: &mut AppState, event: AppEvent) -> Vec<SideEffect> {
    match event {
        AppEvent::Command(command) => reduce_command(state, command),
        AppEvent::TerminalResize { width, height } => reduce_terminal_resize(state, width, height),
        AppEvent::GitRefreshRequested => reduce_git_refresh_requested(state),
        AppEvent::GitStatusUpdated { modified_files } => {
            reduce_git_status_updated(state, modified_files)
        }
        AppEvent::Quit => {
            state.dirty = true;
            vec![SideEffect::Quit]
        }
        AppEvent::ShellOutput(data) => reduce_shell_output(state, data),
        AppEvent::ShellExited(_code) => reduce_shell_exited(state),
        AppEvent::AgentOutput(data) => reduce_agent_output(state, data),
        AppEvent::AgentExited(code) => reduce_agent_exited(state, code),
    }
}

fn reduce_command(state: &mut AppState, command: AppCommand) -> Vec<SideEffect> {
    match command {
        AppCommand::Navigation(nav) => {
            apply_navigation(state, nav);
            agent_spawn_effects_if_needed(state)
        }
        AppCommand::Quit => {
            state.dirty = true;
            vec![SideEffect::Quit]
        }
        AppCommand::RequestGitRefresh => reduce_git_refresh_requested(state),
        AppCommand::ShellWrite(data) => vec![SideEffect::WriteShell(data)],
        AppCommand::ShellScroll(delta) => reduce_shell_scroll(state, delta),
        AppCommand::AgentWrite(data) => vec![SideEffect::WriteAgent(data)],
        AppCommand::AgentScroll(delta) => reduce_agent_scroll(state, delta),
        AppCommand::AgentRestart => reduce_agent_restart(state),
        AppCommand::PaletteOpen => reduce_palette_open(state),
        AppCommand::PaletteClose => reduce_palette_close(state),
        AppCommand::PaletteAppendChar(ch) => reduce_palette_append_char(state, ch),
        AppCommand::PaletteBackspace => reduce_palette_backspace(state),
        AppCommand::PaletteMoveSelection(delta) => reduce_palette_move_selection(state, delta),
        AppCommand::PaletteHistoryUp => reduce_palette_history_up(state),
        AppCommand::PaletteHistoryDown => reduce_palette_history_down(state),
        AppCommand::PaletteExecuteSelected => reduce_palette_execute_selected(state),
        AppCommand::PaletteExecuteMatch(index) => reduce_palette_execute_match(state, index),
    }
}

fn apply_navigation(state: &mut AppState, command: NavCommand) {
    let before = state.navigation.clone();
    state.navigation.apply(command);
    if state.navigation != before {
        state.dirty = true;
    }
}

pub fn agent_spawn_effects_if_needed(state: &mut AppState) -> Vec<SideEffect> {
    if state.navigation.main_tab != MainTab::Agent || state.agent.spawned {
        return Vec::new();
    }

    state.dirty = true;
    vec![SideEffect::SpawnAgent]
}

fn reduce_terminal_resize(state: &mut AppState, width: u16, height: u16) -> Vec<SideEffect> {
    let Ok(layout) = compute_layout(width, height, state.config.app.left_width) else {
        return Vec::new();
    };

    if state.layout == layout {
        return Vec::new();
    }

    state.layout = layout;
    state.dirty = true;

    if !state.shell.running {
        return Vec::new();
    }

    let (cols, rows) = shell_pty_size(&state.layout.rects);
    if cols == state.shell.cols && rows == state.shell.rows {
        return Vec::new();
    }

    state.shell.apply_resize(cols, rows);
    vec![SideEffect::ResizeShell { cols, rows }]
}

fn reduce_git_refresh_requested(state: &mut AppState) -> Vec<SideEffect> {
    state.dirty = true;
    vec![SideEffect::SpawnGitRefresh]
}

fn reduce_git_status_updated(state: &mut AppState, modified_files: Vec<String>) -> Vec<SideEffect> {
    let selected = state.git.selected_path.clone();
    state.git.modified_files = modified_files;

    if let Some(path) = selected {
        if state.git.modified_files.iter().any(|file| file == &path) {
            state.git.selected_path = Some(path);
        } else {
            state.git.selected_path = None;
        }
    }

    state.dirty = true;
    Vec::new()
}

fn reduce_shell_output(state: &mut AppState, data: Vec<u8>) -> Vec<SideEffect> {
    state.shell.scrollback.set_cols(state.shell.cols);
    state.shell.scrollback.append_bytes(&data);
    state.dirty = true;
    Vec::new()
}

fn reduce_shell_exited(state: &mut AppState) -> Vec<SideEffect> {
    state.shell.running = false;
    state.dirty = true;
    Vec::new()
}

fn reduce_agent_output(state: &mut AppState, data: Vec<u8>) -> Vec<SideEffect> {
    state.agent.scrollback.set_cols(state.agent.cols);
    state.agent.scrollback.append_bytes(&data);
    if let Some(status) = infer_status_from_scrollback(&state.agent.scrollback) {
        state.agent.status = status;
    }
    state.dirty = true;
    Vec::new()
}

fn reduce_agent_exited(state: &mut AppState, code: i32) -> Vec<SideEffect> {
    state.agent.apply_exit(code);
    state.dirty = true;
    Vec::new()
}

fn reduce_agent_restart(state: &mut AppState) -> Vec<SideEffect> {
    if state.navigation.main_tab != MainTab::Agent {
        return Vec::new();
    }

    state.dirty = true;
    vec![SideEffect::RestartAgent]
}

fn reduce_shell_scroll(state: &mut AppState, delta: i32) -> Vec<SideEffect> {
    if delta == 0 {
        return Vec::new();
    }

    let (_, page_size) = shell_pty_size(&state.layout.rects);
    state.shell.scroll_by(delta, page_size);
    state.dirty = true;
    Vec::new()
}

fn reduce_agent_scroll(state: &mut AppState, delta: i32) -> Vec<SideEffect> {
    if delta == 0 {
        return Vec::new();
    }

    let (_, page_size) = agent_pty_size(&state.layout.rects);
    state.agent.scroll_by(delta, page_size);
    state.dirty = true;
    Vec::new()
}

fn reduce_palette_open(state: &mut AppState) -> Vec<SideEffect> {
    state.palette.open_with_focus(state.navigation.focus);
    state.navigation.focus = FocusTarget::CommandPalette;
    refresh_matches(state);
    state.dirty = true;
    Vec::new()
}

fn reduce_palette_close(state: &mut AppState) -> Vec<SideEffect> {
    if !state.palette.open {
        return Vec::new();
    }

    state.palette.close(&mut state.navigation.focus);
    state.dirty = true;
    Vec::new()
}

fn reduce_palette_append_char(state: &mut AppState, ch: char) -> Vec<SideEffect> {
    if !state.palette.open {
        return Vec::new();
    }

    state.palette.history_cursor = None;
    state.palette.input.push(ch);
    refresh_matches(state);
    state.dirty = true;
    Vec::new()
}

fn reduce_palette_backspace(state: &mut AppState) -> Vec<SideEffect> {
    if !state.palette.open {
        return Vec::new();
    }

    state.palette.history_cursor = None;
    state.palette.input.pop();
    refresh_matches(state);
    state.dirty = true;
    Vec::new()
}

fn reduce_palette_move_selection(state: &mut AppState, delta: i32) -> Vec<SideEffect> {
    if !state.palette.open {
        return Vec::new();
    }

    state.palette.move_selection(delta as isize);
    state.dirty = true;
    Vec::new()
}

fn reduce_palette_history_up(state: &mut AppState) -> Vec<SideEffect> {
    if !state.palette.open {
        return Vec::new();
    }

    state.palette.history_up();
    refresh_matches(state);
    state.dirty = true;
    Vec::new()
}

fn reduce_palette_history_down(state: &mut AppState) -> Vec<SideEffect> {
    if !state.palette.open {
        return Vec::new();
    }

    state.palette.history_down();
    refresh_matches(state);
    state.dirty = true;
    Vec::new()
}

fn reduce_palette_execute_selected(state: &mut AppState) -> Vec<SideEffect> {
    if !state.palette.open {
        return Vec::new();
    }

    let Some(registry_index) = state.palette.matches.get(state.palette.selected).copied() else {
        return Vec::new();
    };

    execute_command(state, registry_index)
}

fn reduce_palette_execute_match(state: &mut AppState, match_index: usize) -> Vec<SideEffect> {
    if !state.palette.open {
        return Vec::new();
    }

    let Some(registry_index) = state.palette.matches.get(match_index).copied() else {
        return Vec::new();
    };

    execute_command(state, registry_index)
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::agent::AgentStatus;
    use crate::config::ResolvedConfig;
    use crate::layout::compute_layout;
    use crate::layout::FocusTarget;
    use crate::navigation::{LeftNavTab, MainTab, NavCommand, NavigationState};
    use crate::theme::capabilities::TerminalCapabilities;
    use crate::theme::loader::load_theme_with_capabilities;

    use super::*;
    use crate::state::domains::{
        AgentState, CommandPaletteState, DiffState, FileTreeState, GitHubState, GitState,
        PreviewState, SearchState, ShellState, StatusBarState, WorkspaceMeta,
    };

    fn test_state() -> AppState {
        AppState {
            config: ResolvedConfig::default(),
            navigation: NavigationState::default(),
            layout: compute_layout(120, 40, 30).expect("layout"),
            theme: load_theme_with_capabilities(
                &ResolvedConfig::default().theme,
                TerminalCapabilities::TrueColor,
            )
            .expect("theme"),
            repo_root: PathBuf::from("."),
            dirty: false,
            file_tree: FileTreeState::default(),
            preview: PreviewState::default(),
            search: SearchState::default(),
            git: GitState::default(),
            diff: DiffState::default(),
            github: GitHubState::default(),
            agent: AgentState::default(),
            shell: ShellState::default(),
            palette: CommandPaletteState::default(),
            status_bar: StatusBarState::default(),
            workspace_meta: WorkspaceMeta::default(),
        }
    }

    #[test]
    fn navigation_command_sets_dirty() {
        let mut state = test_state();
        reduce(
            &mut state,
            AppEvent::Command(AppCommand::Navigation(NavCommand::SelectLeftTab(
                LeftNavTab::Git,
            ))),
        );
        assert!(state.dirty);
        assert_eq!(state.navigation.left_tab, LeftNavTab::Git);
    }

    #[test]
    fn orthogonal_tabs_preserved_in_app_state() {
        let mut state = test_state();
        reduce(
            &mut state,
            AppEvent::Command(AppCommand::Navigation(NavCommand::SelectLeftTab(
                LeftNavTab::Git,
            ))),
        );
        reduce(
            &mut state,
            AppEvent::Command(AppCommand::Navigation(NavCommand::SelectMainTab(
                MainTab::Issues,
            ))),
        );
        assert_eq!(state.navigation.left_tab, LeftNavTab::Git);
        assert_eq!(state.navigation.main_tab, MainTab::Issues);
    }

    #[test]
    fn git_refresh_preserves_selection_when_file_still_modified() {
        let mut state = test_state();
        state.git.selected_path = Some("src/main.rs".to_string());
        state.git.modified_files = vec!["src/main.rs".to_string(), "other.rs".to_string()];

        reduce(
            &mut state,
            AppEvent::GitStatusUpdated {
                modified_files: vec!["src/main.rs".to_string(), "new.rs".to_string()],
            },
        );

        assert_eq!(state.git.selected_path, Some("src/main.rs".to_string()));
    }

    #[test]
    fn git_refresh_clears_selection_when_file_no_longer_modified() {
        let mut state = test_state();
        state.git.selected_path = Some("removed.rs".to_string());

        reduce(
            &mut state,
            AppEvent::GitStatusUpdated {
                modified_files: vec!["other.rs".to_string()],
            },
        );

        assert_eq!(state.git.selected_path, None);
    }

    #[test]
    fn git_refresh_requested_emits_side_effect() {
        let mut state = test_state();
        let effects = reduce(&mut state, AppEvent::GitRefreshRequested);
        assert!(effects.contains(&SideEffect::SpawnGitRefresh));
    }

    #[test]
    fn quit_command_emits_side_effect() {
        let mut state = test_state();
        let effects = reduce(&mut state, AppEvent::Command(AppCommand::Quit));
        assert!(effects.contains(&SideEffect::Quit));
    }

    #[test]
    fn terminal_resize_updates_layout() {
        let mut state = test_state();
        let before = state.layout;
        reduce(
            &mut state,
            AppEvent::TerminalResize {
                width: 160,
                height: 50,
            },
        );
        assert_ne!(state.layout, before);
        assert!(state.dirty);
    }

    #[test]
    fn agent_spawn_not_requested_off_agent_tab() {
        let mut state = test_state();
        state.navigation.main_tab = MainTab::Issues;
        let effects = agent_spawn_effects_if_needed(&mut state);
        assert!(effects.is_empty());
    }

    #[test]
    fn agent_spawn_requested_on_agent_tab() {
        let mut state = test_state();
        state.navigation.main_tab = MainTab::Agent;
        state.agent.spawned = false;
        let effects = agent_spawn_effects_if_needed(&mut state);
        assert!(effects.contains(&SideEffect::SpawnAgent));
    }

    #[test]
    fn agent_spawn_requested_only_once() {
        let mut state = test_state();
        state.agent.spawned = true;
        let effects = agent_spawn_effects_if_needed(&mut state);
        assert!(effects.is_empty());
    }

    #[test]
    fn selecting_agent_tab_emits_spawn_side_effect() {
        let mut state = test_state();
        state.navigation.main_tab = MainTab::Issues;
        let effects = reduce(
            &mut state,
            AppEvent::Command(AppCommand::Navigation(NavCommand::SelectMainTab(
                MainTab::Agent,
            ))),
        );
        assert!(effects.contains(&SideEffect::SpawnAgent));
    }

    #[test]
    fn terminal_resize_emits_shell_resize_when_running() {
        let mut state = test_state();
        state.shell.running = true;
        state.shell.cols = shell_pty_size(&state.layout.rects).0;
        state.shell.rows = shell_pty_size(&state.layout.rects).1;

        let effects = reduce(
            &mut state,
            AppEvent::TerminalResize {
                width: 160,
                height: 50,
            },
        );

        let (cols, rows) = shell_pty_size(&state.layout.rects);
        assert!(effects.contains(&SideEffect::ResizeShell { cols, rows }));
        assert_eq!(state.shell.cols, cols);
        assert_eq!(state.shell.rows, rows);
    }

    #[test]
    fn shell_output_appends_to_scrollback_and_sets_dirty() {
        let mut state = test_state();
        reduce(&mut state, AppEvent::ShellOutput(b"hello\nworld".to_vec()));
        assert_eq!(state.shell.scrollback.line_count(), 2);
        assert!(state.dirty);
    }

    #[test]
    fn agent_output_appends_to_scrollback_and_sets_dirty() {
        let mut state = test_state();
        reduce(&mut state, AppEvent::AgentOutput(b"agent line\n".to_vec()));
        assert_eq!(state.agent.scrollback.line_count(), 1);
        assert!(state.dirty);
    }

    #[test]
    fn agent_exited_clears_running_and_sets_dirty() {
        let mut state = test_state();
        state.agent.running = true;
        state.agent.status = AgentStatus::Executing;
        reduce(&mut state, AppEvent::AgentExited(1));
        assert!(!state.agent.running);
        assert_eq!(state.agent.status, AgentStatus::Error);
        assert!(state.dirty);
    }

    #[test]
    fn agent_output_updates_status_from_heuristics() {
        let mut state = test_state();
        state.agent.running = true;
        reduce(
            &mut state,
            AppEvent::AgentOutput(b"Thinking about the next step\n".to_vec()),
        );
        assert_eq!(state.agent.status, AgentStatus::Thinking);
    }

    #[test]
    fn agent_exited_zero_sets_success_status() {
        let mut state = test_state();
        state.agent.running = true;
        reduce(&mut state, AppEvent::AgentExited(0));
        assert_eq!(state.agent.status, AgentStatus::Success);
        assert_eq!(state.agent.exit_code, Some(0));
        assert!(state.agent.restart_hint.is_some());
    }

    #[test]
    fn agent_restart_emits_side_effect_on_agent_tab() {
        let mut state = test_state();
        state.navigation.main_tab = MainTab::Agent;
        let effects = reduce(&mut state, AppEvent::Command(AppCommand::AgentRestart));
        assert!(effects.contains(&SideEffect::RestartAgent));
        assert!(state.dirty);
    }

    #[test]
    fn agent_restart_ignored_off_agent_tab() {
        let mut state = test_state();
        state.navigation.main_tab = MainTab::Issues;
        let effects = reduce(&mut state, AppEvent::Command(AppCommand::AgentRestart));
        assert!(effects.is_empty());
    }

    #[test]
    fn agent_exited_sets_restart_hint_with_code() {
        let mut state = test_state();
        reduce(&mut state, AppEvent::AgentExited(2));
        assert_eq!(state.agent.exit_code, Some(2));
        assert!(state
            .agent
            .restart_hint
            .as_deref()
            .is_some_and(|hint| hint.contains("code 2")));
    }

    #[test]
    fn shell_write_emits_side_effect() {
        let mut state = test_state();
        let effects = reduce(
            &mut state,
            AppEvent::Command(AppCommand::ShellWrite(b"ls\n".to_vec())),
        );
        assert!(effects.contains(&SideEffect::WriteShell(b"ls\n".to_vec())));
    }

    #[test]
    fn agent_write_emits_side_effect() {
        let mut state = test_state();
        let effects = reduce(
            &mut state,
            AppEvent::Command(AppCommand::AgentWrite(b"hello\n".to_vec())),
        );
        assert!(effects.contains(&SideEffect::WriteAgent(b"hello\n".to_vec())));
    }

    #[test]
    fn agent_scroll_moves_viewport_and_clears_follow_tail() {
        let mut state = test_state();
        for index in 0..40 {
            state
                .agent
                .scrollback
                .append_bytes(format!("line {index}\n").as_bytes());
        }

        reduce(&mut state, AppEvent::Command(AppCommand::AgentScroll(-1)));

        assert!(!state.agent.follow_tail);
        assert!(state.dirty);
    }

    #[test]
    fn palette_open_sets_state_and_focus() {
        let mut state = test_state();
        reduce(&mut state, AppEvent::Command(AppCommand::PaletteOpen));
        assert!(state.palette.open);
        assert_eq!(state.navigation.focus, FocusTarget::CommandPalette);
        assert!(!state.palette.matches.is_empty());
    }

    #[test]
    fn palette_close_restores_previous_focus() {
        let mut state = test_state();
        state.navigation.focus = FocusTarget::Shell;
        reduce(&mut state, AppEvent::Command(AppCommand::PaletteOpen));
        reduce(&mut state, AppEvent::Command(AppCommand::PaletteClose));
        assert!(!state.palette.open);
        assert_eq!(state.navigation.focus, FocusTarget::Shell);
    }

    #[test]
    fn palette_fuzzy_query_matches_git_refresh() {
        let mut state = test_state();
        reduce(&mut state, AppEvent::Command(AppCommand::PaletteOpen));
        reduce(
            &mut state,
            AppEvent::Command(AppCommand::PaletteAppendChar('g')),
        );
        reduce(
            &mut state,
            AppEvent::Command(AppCommand::PaletteAppendChar('i')),
        );
        reduce(
            &mut state,
            AppEvent::Command(AppCommand::PaletteAppendChar('t')),
        );
        reduce(
            &mut state,
            AppEvent::Command(AppCommand::PaletteAppendChar(' ')),
        );
        reduce(
            &mut state,
            AppEvent::Command(AppCommand::PaletteAppendChar('r')),
        );
        reduce(
            &mut state,
            AppEvent::Command(AppCommand::PaletteAppendChar('e')),
        );
        reduce(
            &mut state,
            AppEvent::Command(AppCommand::PaletteAppendChar('f')),
        );
        let first = state.palette.matches.first().copied().expect("match");
        assert_eq!(crate::commands::COMMANDS[first].id, "git.refresh");
    }

    #[test]
    fn palette_execute_selected_closes_palette() {
        let mut state = test_state();
        reduce(&mut state, AppEvent::Command(AppCommand::PaletteOpen));
        reduce(
            &mut state,
            AppEvent::Command(AppCommand::PaletteExecuteSelected),
        );
        assert!(!state.palette.open);
    }
}
