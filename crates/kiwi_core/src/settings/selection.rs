use crate::state::SettingsState;
use crate::theme::BUILTIN_THEME_NAMES;

pub fn settings_theme_count() -> usize {
    BUILTIN_THEME_NAMES.len()
}

pub fn ensure_settings_selection(
    state: &mut SettingsState,
    active_theme: &str,
    viewport_rows: usize,
) {
    let selected = BUILTIN_THEME_NAMES
        .iter()
        .position(|name| *name == active_theme)
        .unwrap_or(0);
    state.selected_index = selected;
    state.scroll_offset =
        scroll_offset_for_index(selected, state.scroll_offset, viewport_rows.max(1));
}

pub fn settings_move_selection(state: &mut SettingsState, delta: i32, viewport_rows: usize) {
    let count = settings_theme_count();
    if count == 0 || viewport_rows == 0 {
        return;
    }

    let next = (state.selected_index as i32 + delta).clamp(0, count.saturating_sub(1) as i32);
    let next = usize::try_from(next).unwrap_or(0);
    state.selected_index = next;
    state.scroll_offset = scroll_offset_for_index(next, state.scroll_offset, viewport_rows);
}

pub fn settings_select_row(state: &mut SettingsState, row_index: usize, viewport_rows: usize) {
    if row_index >= settings_theme_count() {
        return;
    }

    state.selected_index = row_index;
    state.scroll_offset = scroll_offset_for_index(row_index, state.scroll_offset, viewport_rows);
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
    use crate::state::SettingsState;

    #[test]
    fn move_selection_updates_scroll_when_needed() {
        let mut state = SettingsState {
            selected_index: 0,
            scroll_offset: 0,
        };

        settings_move_selection(&mut state, 7, 3);
        assert_eq!(state.selected_index, 7);
        assert_eq!(state.scroll_offset, 5);
    }
}
