use crate::layout::compute_layout;
use crate::layout::shell_pty_size;
use crate::navigation::NavCommand;

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
    }
}

fn reduce_command(state: &mut AppState, command: AppCommand) -> Vec<SideEffect> {
    match command {
        AppCommand::Navigation(nav) => {
            apply_navigation(state, nav);
            Vec::new()
        }
        AppCommand::Quit => {
            state.dirty = true;
            vec![SideEffect::Quit]
        }
        AppCommand::RequestGitRefresh => reduce_git_refresh_requested(state),
        AppCommand::ShellWrite(data) => vec![SideEffect::WriteShell(data)],
        AppCommand::ShellScroll(delta) => reduce_shell_scroll(state, delta),
    }
}

fn apply_navigation(state: &mut AppState, command: NavCommand) {
    let before = state.navigation.clone();
    state.navigation.apply(command);
    if state.navigation != before {
        state.dirty = true;
    }
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
    state.shell.scrollback.append_bytes(&data);
    state.dirty = true;
    Vec::new()
}

fn reduce_shell_exited(state: &mut AppState) -> Vec<SideEffect> {
    state.shell.running = false;
    state.dirty = true;
    Vec::new()
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

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::config::ResolvedConfig;
    use crate::layout::compute_layout;
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
        assert_eq!(state.shell.scrollback.line_count(), 1);
        assert!(state.dirty);
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
    fn shell_scroll_moves_viewport_and_clears_follow_tail() {
        let mut state = test_state();
        for index in 0..20 {
            state
                .shell
                .scrollback
                .append_bytes(format!("line {index}\n").as_bytes());
        }

        reduce(&mut state, AppEvent::Command(AppCommand::ShellScroll(-1)));

        assert!(!state.shell.follow_tail);
        assert!(state.dirty);
    }
}
