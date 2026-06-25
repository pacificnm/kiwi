use std::path::PathBuf;

use crate::layout::FocusTarget;
use crate::navigation::{LeftNavTab, MainTab};
use crate::state::AppState;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EditorTarget {
    pub path: PathBuf,
    pub line: Option<u32>,
}

pub fn resolve_editor_target(state: &AppState) -> Option<EditorTarget> {
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
        _ => editor_target_fallback(state),
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

fn editor_target_fallback(state: &AppState) -> Option<EditorTarget> {
    editor_target_from_preview(state)
        .or_else(|| editor_target_from_search(state))
        .or_else(|| editor_target_from_file_tree(state))
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::config::ResolvedConfig;
    use crate::file_tree::{DirectoryEntry, FileTreeState};
    use crate::layout::{compute_layout, FocusTarget};
    use crate::navigation::{LeftNavTab, MainTab, NavCommand};
    use crate::preview::PreviewState;
    use crate::search::{SearchMode, SearchResult, SearchState};
    use crate::state::AppState;
    use crate::theme::capabilities::TerminalCapabilities;
    use crate::theme::loader::load_theme_with_capabilities;

    use super::*;

    fn test_state() -> AppState {
        AppState::from_startup(
            PathBuf::from("/tmp/repo"),
            true,
            ResolvedConfig::default(),
            load_theme_with_capabilities(
                &ResolvedConfig::default().theme,
                TerminalCapabilities::TrueColor,
            )
            .expect("theme"),
            compute_layout(120, 40, 30).expect("layout"),
        )
    }

    #[test]
    fn preview_focus_uses_path_and_viewport_line() {
        let mut state = test_state();
        state
            .navigation
            .apply(NavCommand::SetFocus(FocusTarget::Main));
        state
            .navigation
            .apply(NavCommand::SelectMainTab(MainTab::Preview));
        state.preview = PreviewState {
            path: Some(PathBuf::from("/tmp/repo/src/main.rs")),
            scroll_offset: 41,
            ..PreviewState::default()
        };

        let target = resolve_editor_target(&state).expect("target");
        assert_eq!(target.path, PathBuf::from("/tmp/repo/src/main.rs"));
        assert_eq!(target.line, Some(42));
    }

    #[test]
    fn search_focus_uses_result_line_for_content_hits() {
        let mut state = test_state();
        state
            .navigation
            .apply(NavCommand::SetFocus(FocusTarget::Left));
        state
            .navigation
            .apply(NavCommand::SelectLeftTab(LeftNavTab::Search));
        state.search = SearchState {
            mode: SearchMode::Content,
            results: vec![SearchResult::content(
                PathBuf::from("/tmp/repo/src/lib.rs"),
                12,
                "fn main".to_string(),
            )],
            selected: 0,
            ..SearchState::default()
        };

        let target = resolve_editor_target(&state).expect("target");
        assert_eq!(target.line, Some(12));
    }

    #[test]
    fn file_tree_focus_skips_directories() {
        let mut state = test_state();
        let root = PathBuf::from("/tmp/repo");
        let mut file_tree = FileTreeState::at_root(root.clone());
        file_tree.apply_children_loaded(
            &root,
            vec![DirectoryEntry {
                path: root.join("src"),
                name: "src".to_string(),
                is_dir: true,
            }],
            None,
        );
        file_tree.selected = Some(root.join("src"));
        state.file_tree = file_tree;
        state
            .navigation
            .apply(NavCommand::SetFocus(FocusTarget::Left));
        state
            .navigation
            .apply(NavCommand::SelectLeftTab(LeftNavTab::Files));

        assert!(resolve_editor_target(&state).is_none());
    }

    #[test]
    fn fallback_prefers_preview_over_file_tree() {
        let mut state = test_state();
        state.preview.path = Some(PathBuf::from("/tmp/repo/preview.rs"));
        state.file_tree.selected = Some(PathBuf::from("/tmp/repo/other.rs"));

        let target = resolve_editor_target(&state).expect("target");
        assert_eq!(target.path, PathBuf::from("/tmp/repo/preview.rs"));
    }
}
