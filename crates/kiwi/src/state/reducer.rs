use std::path::PathBuf;

use crate::agent::infer_status_from_scrollback;
use crate::clipboard::{resolve_copy_text, PasteTarget};
use crate::commands::{execute_command, history_input_for_id, refresh_matches};
use crate::file_tree::ExpandAction;
use crate::git::{
    ensure_git_selection, git_move_selection, git_select_row, patch_git_file_entries, GitFileEntry,
};
use crate::layout::{agent_pty_size, compute_layout, shell_pty_size, FocusTarget};
use crate::navigation::{LeftNavTab, MainTab, NavCommand};

use super::app_state::AppState;
use super::event::{AppCommand, AppEvent, SideEffect};

pub fn reduce(state: &mut AppState, event: AppEvent) -> Vec<SideEffect> {
    match event {
        AppEvent::Command(command) => reduce_command(state, command),
        AppEvent::TerminalResize { width, height } => reduce_terminal_resize(state, width, height),
        AppEvent::GitRefreshRequested => reduce_git_refresh_requested(state),
        AppEvent::GitStatusUpdated {
            branch,
            ahead,
            behind,
            file_entries,
            error,
        } => reduce_git_status_updated(state, branch, ahead, behind, file_entries, error),
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
        AppCommand::GitMoveSelection(delta) => reduce_git_move_selection(state, delta),
        AppCommand::GitSelect(index) => reduce_git_select(state, index),
        AppCommand::GitOpenSelected => reduce_git_open_selected(state),
        AppCommand::GitRefresh => reduce_git_refresh_requested(state),
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
        AppCommand::ClipboardCopy => reduce_clipboard_copy(state),
        AppCommand::ClipboardCut => reduce_clipboard_cut(state),
        AppCommand::ClipboardPaste => reduce_clipboard_paste(state),
        AppCommand::PasteText(text) => reduce_paste_text(state, text),
        AppCommand::SelectionBegin { pane, line, col } => {
            reduce_selection_begin(state, pane, line, col)
        }
        AppCommand::SelectionExtend { line, col } => reduce_selection_extend(state, line, col),
        AppCommand::SelectionEnd => reduce_selection_end(state),
        AppCommand::SelectionClear => reduce_selection_clear(state),
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
    if state.navigation.left_tab == LeftNavTab::Git {
        ensure_git_selection(&mut state.git, state.config.git.show_untracked);
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

pub fn git_refresh_effects(state: &mut AppState) -> Vec<SideEffect> {
    state.dirty = true;
    if !state.workspace_meta.is_git_repo {
        return Vec::new();
    }

    state.git.loading = true;
    vec![SideEffect::SpawnGitRefresh]
}

fn reduce_git_refresh_requested(state: &mut AppState) -> Vec<SideEffect> {
    git_refresh_effects(state)
}

fn reduce_git_status_updated(
    state: &mut AppState,
    branch: Option<String>,
    ahead: u32,
    behind: u32,
    file_entries: Vec<GitFileEntry>,
    error: Option<String>,
) -> Vec<SideEffect> {
    let git_selected = state.git.selected_path.clone();
    let tree_selected = state.file_tree.selected.clone();

    state.git.loading = false;
    state.git.ahead = ahead;
    state.git.behind = behind;

    if let Some(message) = error {
        state.git.error = Some(message.clone());
        state
            .logs
            .push_error(format!("git refresh failed: {message}"));
    } else {
        state.git.error = None;
        state.git.branch = branch;
        let file_patch = patch_git_file_entries(&mut state.git.file_entries, &file_entries);
        sync_git_status_patch_to_file_tree(state, &file_patch);
        ensure_git_selection(&mut state.git, state.config.git.show_untracked);
    }

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

    if let Some(path) = tree_selected {
        if state.file_tree.nodes.contains_key(&path) {
            state.file_tree.selected = Some(path);
        }
    }

    state.dirty = true;
    Vec::new()
}

fn sync_git_status_patch_to_file_tree(
    state: &mut AppState,
    patch: &crate::git::GitFileStatusPatch,
) {
    if patch.is_empty() {
        return;
    }

    state.file_tree.apply_git_status_patch(
        &state.repo_root,
        patch,
        state.config.git.show_untracked,
    );
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

fn git_viewport_rows(state: &AppState) -> usize {
    crate::ui::git_viewport_rows(state.layout.rects.left_content)
}

fn reduce_git_move_selection(state: &mut AppState, delta: i32) -> Vec<SideEffect> {
    if delta == 0 {
        return Vec::new();
    }

    let viewport_rows = git_viewport_rows(state);
    let show_untracked = state.config.git.show_untracked;
    git_move_selection(&mut state.git, delta, viewport_rows, show_untracked);
    state.dirty = true;
    Vec::new()
}

fn reduce_git_select(state: &mut AppState, index: usize) -> Vec<SideEffect> {
    let viewport_rows = git_viewport_rows(state);
    let show_untracked = state.config.git.show_untracked;
    git_select_row(&mut state.git, index, viewport_rows, show_untracked);
    state.dirty = true;
    Vec::new()
}

fn reduce_git_open_selected(state: &mut AppState) -> Vec<SideEffect> {
    let Some(path) = state.git.selected_path.clone() else {
        return Vec::new();
    };

    state.diff.selected_path = Some(path);
    apply_navigation(state, NavCommand::SelectMainTab(MainTab::Diff));
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

fn reduce_preview_file(state: &mut AppState, path: PathBuf, line: Option<u32>) -> Vec<SideEffect> {
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

fn reduce_clipboard_copy(state: &mut AppState) -> Vec<SideEffect> {
    let Some(text) = resolve_copy_text(state) else {
        state.notifications.show_toast("Nothing to copy");
        state.dirty = true;
        return Vec::new();
    };

    state.dirty = true;
    vec![SideEffect::CopyToClipboard(text)]
}

fn reduce_clipboard_cut(state: &mut AppState) -> Vec<SideEffect> {
    let Some(text) = resolve_copy_text(state) else {
        state.notifications.show_toast("Nothing to cut");
        state.dirty = true;
        return Vec::new();
    };

    apply_cut_mutation(state);
    state.dirty = true;
    vec![SideEffect::CopyToClipboard(text)]
}

fn reduce_clipboard_paste(state: &mut AppState) -> Vec<SideEffect> {
    state.dirty = true;
    vec![SideEffect::PasteFromClipboard]
}

fn reduce_paste_text(state: &mut AppState, text: String) -> Vec<SideEffect> {
    if text.is_empty() {
        return Vec::new();
    }

    state.dirty = true;
    match crate::clipboard::resolve_paste_target(state) {
        PasteTarget::PaletteInput => {
            state.palette.history_cursor = None;
            state.palette.input.push_str(&text);
            refresh_matches(state);
            Vec::new()
        }
        PasteTarget::SearchQuery => {
            let mut query = state.search.query.clone();
            query.push_str(&text);
            reduce_search_set_query(state, query)
        }
        PasteTarget::ShellPty => vec![SideEffect::WriteShell(crate::clipboard::pty_paste_bytes(
            &text,
        ))],
        PasteTarget::AgentPty => vec![SideEffect::WriteAgent(crate::clipboard::pty_paste_bytes(
            &text,
        ))],
        PasteTarget::Unsupported => {
            state
                .notifications
                .show_toast("Paste is not supported in this pane");
            Vec::new()
        }
    }
}

fn reduce_selection_begin(
    state: &mut AppState,
    pane: crate::selection::SelectionPane,
    line: usize,
    col: usize,
) -> Vec<SideEffect> {
    state
        .text_selection
        .begin(pane, crate::selection::TextPosition { line, col });
    state.dirty = true;
    Vec::new()
}

fn reduce_selection_extend(state: &mut AppState, line: usize, col: usize) -> Vec<SideEffect> {
    if state.text_selection.dragging {
        state
            .text_selection
            .extend(crate::selection::TextPosition { line, col });
        state.dirty = true;
    }
    Vec::new()
}

fn reduce_selection_end(state: &mut AppState) -> Vec<SideEffect> {
    state.text_selection.end_drag();
    state.dirty = true;
    Vec::new()
}

fn reduce_selection_clear(state: &mut AppState) -> Vec<SideEffect> {
    if state.text_selection.pane.is_some() {
        state.text_selection.clear();
        state.dirty = true;
    }
    Vec::new()
}

fn apply_cut_mutation(state: &mut AppState) {
    if state.palette.open {
        state.palette.input.clear();
        refresh_matches(state);
        return;
    }

    if state.navigation.focus == FocusTarget::Left
        && state.navigation.left_tab == LeftNavTab::Search
        && state.search.results.is_empty()
    {
        state.search.query.clear();
    }
}

fn reduce_fs_changed(state: &mut AppState, paths: Vec<PathBuf>) -> Vec<SideEffect> {
    let mut effects = Vec::new();

    let reload_dirs = state
        .file_tree
        .apply_fs_invalidation(&state.repo_root, &paths);
    if !reload_dirs.is_empty() {
        state.dirty = true;
        for dir in reload_dirs {
            effects.push(SideEffect::LoadDirectoryChildren(dir));
        }
    }

    if let Some(preview_path) = state.preview.path.clone() {
        if !state.preview.loading
            && crate::watcher::preview_reload_paths(&paths, &preview_path)
            && !preview_file_unchanged(&preview_path, state.preview.loaded_mtime)
        {
            state.preview.begin_reload();
            state.dirty = true;
            effects.push(SideEffect::LoadPreviewFile(preview_path));
        }
    }

    if state.workspace_meta.is_git_repo && state.config.git.watch {
        effects.extend(reduce_git_refresh_requested(state));
    }

    effects
}

fn preview_file_unchanged(
    path: &std::path::Path,
    loaded_mtime: Option<std::time::SystemTime>,
) -> bool {
    let Some(loaded_mtime) = loaded_mtime else {
        return false;
    };
    std::fs::metadata(path)
        .ok()
        .and_then(|metadata| metadata.modified().ok())
        .is_some_and(|modified| modified == loaded_mtime)
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
            text_selection: crate::selection::TextSelection::default(),
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
                branch: None,
                ahead: 0,
                behind: 0,
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
                error: None,
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
                branch: None,
                ahead: 0,
                behind: 0,
                file_entries: vec![GitFileEntry {
                    path: "other.rs".to_string(),
                    status: GitFileStatus::Modified,
                }],
                error: None,
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
                branch: None,
                ahead: 0,
                behind: 0,
                file_entries: vec![GitFileEntry {
                    path: "src/main.rs".to_string(),
                    status: GitFileStatus::Modified,
                }],
                error: None,
            },
        );
        assert_eq!(state.file_tree.selected, Some(selected));
        assert_eq!(
            state.file_tree.nodes[&state.file_tree.root.join("src/main.rs")].git_status,
            Some(GitFileStatus::Modified)
        );
    }

    #[test]
    fn git_refresh_requested_emits_side_effect_for_git_repo() {
        let mut state = test_state();
        state.workspace_meta.is_git_repo = true;
        let effects = reduce(&mut state, AppEvent::GitRefreshRequested);
        assert!(effects.contains(&SideEffect::SpawnGitRefresh));
        assert!(state.git.loading);
    }

    #[test]
    fn git_refresh_requested_skips_non_git_repo() {
        let mut state = test_state();
        state.workspace_meta.is_git_repo = false;
        let effects = reduce(&mut state, AppEvent::GitRefreshRequested);
        assert!(effects.is_empty());
        assert!(!state.git.loading);
    }

    #[test]
    fn git_status_updated_sets_branch_and_tracking_counts() {
        let mut state = test_state();
        reduce(
            &mut state,
            AppEvent::GitStatusUpdated {
                branch: Some("feature/42".to_string()),
                ahead: 2,
                behind: 1,
                file_entries: Vec::new(),
                error: None,
            },
        );

        assert_eq!(state.git.branch.as_deref(), Some("feature/42"));
        assert_eq!(state.git.ahead, 2);
        assert_eq!(state.git.behind, 1);
        assert!(!state.git.loading);
        assert!(state.git.error.is_none());
    }

    #[test]
    fn git_status_updated_records_error_without_branch() {
        let mut state = test_state();
        state.git.branch = Some("main".to_string());

        reduce(
            &mut state,
            AppEvent::GitStatusUpdated {
                branch: None,
                ahead: 0,
                behind: 0,
                file_entries: Vec::new(),
                error: Some("corrupt".to_string()),
            },
        );

        assert_eq!(state.git.branch, Some("main".to_string()));
        assert_eq!(state.git.error.as_deref(), Some("corrupt"));
        assert_eq!(state.logs.entries.len(), 1);
    }

    #[test]
    fn git_status_updated_applies_incremental_file_patch() {
        let mut state = test_state();
        state.git.file_entries = vec![GitFileEntry {
            path: "src/main.rs".to_string(),
            status: GitFileStatus::Modified,
        }];

        reduce(
            &mut state,
            AppEvent::GitStatusUpdated {
                branch: Some("main".to_string()),
                ahead: 0,
                behind: 0,
                file_entries: vec![
                    GitFileEntry {
                        path: "src/main.rs".to_string(),
                        status: GitFileStatus::Modified,
                    },
                    GitFileEntry {
                        path: "src/new.rs".to_string(),
                        status: GitFileStatus::Added,
                    },
                ],
                error: None,
            },
        );

        assert_eq!(state.git.file_entries.len(), 2);
        assert!(state
            .git
            .file_entries
            .iter()
            .any(|entry| entry.path == "src/new.rs"));
    }

    #[test]
    fn fs_changed_many_paths_trigger_single_git_refresh() {
        let mut state = test_state();
        state.workspace_meta.is_git_repo = true;
        state.config.git.watch = true;

        let paths: Vec<PathBuf> = (0..50)
            .map(|index| PathBuf::from(format!("/repo/file{index}.rs")))
            .collect();
        let effects = reduce(&mut state, AppEvent::FsChanged { paths });

        assert_eq!(
            effects
                .iter()
                .filter(|effect| **effect == SideEffect::SpawnGitRefresh)
                .count(),
            1
        );
        assert!(state.git.loading);
    }

    #[test]
    fn fs_changed_git_head_triggers_refresh() {
        let mut state = test_state();
        state.workspace_meta.is_git_repo = true;
        state.config.git.watch = true;

        let effects = reduce(
            &mut state,
            AppEvent::FsChanged {
                paths: vec![PathBuf::from("/repo/.git/HEAD")],
            },
        );

        assert!(effects.contains(&SideEffect::SpawnGitRefresh));
        assert!(state.git.loading);
    }

    #[test]
    fn fs_changed_requests_git_refresh_when_watch_enabled() {
        let mut state = test_state();
        state.workspace_meta.is_git_repo = true;
        state.config.git.watch = true;

        let effects = reduce(
            &mut state,
            AppEvent::FsChanged {
                paths: vec![PathBuf::from("/tmp/repo/src/main.rs")],
            },
        );

        assert!(effects.contains(&SideEffect::SpawnGitRefresh));
        assert!(state.git.loading);
    }

    #[test]
    fn fs_changed_skips_git_refresh_when_watch_disabled() {
        let mut state = test_state();
        state.workspace_meta.is_git_repo = true;
        state.config.git.watch = false;

        let effects = reduce(
            &mut state,
            AppEvent::FsChanged {
                paths: vec![PathBuf::from("/tmp/repo/src/main.rs")],
            },
        );

        assert!(!effects.contains(&SideEffect::SpawnGitRefresh));
    }

    #[test]
    fn request_git_refresh_sets_loading() {
        let mut state = test_state();
        state.workspace_meta.is_git_repo = true;
        let effects = reduce(&mut state, AppEvent::Command(AppCommand::RequestGitRefresh));

        assert!(effects.contains(&SideEffect::SpawnGitRefresh));
        assert!(state.git.loading);
    }

    #[test]
    fn git_refresh_command_emits_side_effect() {
        let mut state = test_state();
        state.workspace_meta.is_git_repo = true;
        let effects = reduce(&mut state, AppEvent::Command(AppCommand::GitRefresh));
        assert!(effects.contains(&SideEffect::SpawnGitRefresh));
    }

    #[test]
    fn git_open_selected_switches_to_main_diff_tab() {
        let mut state = test_state();
        state.git.selected_path = Some("src/main.rs".to_string());

        reduce(&mut state, AppEvent::Command(AppCommand::GitOpenSelected));

        assert_eq!(state.navigation.main_tab, MainTab::Diff);
        assert_eq!(state.diff.selected_path.as_deref(), Some("src/main.rs"));
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
    fn palette_execute_git_refresh_sets_loading() {
        let mut state = test_state();
        state.workspace_meta.is_git_repo = true;
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
        let effects = reduce(
            &mut state,
            AppEvent::Command(AppCommand::PaletteExecuteSelected),
        );

        assert!(effects.contains(&SideEffect::SpawnGitRefresh));
        assert!(state.git.loading);
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
                    modified_at: None,
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
                    modified_at: None,
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
    fn clipboard_paste_into_agent_emits_write_side_effect() {
        let mut state = test_state();
        state.agent.running = true;
        state
            .navigation
            .apply(NavCommand::SelectMainTab(MainTab::Agent));
        state
            .navigation
            .apply(NavCommand::SetFocus(FocusTarget::Main));
        let effects = reduce(
            &mut state,
            AppEvent::Command(AppCommand::PasteText("hello".to_string())),
        );
        assert!(effects.iter().any(|effect| {
            matches!(
                effect,
                SideEffect::WriteAgent(bytes) if bytes == b"hello"
            )
        }));
    }

    #[test]
    fn clipboard_paste_multiline_into_agent_uses_bracketed_paste() {
        let mut state = test_state();
        state.agent.running = true;
        state
            .navigation
            .apply(NavCommand::SelectMainTab(MainTab::Agent));
        state
            .navigation
            .apply(NavCommand::SetFocus(FocusTarget::Main));
        let effects = reduce(
            &mut state,
            AppEvent::Command(AppCommand::PasteText("line\n".to_string())),
        );
        assert!(effects.iter().any(|effect| {
            matches!(
                effect,
                SideEffect::WriteAgent(bytes) if bytes.starts_with(b"\x1b[200~")
            )
        }));
    }

    #[test]
    fn clipboard_copy_prefers_mouse_selection() {
        use crate::selection::{SelectionPane, TextPosition, TextSelection};

        let mut state = test_state();
        state.preview.lines = vec!["selected text".to_string()];
        state.text_selection = TextSelection {
            pane: Some(SelectionPane::Preview),
            anchor: TextPosition { line: 0, col: 0 },
            cursor: TextPosition { line: 0, col: 8 },
            dragging: false,
        };
        state
            .navigation
            .apply(NavCommand::SelectMainTab(MainTab::Preview));

        let effects = reduce(&mut state, AppEvent::Command(AppCommand::ClipboardCopy));
        assert!(effects.contains(&SideEffect::CopyToClipboard("selected".to_string())));
    }

    #[test]
    fn clipboard_copy_emits_side_effect_for_preview_line() {
        let mut state = test_state();
        state.preview.lines = vec!["line one".to_string()];
        state
            .navigation
            .apply(NavCommand::SelectMainTab(MainTab::Preview));
        let effects = reduce(&mut state, AppEvent::Command(AppCommand::ClipboardCopy));
        assert!(effects.contains(&SideEffect::CopyToClipboard("line one".to_string())));
    }

    #[test]
    fn clipboard_paste_into_search_query_schedules_search() {
        let mut state = test_state();
        state
            .navigation
            .apply(NavCommand::SetFocus(FocusTarget::Left));
        state
            .navigation
            .apply(NavCommand::SelectLeftTab(LeftNavTab::Search));
        let effects = reduce(
            &mut state,
            AppEvent::Command(AppCommand::PasteText("main".to_string())),
        );
        assert_eq!(state.search.query, "main");
        assert!(state.search.debounce_scheduled);
        assert!(effects
            .iter()
            .any(|effect| matches!(effect, SideEffect::CancelSearch)));
    }

    #[test]
    fn fs_changed_invalidates_expanded_file_tree_directory() {
        let root = PathBuf::from("/repo");
        let mut state = test_state();
        state.repo_root = root.clone();
        state.file_tree = FileTreeState::at_root(root.clone());
        let _ = state.file_tree.expand(&root);
        state.file_tree.apply_children_loaded(
            &root,
            vec![DirectoryEntry {
                path: root.join("src"),
                name: "src".to_string(),
                is_dir: true,
            }],
            None,
        );
        state
            .file_tree
            .nodes
            .get_mut(&root.join("src"))
            .expect("src")
            .expanded = true;
        state.file_tree.apply_children_loaded(
            &root.join("src"),
            vec![DirectoryEntry {
                path: root.join("src/main.rs"),
                name: "main.rs".to_string(),
                is_dir: false,
            }],
            None,
        );
        state.file_tree.select(root.join("src/main.rs"));

        let effects = reduce(
            &mut state,
            AppEvent::FsChanged {
                paths: vec![root.join("src/new.rs")],
            },
        );

        assert!(effects.contains(&SideEffect::LoadDirectoryChildren(root.join("src"))));
        assert!(!state.file_tree.nodes[&root.join("src")].children_loaded);
        assert_eq!(state.file_tree.selected, Some(root.join("src/main.rs")));
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
    fn fs_changed_skips_reload_when_mtime_unchanged() {
        use std::fs;
        use std::time::SystemTime;

        let temp = std::env::temp_dir().join(format!("kiwi-fs-changed-{}", std::process::id()));
        let _ = fs::remove_dir_all(&temp);
        fs::create_dir_all(&temp).expect("mkdir");
        let file = temp.join("main.rs");
        fs::write(&file, "unchanged").expect("write");
        let mtime = fs::metadata(&file).expect("metadata").modified().ok();

        let mut state = test_state();
        state.preview.path = Some(file.clone());
        state.preview.lines = vec!["unchanged".to_string()];
        state.preview.loaded_mtime = mtime;

        let effects = reduce(
            &mut state,
            AppEvent::FsChanged {
                paths: vec![file.clone()],
            },
        );

        assert!(effects.is_empty());
        assert!(!state.preview.loading);

        fs::write(&file, "changed").expect("rewrite");
        // Ensure mtime advances on platforms with coarse timestamps.
        if let Ok(updated) = fs::metadata(&file).and_then(|metadata| metadata.modified()) {
            if updated == mtime.unwrap_or(SystemTime::UNIX_EPOCH) {
                std::thread::sleep(std::time::Duration::from_millis(1100));
            }
        }

        let effects = reduce(
            &mut state,
            AppEvent::FsChanged {
                paths: vec![file.clone()],
            },
        );

        assert!(effects.contains(&SideEffect::LoadPreviewFile(file)));
        let _ = fs::remove_dir_all(temp);
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
