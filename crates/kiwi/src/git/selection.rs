use crate::state::GitState;

use super::panel::{
    build_panel_rows, path_for_row, row_for_path, scroll_offset_for_row, selectable_row_indices,
};

pub fn ensure_git_selection(state: &mut GitState, show_untracked: bool) {
    let rows = build_panel_rows(&state.file_entries, show_untracked);
    let selectable = selectable_row_indices(&rows);
    if selectable.is_empty() {
        state.selected_path = None;
        state.scroll_offset = 0;
        return;
    }

    if let Some(path) = &state.selected_path {
        if row_for_path(&rows, path).is_some() {
            return;
        }
    }

    let first_row = selectable[0];
    state.selected_path = path_for_row(&rows, first_row).map(str::to_string);
    state.scroll_offset = 0;
}

pub fn git_move_selection(
    state: &mut GitState,
    delta: i32,
    viewport_rows: usize,
    show_untracked: bool,
) {
    let rows = build_panel_rows(&state.file_entries, show_untracked);
    let selectable = selectable_row_indices(&rows);
    if selectable.is_empty() || viewport_rows == 0 {
        return;
    }

    let current_row = state
        .selected_path
        .as_deref()
        .and_then(|path| row_for_path(&rows, path))
        .unwrap_or(selectable[0]);

    let current_pos = selectable
        .iter()
        .position(|row| *row == current_row)
        .unwrap_or(0);
    let next_pos = (current_pos as i32 + delta).clamp(0, selectable.len().saturating_sub(1) as i32);
    let next_row = selectable[usize::try_from(next_pos).unwrap_or(0)];

    state.selected_path = path_for_row(&rows, next_row).map(str::to_string);
    state.scroll_offset = scroll_offset_for_row(next_row, state.scroll_offset, viewport_rows);
}

pub fn git_select_row(
    state: &mut GitState,
    row_index: usize,
    viewport_rows: usize,
    show_untracked: bool,
) {
    let rows = build_panel_rows(&state.file_entries, show_untracked);
    let Some(path) = path_for_row(&rows, row_index) else {
        return;
    };

    state.selected_path = Some(path.to_string());
    state.scroll_offset = scroll_offset_for_row(row_index, state.scroll_offset, viewport_rows);
}

pub fn git_row_at_viewport(
    state: &GitState,
    viewport_index: usize,
    show_untracked: bool,
) -> Option<super::panel::GitPanelRow> {
    let rows = build_panel_rows(&state.file_entries, show_untracked);
    rows.get(state.scroll_offset.saturating_add(viewport_index))
        .cloned()
}

pub fn git_selected_row_index(state: &GitState, show_untracked: bool) -> Option<usize> {
    let path = state.selected_path.as_deref()?;
    row_for_path(&build_panel_rows(&state.file_entries, show_untracked), path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::git::{GitFileEntry, GitFileStatus};

    #[test]
    fn move_selection_skips_section_headers() {
        let mut state = GitState {
            file_entries: vec![
                GitFileEntry {
                    path: "a.rs".to_string(),
                    status: GitFileStatus::Modified,
                },
                GitFileEntry {
                    path: "b.rs".to_string(),
                    status: GitFileStatus::Added,
                },
            ],
            selected_path: Some("a.rs".to_string()),
            ..GitState::default()
        };

        git_move_selection(&mut state, 1, 10, true);
        assert_eq!(state.selected_path.as_deref(), Some("b.rs"));
    }
}
