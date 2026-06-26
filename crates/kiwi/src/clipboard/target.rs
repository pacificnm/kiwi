use kiwi_core::clipboard::{
    resolve_copy_text_for_focus as core_resolve_copy_text_for_focus,
    resolve_paste_target as core_resolve_paste_target, PasteTarget,
};
use kiwi_core::navigation::FocusTarget;

use crate::state::AppState;

pub fn resolve_copy_text(state: &mut AppState) -> Option<String> {
    if let Some(text) = state.text_selection.extract_text(state) {
        if !text.is_empty() {
            return Some(text);
        }
    }

    if state.palette.open {
        if !state.palette.input.is_empty() {
            return Some(state.palette.input.clone());
        }
        return resolve_copy_text_for_focus(state, state.palette.focus_before_open);
    }

    resolve_copy_text_for_focus(state, state.navigation.focus)
}

pub fn resolve_copy_text_for_focus(state: &mut AppState, focus: FocusTarget) -> Option<String> {
    core_resolve_copy_text_for_focus(&state.reduce_view(), focus)
}

pub fn resolve_paste_target(state: &mut AppState) -> PasteTarget {
    core_resolve_paste_target(&state.reduce_view())
}
