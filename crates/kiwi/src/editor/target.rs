pub use kiwi_core::editor::EditorTarget;

use kiwi_core::navigation::{FocusTarget, LeftNavTab, MainTab};

use crate::state::AppState;

pub fn resolve_editor_target(state: &mut AppState) -> Option<EditorTarget> {
    kiwi_core::editor::resolve_editor_target(&state.reduce_view())
}

pub fn resolve_editor_target_readonly(state: &AppState) -> Option<EditorTarget> {
    match state.navigation.focus {
        FocusTarget::Left if state.navigation.left_tab == LeftNavTab::Search => {
            editor_target_from_search(state)
        }
        FocusTarget::Left if state.navigation.left_tab == LeftNavTab::Files => {
            editor_target_from_file_tree(state)
        }
        FocusTarget::Main if state.navigation.main_tab == MainTab::Preview => {
            editor_target_from_preview(state)
        }
        _ => editor_target_from_preview(state)
            .or_else(|| editor_target_from_search(state))
            .or_else(|| editor_target_from_file_tree(state)),
    }
}

fn editor_target_from_search(state: &AppState) -> Option<EditorTarget> {
    let result = state.search.results.get(state.search.selected)?;
    Some(EditorTarget {
        path: result.path.clone(),
        line: result.line,
    })
}

fn editor_target_from_preview(state: &AppState) -> Option<EditorTarget> {
    let path = state.preview.path.clone()?;
    let line = u32::try_from(state.preview.scroll_offset.saturating_add(1)).ok();
    Some(EditorTarget { path, line })
}

fn editor_target_from_file_tree(state: &AppState) -> Option<EditorTarget> {
    let path = state.file_tree.selected.clone()?;
    let node = state.file_tree.nodes.get(&path)?;
    if node.is_dir {
        return None;
    }
    Some(EditorTarget { path, line: None })
}
