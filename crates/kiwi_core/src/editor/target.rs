use std::path::PathBuf;

use crate::navigation::{FocusTarget, LeftNavTab, MainTab};
use crate::state::ReduceView;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EditorTarget {
    pub path: PathBuf,
    pub line: Option<u32>,
}

pub fn resolve_editor_target(state: &ReduceView<'_>) -> Option<EditorTarget> {
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

fn editor_target_from_search(state: &ReduceView<'_>) -> Option<EditorTarget> {
    let result = state.search.results.get(state.search.selected)?;
    Some(EditorTarget {
        path: result.path.clone(),
        line: result.line,
    })
}

fn editor_target_from_preview(state: &ReduceView<'_>) -> Option<EditorTarget> {
    let path = state.preview.path.clone()?;
    let line = u32::try_from(state.preview.scroll_offset.saturating_add(1)).ok();
    Some(EditorTarget { path, line })
}

fn editor_target_from_file_tree(state: &ReduceView<'_>) -> Option<EditorTarget> {
    let path = state.file_tree.selected.clone()?;
    let node = state.file_tree.nodes.get(&path)?;
    if node.is_dir {
        return None;
    }
    Some(EditorTarget { path, line: None })
}

fn editor_target_fallback(state: &ReduceView<'_>) -> Option<EditorTarget> {
    editor_target_from_preview(state)
        .or_else(|| editor_target_from_search(state))
        .or_else(|| editor_target_from_file_tree(state))
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::config::ResolvedConfig;
    use crate::file_tree::{DirectoryEntry, FileTreeState};
    use crate::navigation::{NavCommand, NavigationState};
    use crate::preview::PreviewState;
    use crate::search::{SearchMode, SearchResult, SearchState};
    use crate::state::{AppState, ViewportMetrics};
    use crate::theme::{load_theme_with_capabilities, TerminalCapabilities};

    use super::*;

    fn test_view() -> AppState {
        AppState::from_startup(
            PathBuf::from("/tmp/repo"),
            true,
            ResolvedConfig::default(),
            load_theme_with_capabilities(
                &ResolvedConfig::default().theme,
                TerminalCapabilities::TrueColor,
            )
            .expect("theme"),
            TerminalCapabilities::TrueColor,
            ViewportMetrics::default(),
        )
    }

    #[test]
    fn preview_focus_uses_path_and_viewport_line() {
        let mut app = test_view();
        app.navigation
            .apply(NavCommand::SetFocus(FocusTarget::Main));
        app.navigation
            .apply(NavCommand::SelectMainTab(MainTab::Preview));
        app.preview = PreviewState {
            path: Some(PathBuf::from("/tmp/repo/src/main.rs")),
            scroll_offset: 41,
            ..PreviewState::default()
        };

        let view = ReduceView::from_app_state(&mut app);
        let target = resolve_editor_target(&view).expect("target");
        assert_eq!(target.path, PathBuf::from("/tmp/repo/src/main.rs"));
        assert_eq!(target.line, Some(42));
    }

    #[test]
    fn file_tree_focus_skips_directories() {
        let mut app = test_view();
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
        app.file_tree = file_tree;
        app.navigation
            .apply(NavCommand::SetFocus(FocusTarget::Left));
        app.navigation
            .apply(NavCommand::SelectLeftTab(LeftNavTab::Files));

        let view = ReduceView::from_app_state(&mut app);
        assert!(resolve_editor_target(&view).is_none());
    }
}
