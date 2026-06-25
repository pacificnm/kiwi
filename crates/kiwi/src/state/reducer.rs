use std::path::PathBuf;

use crate::agent::infer_status_from_scrollback;
use crate::commands::{execute_command, history_input_for_id, refresh_matches};
use crate::file_tree::ExpandAction;
use crate::git::GitFileEntry;
use crate::layout::{agent_pty_size, compute_layout, shell_pty_size, FocusTarget};
use crate::navigation::{LeftNavTab, MainTab, NavCommand};

use super::app_state::AppState;
use super::event::{AppCommand, AppEvent, SideEffect};

pub fn reduce(state: &mut AppState, event: AppEvent) -> Vec<SideEffect> {
    match event {
        AppEvent::Command(command) => reduce_command(state, command),
        AppEvent::TerminalResize { width, height } => reduce_terminal_resize(state, width, height),
        AppEvent::GitRefreshRequested => reduce_git_refresh_requested(state),
        AppEvent::GitStatusUpdated { file_entries } => {
            reduce_git_status_updated(state, file_entries)
        }
        AppEvent::Quit => {
            state.dirty = true;
            vec![SideEffect::Quit]
        }
        AppEvent::ShellOutput(data) => reduce_shell_output(state, data),
        AppEvent::ShellExited(_code) => reduce_shell_exited(state),
        AppEvent::AgentOutput(data) => reduce_agent_output(state, data),
        AppEvent::AgentExited(code) => reduce_agent_exited(state, code),
        AppEvent::FileTreeChildrenLoaded {
            parent,
            children,
            error,
        } => reduce_file_tree_children_loaded(state, parent, children, error),
        AppEvent::PreviewLoaded { path, result } => reduce_preview_loaded(state, path, result),
        AppEvent::SearchCompleted {
            generation,
            results,
            truncated,
            error,
        } => reduce_search_completed(state, generation, results, truncated, error),
        AppEvent::EditorLaunched { path, command } => reduce_editor_launched(state, path, command),
        AppEvent::EditorLaunchFailed {
            path,
            error,
            show_modal,
        } => reduce_editor_launch_failed(state, path, error, show_modal),
        AppEvent::FsChanged { paths } => reduce_fs_changed(state, paths),
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
        AppCommand::FileTreeExpand(path) => reduce_file_tree_expand(state, path),
        AppCommand::FileTreeCollapse(path) => reduce_file_tree_collapse(state, path),
        AppCommand::FileTreeSelect(path) => reduce_file_tree_select(state, path),
        AppCommand::FileTreeRefresh => reduce_file_tree_refresh(state),
        AppCommand::FileTreeMoveSelection(delta) => reduce_file_tree_move_selection(state, delta),
        AppCommand::PreviewFile { path, line } => reduce_preview_file(state, path, line),
        AppCommand::PreviewScroll(delta) => reduce_preview_scroll(state, delta),
        AppCommand::PreviewPageScroll(delta) => reduce_preview_page_scroll(state, delta),
        AppCommand::SearchSetQuery(query) => reduce_search_set_query(state, query),
        AppCommand::SearchAppendChar(ch) => reduce_search_append_char(state, ch),
        AppCommand::SearchBackspace => reduce_search_backspace(state),
        AppCommand::SearchClear => reduce_search_clear(state),
        AppCommand::SearchSetMode(mode) => reduce_search_set_mode(state, mode),
        AppCommand::SearchExecute => reduce_search_execute(state),
        AppCommand::SearchCancel => reduce_search_cancel(state),
        AppCommand::SearchMoveSelection(delta) => reduce_search_move_selection(state, delta),
        AppCommand::SearchSelect(index) => reduce_search_select(state, index),
        AppCommand::OpenEditor { path, line } => reduce_open_editor(state, path, line),
        AppCommand::ModalDismiss => reduce_modal_dismiss(state),
    }
}

fn apply_navigation(state: &mut AppState, command: NavCommand) {
    let before = state.navigation.clone();
    state.navigation.apply(command);
    if state.navigation != before {
        state.dirty = true;
    }
    if state.navigation.left_tab == LeftNavTab::Files {
        state.file_tree.ensure_selection();
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

fn reduce_git_status_updated(
    state: &mut AppState,
    file_entries: Vec<GitFileEntry>,
) -> Vec<SideEffect> {
    let git_selected = state.git.selected_path.clone();
    let tree_selected = state.file_tree.selected.clone();

    state.git.file_entries = file_entries;

    if let Some(path) = git_selected {
        if state
            .git
            .file_entries
            .iter()
            .any(|entry| entry.path == path)
        {
            state.git.selected_path = Some(path);
        } else {
            state.git.selected_path = None;
        }
    }

    sync_git_statuses_to_file_tree(state);

    if let Some(path) = tree_selected {
        if state.file_tree.nodes.contains_key(&path) {
            state.file_tree.selected = Some(path);
        }
    }

    state.dirty = true;
    Vec::new()
}

fn sync_git_statuses_to_file_tree(state: &mut AppState) {
    let entries = state.git.file_entries.clone();
    state
        .file_tree
        .apply_git_statuses(&state.repo_root, &entries, state.config.git.show_untracked);
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

    if let Some(command_id) = state.palette.history_up() {
        state.palette.input = history_input_for_id(&command_id);
    }
    refresh_matches(state);
    state.dirty = true;
    Vec::new()
}

fn reduce_palette_history_down(state: &mut AppState) -> Vec<SideEffect> {
    if !state.palette.open {
        return Vec::new();
    }

    match state.palette.history_down() {
        None => {}
        Some(None) => state.palette.input.clear(),
        Some(Some(command_id)) => state.palette.input = history_input_for_id(&command_id),
    }
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

fn reduce_file_tree_expand(state: &mut AppState, path: PathBuf) -> Vec<SideEffect> {
    match state.file_tree.expand(&path) {
        Ok(ExpandAction::NeedsLoad) => {
            state.dirty = true;
            vec![SideEffect::LoadDirectoryChildren(path)]
        }
        Ok(ExpandAction::AlreadyExpanded) => {
            state.dirty = true;
            Vec::new()
        }
        Err(_) => {
            state.dirty = true;
            Vec::new()
        }
    }
}

fn reduce_file_tree_collapse(state: &mut AppState, path: PathBuf) -> Vec<SideEffect> {
    state.file_tree.collapse(&path);
    state.dirty = true;
    Vec::new()
}

fn reduce_file_tree_select(state: &mut AppState, path: PathBuf) -> Vec<SideEffect> {
    state.file_tree.select(path);
    state.dirty = true;
    Vec::new()
}

fn file_tree_viewport_rows(state: &AppState) -> usize {
    state.layout.rects.left_content.height.saturating_sub(2) as usize
}

fn reduce_file_tree_move_selection(state: &mut AppState, delta: i32) -> Vec<SideEffect> {
    state
        .file_tree
        .move_selection(delta, file_tree_viewport_rows(state));
    state.dirty = true;
    Vec::new()
}

fn reduce_file_tree_refresh(state: &mut AppState) -> Vec<SideEffect> {
    let expanded: Vec<PathBuf> = state
        .file_tree
        .nodes
        .values()
        .filter(|node| node.expanded)
        .map(|node| node.path.clone())
        .collect();

    for path in &expanded {
        state.file_tree.invalidate_children(path);
    }

    let mut effects = Vec::new();
    for path in expanded {
        if let Ok(ExpandAction::NeedsLoad) = state.file_tree.expand(&path) {
            effects.push(SideEffect::LoadDirectoryChildren(path));
        }
    }

    state.dirty = true;
    effects
}

fn reduce_file_tree_children_loaded(
    state: &mut AppState,
    parent: PathBuf,
    children: Vec<crate::file_tree::DirectoryEntry>,
    error: Option<String>,
) -> Vec<SideEffect> {
    state
        .file_tree
        .apply_children_loaded(&parent, children, error);
    sync_git_statuses_to_file_tree(state);
    state.dirty = true;
    Vec::new()
}

fn preview_viewport_rows(state: &AppState) -> usize {
    state.layout.rects.main_content.height.saturating_sub(4) as usize
}

fn reduce_preview_file(
    state: &mut AppState,
    path: PathBuf,
    line: Option<u32>,
) -> Vec<SideEffect> {
    state.preview.begin_load(path.clone(), line);
    state
        .navigation
        .apply(NavCommand::SelectMainTab(MainTab::Preview));
    state
        .navigation
        .apply(NavCommand::SetFocus(FocusTarget::Main));
    state.dirty = true;
    vec![SideEffect::LoadPreviewFile(path)]
}

fn reduce_preview_loaded(
    state: &mut AppState,
    path: PathBuf,
    result: crate::preview::PreviewLoadResult,
) -> Vec<SideEffect> {
    if state.preview.path.as_ref() != Some(&path) {
        return Vec::new();
    }

    state
        .preview
        .apply_loaded(path, result, preview_viewport_rows(state));
    state.dirty = true;
    Vec::new()
}

fn reduce_preview_scroll(state: &mut AppState, delta: i32) -> Vec<SideEffect> {
    if delta == 0 {
        return Vec::new();
    }

    state.preview.scroll(delta, preview_viewport_rows(state));
    state.dirty = true;
    Vec::new()
}

fn reduce_preview_page_scroll(state: &mut AppState, delta: i32) -> Vec<SideEffect> {
    if delta == 0 {
        return Vec::new();
    }

    state
        .preview
        .page_scroll(delta, preview_viewport_rows(state));
    state.dirty = true;
    Vec::new()
}

fn reduce_search_set_query(state: &mut AppState, query: String) -> Vec<SideEffect> {
    state.search.schedule_query(query);
    state.dirty = true;
    vec![SideEffect::CancelSearch]
}

fn reduce_search_append_char(state: &mut AppState, ch: char) -> Vec<SideEffect> {
    let mut query = state.search.query.clone();
    query.push(ch);
    reduce_search_set_query(state, query)
}

fn reduce_search_backspace(state: &mut AppState) -> Vec<SideEffect> {
    if state.search.query.is_empty() {
        return Vec::new();
    }

    let mut query = state.search.query.clone();
    query.pop();
    reduce_search_set_query(state, query)
}

fn reduce_search_clear(state: &mut AppState) -> Vec<SideEffect> {
    state.search.clear_query();
    state.dirty = true;
    vec![SideEffect::CancelSearch]
}

fn reduce_search_set_mode(
    state: &mut AppState,
    mode: crate::search::SearchMode,
) -> Vec<SideEffect> {
    state.search.set_mode(mode);
    state.dirty = true;
    vec![SideEffect::CancelSearch]
}

fn reduce_search_execute(state: &mut AppState) -> Vec<SideEffect> {
    let generation = state.search.begin_execute();
    state.dirty = true;

    if state.search.query.is_empty() {
        return vec![SideEffect::CancelSearch];
    }

    vec![SideEffect::RunSearch {
        mode: state.search.mode,
        query: state.search.query.clone(),
        generation,
    }]
}

fn reduce_search_cancel(state: &mut AppState) -> Vec<SideEffect> {
    state.search.cancel();
    state.dirty = true;
    vec![SideEffect::CancelSearch]
}

fn search_viewport_rows(state: &AppState) -> usize {
    state.layout.rects.left_content.height.saturating_sub(4) as usize
}

fn reduce_search_move_selection(state: &mut AppState, delta: i32) -> Vec<SideEffect> {
    if delta == 0 {
        return Vec::new();
    }

    state
        .search
        .move_selection(delta, search_viewport_rows(state));
    state.dirty = true;
    Vec::new()
}

fn reduce_search_select(state: &mut AppState, index: usize) -> Vec<SideEffect> {
    state
        .search
        .select_index(index, search_viewport_rows(state));
    state.dirty = true;
    Vec::new()
}

fn reduce_search_completed(
    state: &mut AppState,
    generation: u64,
    results: Vec<crate::search::SearchResult>,
    truncated: bool,
    error: Option<String>,
) -> Vec<SideEffect> {
    if generation != state.search.generation {
        return Vec::new();
    }

    if let Some(message) = error {
        state.search.apply_error(generation, message);
    } else {
        state.search.apply_results(generation, results, truncated);
    }
    state.dirty = true;
    Vec::new()
}

fn reduce_open_editor(state: &mut AppState, path: PathBuf, line: Option<u32>) -> Vec<SideEffect> {
    state.dirty = true;
    vec![SideEffect::LaunchEditor { path, line }]
}

fn reduce_modal_dismiss(state: &mut AppState) -> Vec<SideEffect> {
    if state.notifications.modal.is_some() {
        state.notifications.dismiss_modal();
        state.dirty = true;
    }
    Vec::new()
}

fn reduce_editor_launched(state: &mut AppState, path: PathBuf, command: String) -> Vec<SideEffect> {
    state.logs.push_info(format!(
        "Launched editor `{command}` for {}",
        path.display()
    ));
    state.dirty = true;
    Vec::new()
}

fn reduce_editor_launch_failed(
    state: &mut AppState,
    path: PathBuf,
    error: String,
    show_modal: bool,
) -> Vec<SideEffect> {
    state.logs.push_error(format!(
        "Editor launch failed for {}: {error}",
        path.display()
    ));
    if show_modal {
        state.notifications.show_modal(
            "Editor command not found",
            format!("{error}\n\nPress Esc to dismiss."),
        );
    } else {
        state.notifications.show_toast(error);
    }
    state.dirty = true;
    Vec::new()
}

fn reduce_fs_changed(state: &mut AppState, paths: Vec<PathBuf>) -> Vec<SideEffect> {
    let Some(preview_path) = state.preview.path.clone() else {
        return Vec::new();
    };

    if state.preview.loading {
        return Vec::new();
    }

    if !crate::watcher::preview_reload_paths(&paths, &preview_path) {
        return Vec::new();
    }

    state.preview.begin_reload();
    state.dirty = true;
    vec![SideEffect::LoadPreviewFile(preview_path)]
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::agent::AgentStatus;
    use crate::config::ResolvedConfig;
    use crate::git::{GitFileEntry, GitFileStatus};
    use crate::layout::compute_layout;
    use crate::layout::FocusTarget;
    use crate::navigation::{LeftNavTab, MainTab, NavCommand, NavigationState};
    use crate::theme::capabilities::TerminalCapabilities;
    use crate::theme::loader::load_theme_with_capabilities;

    use super::*;
    use crate::file_tree::{DirectoryEntry, FileTreeState};
    use crate::preview::{PreviewLoadResult, PreviewState};
    use crate::search::{SearchMode, SearchResult, SearchState};
    use crate::state::domains::{
        AgentState, CommandPaletteState, DiffState, GitHubState, GitState, ShellState,
        StatusBarState, WorkspaceMeta,
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
            file_tree: FileTreeState::at_root(PathBuf::from(".")),
            preview: PreviewState::default(),
            search: SearchState::default(),
            git: GitState::default(),
            diff: DiffState::default(),
            github: GitHubState::default(),
            agent: AgentState::default(),
            shell: ShellState::default(),
            palette: CommandPaletteState::default(),
            logs: crate::state::domains::LogsState::default(),
            notifications: crate::state::domains::NotificationState::default(),
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
        state.git.file_entries = vec![
            GitFileEntry {
                path: "src/main.rs".to_string(),
                status: GitFileStatus::Modified,
            },
            GitFileEntry {
                path: "other.rs".to_string(),
                status: GitFileStatus::Modified,
            },
        ];

        reduce(
            &mut state,
            AppEvent::GitStatusUpdated {
                file_entries: vec![
                    GitFileEntry {
                        path: "src/main.rs".to_string(),
                        status: GitFileStatus::Modified,
                    },
                    GitFileEntry {
                        path: "new.rs".to_string(),
                        status: GitFileStatus::Added,
                    },
                ],
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
                file_entries: vec![GitFileEntry {
                    path: "other.rs".to_string(),
                    status: GitFileStatus::Modified,
                }],
            },
        );

        assert_eq!(state.git.selected_path, None);
    }

    #[test]
    fn git_status_refresh_preserves_file_tree_selection() {
        let mut state = test_state();
        let selected = state.file_tree.root.join("src/main.rs");
        state.file_tree.nodes.insert(
            selected.clone(),
            crate::file_tree::FileNode {
                path: selected.clone(),
                name: "main.rs".to_string(),
                is_dir: false,
                expanded: false,
                children_loaded: true,
                load_error: None,
                git_status: None,
            },
        );
        state.file_tree.selected = Some(selected.clone());
        reduce(
            &mut state,
            AppEvent::GitStatusUpdated {
                file_entries: vec![GitFileEntry {
                    path: "src/main.rs".to_string(),
                    status: GitFileStatus::Modified,
                }],
            },
        );
        assert_eq!(state.file_tree.selected, Some(selected));
        assert_eq!(
            state.file_tree.nodes[&state.file_tree.root.join("src/main.rs")].git_status,
            Some(GitFileStatus::Modified)
        );
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

    #[test]
    fn palette_execute_persists_history_when_enabled() {
        let mut state = test_state();
        reduce(&mut state, AppEvent::Command(AppCommand::PaletteOpen));
        let effects = reduce(
            &mut state,
            AppEvent::Command(AppCommand::PaletteExecuteSelected),
        );
        assert!(effects.contains(&SideEffect::SavePaletteHistory));
    }

    #[test]
    fn palette_history_up_uses_command_title() {
        let mut state = test_state();
        state.palette.history = vec!["git.refresh".to_string()];
        reduce(&mut state, AppEvent::Command(AppCommand::PaletteOpen));
        reduce(&mut state, AppEvent::Command(AppCommand::PaletteHistoryUp));
        assert_eq!(state.palette.input, "Git: Refresh Status");
    }

    #[test]
    fn startup_file_tree_contains_root_only() {
        let state = test_state();
        assert_eq!(state.file_tree.nodes.len(), 1);
        assert!(state.file_tree.children.is_empty());
    }

    #[test]
    fn file_tree_expand_emits_load_side_effect() {
        let mut state = test_state();
        let root = state.file_tree.root.clone();
        let effects = reduce(
            &mut state,
            AppEvent::Command(AppCommand::FileTreeExpand(root.clone())),
        );
        assert!(effects.contains(&SideEffect::LoadDirectoryChildren(root)));
        assert!(state.file_tree.loading.contains(&state.file_tree.root));
    }

    #[test]
    fn file_tree_children_loaded_updates_state() {
        let mut state = test_state();
        let root = state.file_tree.root.clone();
        reduce(
            &mut state,
            AppEvent::Command(AppCommand::FileTreeExpand(root.clone())),
        );
        reduce(
            &mut state,
            AppEvent::FileTreeChildrenLoaded {
                parent: root.clone(),
                children: vec![DirectoryEntry {
                    path: root.join("src"),
                    name: "src".to_string(),
                    is_dir: true,
                }],
                error: None,
            },
        );
        assert_eq!(state.file_tree.children[&root].len(), 1);
        assert!(state.file_tree.nodes.contains_key(&root.join("src")));
    }

    #[test]
    fn file_tree_move_selection_changes_selected_row() {
        let mut state = test_state();
        let root = state.file_tree.root.clone();
        reduce(
            &mut state,
            AppEvent::Command(AppCommand::FileTreeExpand(root.clone())),
        );
        reduce(
            &mut state,
            AppEvent::FileTreeChildrenLoaded {
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
            },
        );
        reduce(
            &mut state,
            AppEvent::Command(AppCommand::FileTreeMoveSelection(1)),
        );
        assert_eq!(state.file_tree.selected, Some(root.join("src")));
    }

    #[test]
    fn preview_file_emits_load_side_effect_and_switches_tab() {
        let mut state = test_state();
        let path = PathBuf::from("src/main.rs");
        let effects = reduce(
            &mut state,
            AppEvent::Command(AppCommand::PreviewFile {
                path: path.clone(),
                line: None,
            }),
        );
        assert!(effects.contains(&SideEffect::LoadPreviewFile(path)));
        assert_eq!(state.navigation.main_tab, MainTab::Preview);
        assert_eq!(state.navigation.focus, FocusTarget::Main);
        assert!(state.preview.loading);
    }

    #[test]
    fn preview_loaded_applies_content() {
        let mut state = test_state();
        let path = PathBuf::from("README.md");
        reduce(
            &mut state,
            AppEvent::Command(AppCommand::PreviewFile {
                path: path.clone(),
                line: None,
            }),
        );
        reduce(
            &mut state,
            AppEvent::PreviewLoaded {
                path: path.clone(),
                result: PreviewLoadResult {
                    lines: vec!["hello".to_string()],
                    truncated: false,
                    oversize: false,
                    binary: false,
                    lossy_utf8: false,
                    file_size: 5,
                    error: None,
                },
            },
        );
        assert!(!state.preview.loading);
        assert_eq!(state.preview.lines, vec!["hello".to_string()]);
    }

    #[test]
    fn preview_file_scrolls_to_requested_line() {
        let mut state = test_state();
        let path = PathBuf::from("/tmp/repo/src/main.rs");
        reduce(
            &mut state,
            AppEvent::Command(AppCommand::PreviewFile {
                path: path.clone(),
                line: Some(25),
            }),
        );
        reduce(
            &mut state,
            AppEvent::PreviewLoaded {
                path: path.clone(),
                result: PreviewLoadResult {
                    lines: (1..=50).map(|index| format!("line {index}")).collect(),
                    truncated: false,
                    oversize: false,
                    binary: false,
                    lossy_utf8: false,
                    file_size: 1,
                    error: None,
                },
            },
        );
        assert_eq!(state.preview.scroll_offset, 24);
        assert_eq!(state.navigation.main_tab, MainTab::Preview);
    }

    #[test]
    fn search_selection_preview_command_includes_content_line() {
        let mut state = test_state();
        let path = PathBuf::from("src/main.rs");
        state.search.results = vec![SearchResult::content(
            path.clone(),
            12,
            "fn main()".to_string(),
        )];
        state.search.selected = 0;

        let effects = reduce(
            &mut state,
            AppEvent::Command(AppCommand::PreviewFile {
                path: path.clone(),
                line: Some(12),
            }),
        );

        assert!(effects.contains(&SideEffect::LoadPreviewFile(path)));
        assert_eq!(state.navigation.main_tab, MainTab::Preview);
    }

    #[test]
    fn preview_scroll_clamps_within_file() {
        let mut state = test_state();
        state.preview.lines = (0..100).map(|index| format!("line {index}")).collect();
        reduce(
            &mut state,
            AppEvent::Command(AppCommand::PreviewScroll(200)),
        );
        assert!(state.preview.scroll_offset > 0);
        reduce(
            &mut state,
            AppEvent::Command(AppCommand::PreviewScroll(-500)),
        );
        assert_eq!(state.preview.scroll_offset, 0);
    }

    #[test]
    fn search_set_query_schedules_debounce_and_cancels() {
        let mut state = test_state();
        let effects = reduce(
            &mut state,
            AppEvent::Command(AppCommand::SearchSetQuery("main".to_string())),
        );
        assert!(effects.contains(&SideEffect::CancelSearch));
        assert!(state.search.debounce_scheduled);
        assert_eq!(state.search.generation, 1);
    }

    #[test]
    fn search_execute_emits_run_side_effect() {
        let mut state = test_state();
        state.search.query = "main".to_string();
        state.search.generation = 1;
        let effects = reduce(&mut state, AppEvent::Command(AppCommand::SearchExecute));
        assert!(effects.iter().any(|effect| matches!(
            effect,
            SideEffect::RunSearch {
                mode: SearchMode::Files,
                query,
                generation: 1,
            } if query == "main"
        )));
        assert!(state.search.running);
    }

    #[test]
    fn search_completed_ignores_stale_generation() {
        let mut state = test_state();
        state.search.generation = 2;
        reduce(
            &mut state,
            AppEvent::SearchCompleted {
                generation: 1,
                results: vec![],
                truncated: false,
                error: None,
            },
        );
        assert!(state.search.results.is_empty());
    }

    #[test]
    fn search_clear_cancels_running_query() {
        let mut state = test_state();
        state.search.query = "abc".to_string();
        state.search.running = true;
        let effects = reduce(&mut state, AppEvent::Command(AppCommand::SearchClear));
        assert!(effects.contains(&SideEffect::CancelSearch));
        assert!(state.search.query.is_empty());
        assert!(!state.search.running);
    }

    #[test]
    fn open_editor_emits_launch_side_effect() {
        let mut state = test_state();
        let path = PathBuf::from("src/main.rs");
        let effects = reduce(
            &mut state,
            AppEvent::Command(AppCommand::OpenEditor {
                path: path.clone(),
                line: None,
            }),
        );
        assert!(effects.contains(&SideEffect::LaunchEditor { path, line: None }));
    }

    #[test]
    fn editor_launched_records_log_entry() {
        let mut state = test_state();
        reduce(
            &mut state,
            AppEvent::EditorLaunched {
                path: PathBuf::from("/tmp/a.rs"),
                command: "nvim".to_string(),
            },
        );
        assert_eq!(state.logs.entries.len(), 1);
        assert!(state.logs.entries[0].message.contains("nvim"));
    }

    #[test]
    fn editor_launch_failed_shows_modal_for_missing_command() {
        let mut state = test_state();
        reduce(
            &mut state,
            AppEvent::EditorLaunchFailed {
                path: PathBuf::from("/tmp/a.rs"),
                error: "Editor command not found: missing".to_string(),
                show_modal: true,
            },
        );
        assert!(state.notifications.modal.is_some());
        assert_eq!(state.logs.entries.len(), 1);
    }

    #[test]
    fn modal_dismiss_clears_modal_state() {
        let mut state = test_state();
        state.notifications.show_modal("Title", "Message");
        reduce(&mut state, AppEvent::Command(AppCommand::ModalDismiss));
        assert!(state.notifications.modal.is_none());
    }

    #[test]
    fn fs_changed_reloads_matching_preview_file() {
        let mut state = test_state();
        let path = PathBuf::from("/tmp/repo/src/main.rs");
        state.preview.path = Some(path.clone());
        state.preview.lines = vec!["old".to_string()];
        state.preview.scroll_offset = 5;

        let effects = reduce(
            &mut state,
            AppEvent::FsChanged {
                paths: vec![PathBuf::from("/tmp/repo/src/other.rs"), path.clone()],
            },
        );

        assert!(effects.contains(&SideEffect::LoadPreviewFile(path)));
        assert!(state.preview.loading);
        assert!(state.preview.preserve_scroll_on_load);
        assert_eq!(state.preview.scroll_offset, 5);
        assert_eq!(state.navigation.main_tab, MainTab::Agent);
    }

    #[test]
    fn fs_changed_ignores_unrelated_paths_and_in_flight_loads() {
        let mut state = test_state();
        let path = PathBuf::from("/tmp/repo/src/main.rs");
        state.preview.path = Some(path.clone());
        state.preview.loading = true;

        let effects = reduce(
            &mut state,
            AppEvent::FsChanged {
                paths: vec![PathBuf::from("/tmp/repo/src/other.rs")],
            },
        );
        assert!(effects.is_empty());

        state.preview.loading = false;
        let effects = reduce(
            &mut state,
            AppEvent::FsChanged {
                paths: vec![PathBuf::from("/tmp/repo/src/other.rs")],
            },
        );
        assert!(effects.is_empty());
    }
}
