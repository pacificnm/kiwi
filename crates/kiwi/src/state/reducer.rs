use crate::layout::compute_layout;
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
    if let Ok(layout) = compute_layout(width, height, state.config.app.left_width) {
        if state.layout != layout {
            state.layout = layout;
            state.dirty = true;
        }
    }
    Vec::new()
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
    fn shell_output_appends_to_scrollback_and_sets_dirty() {
        let mut state = test_state();
        reduce(&mut state, AppEvent::ShellOutput(b"hello\nworld".to_vec()));
        assert_eq!(state.shell.scrollback.line_count(), 1);
        assert!(state.dirty);
    }
}
