use crate::state::BranchState;

use super::branches::BranchEntry;

pub fn ensure_branch_selection(state: &mut BranchState) {
    if state.entries.is_empty() {
        state.selected_index = None;
        state.scroll_offset = 0;
        return;
    }

    if let Some(index) = state.selected_index {
        if index < state.entries.len() {
            return;
        }
    }

    let index = state
        .entries
        .iter()
        .position(|entry| entry.is_current)
        .unwrap_or(0);
    state.selected_index = Some(index);
    state.scroll_offset = scroll_offset_for_index(index, state.scroll_offset, 1);
}

pub fn branch_move_selection(state: &mut BranchState, delta: i32, viewport_rows: usize) {
    if state.entries.is_empty() || viewport_rows == 0 {
        return;
    }

    let current = state.selected_index.unwrap_or_else(|| {
        state
            .entries
            .iter()
            .position(|entry| entry.is_current)
            .unwrap_or(0)
    });
    let next = (current as i32 + delta).clamp(0, state.entries.len().saturating_sub(1) as i32);
    let next = usize::try_from(next).unwrap_or(0);
    state.selected_index = Some(next);
    state.scroll_offset = scroll_offset_for_index(next, state.scroll_offset, viewport_rows);
}

pub fn branch_select_row(state: &mut BranchState, row_index: usize, viewport_rows: usize) {
    if row_index >= state.entries.len() {
        return;
    }

    state.selected_index = Some(row_index);
    state.scroll_offset = scroll_offset_for_index(row_index, state.scroll_offset, viewport_rows);
}

pub fn branch_selected_name(state: &BranchState) -> Option<&str> {
    let index = state.selected_index?;
    state.entries.get(index).map(|entry| entry.name.as_str())
}

pub fn branch_row_at_viewport(state: &BranchState, viewport_index: usize) -> Option<&BranchEntry> {
    state
        .entries
        .get(state.scroll_offset.saturating_add(viewport_index))
}

pub fn branch_selected_row_index(state: &BranchState) -> Option<usize> {
    state.selected_index
}

fn scroll_offset_for_index(index: usize, current_offset: usize, viewport_rows: usize) -> usize {
    if viewport_rows == 0 {
        return 0;
    }

    if index < current_offset {
        return index;
    }

    if index >= current_offset.saturating_add(viewport_rows) {
        return index.saturating_sub(viewport_rows.saturating_sub(1));
    }

    current_offset
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::BranchState;

    fn sample_entries() -> Vec<BranchEntry> {
        vec![
            BranchEntry {
                name: "main".to_string(),
                is_current: true,
            },
            BranchEntry {
                name: "dev".to_string(),
                is_current: false,
            },
            BranchEntry {
                name: "feature".to_string(),
                is_current: false,
            },
        ]
    }

    #[test]
    fn move_selection_updates_scroll_when_needed() {
        let mut state = BranchState {
            entries: sample_entries(),
            selected_index: Some(0),
            scroll_offset: 0,
            ..BranchState::default()
        };

        branch_move_selection(&mut state, 2, 1);
        assert_eq!(state.selected_index, Some(2));
        assert_eq!(state.scroll_offset, 2);
    }

    #[test]
    fn ensure_selection_picks_current_branch() {
        let mut state = BranchState {
            entries: sample_entries(),
            ..BranchState::default()
        };

        ensure_branch_selection(&mut state);
        assert_eq!(state.selected_index, Some(0));
    }
}
