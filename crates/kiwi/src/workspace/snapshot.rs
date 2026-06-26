#![allow(dead_code)] // scroll_view constants are public API for callers.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::github::GitHubLeftPane;
use crate::layout::FocusTarget;
use crate::navigation::{LeftNavTab, MainTab};
use crate::state::AppState;

use super::MAX_PALETTE_HISTORY_ENTRIES;

pub const WORKSPACE_SCHEMA_VERSION: u32 = 1;

const DIFF_SCROLL_PREFIX: &str = "diff:";
const DIFF_H_SCROLL_PREFIX: &str = "diff_h:";

/// Stable view identifiers for [`WorkspaceSnapshot::scroll_positions`].
pub mod scroll_view {
    pub const FILE_TREE: &str = "file_tree";
    pub const GIT: &str = "git";
    pub const SEARCH: &str = "search";
    pub const BRANCHES: &str = "branches";
    pub const PREVIEW: &str = "preview";
    pub const DIFF: &str = "diff";
    pub const DIFF_HORIZONTAL: &str = "diff_h";
    pub const GITHUB_ISSUES: &str = "github.issues";
    pub const GITHUB_PRS: &str = "github.prs";
    pub const GITHUB_ISSUE_DETAIL: &str = "github.issue_detail";
    pub const GITHUB_PR_DETAIL: &str = "github.pr_detail";
}

/// Serializable workspace state for a single repository (SPEC-017, ADR-016).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkspaceSnapshot {
    pub schema_version: u32,
    pub left_nav_tab: String,
    pub main_tab: String,
    pub focus: String,
    pub left_width: u8,
    pub expanded_paths: Vec<String>,
    pub selected_path: Option<String>,
    #[serde(default)]
    pub scroll_positions: HashMap<String, usize>,
    #[serde(default, alias = "palette_history")]
    pub command_palette_history: Vec<String>,
}

impl Default for WorkspaceSnapshot {
    fn default() -> Self {
        Self {
            schema_version: WORKSPACE_SCHEMA_VERSION,
            left_nav_tab: LeftNavTab::default().label().to_string(),
            main_tab: MainTab::default().label().to_string(),
            focus: focus_label(FocusTarget::Main).to_string(),
            left_width: 30,
            expanded_paths: Vec::new(),
            selected_path: None,
            scroll_positions: HashMap::new(),
            command_palette_history: Vec::new(),
        }
    }
}

impl WorkspaceSnapshot {
    #[must_use]
    pub fn is_compatible(&self) -> bool {
        self.schema_version == WORKSPACE_SCHEMA_VERSION
    }

    #[must_use]
    pub fn from_app_state(state: &AppState) -> Self {
        let repo_root = &state.repo_root;
        let mut scroll_positions = HashMap::new();
        scroll_positions.insert(
            scroll_view::FILE_TREE.to_string(),
            state.file_tree.scroll_offset,
        );
        scroll_positions.insert(scroll_view::GIT.to_string(), state.git.scroll_offset);
        scroll_positions.insert(scroll_view::SEARCH.to_string(), state.search.scroll_offset);
        scroll_positions.insert(
            scroll_view::BRANCHES.to_string(),
            state.branches.scroll_offset,
        );
        scroll_positions.insert(
            scroll_view::PREVIEW.to_string(),
            state.preview.scroll_offset,
        );
        scroll_positions.insert(scroll_view::DIFF.to_string(), state.diff.scroll_offset);
        scroll_positions.insert(
            scroll_view::DIFF_HORIZONTAL.to_string(),
            state.diff.horizontal_scroll_offset,
        );
        scroll_positions.insert(
            scroll_view::GITHUB_ISSUES.to_string(),
            state.github.issues_scroll_offset,
        );
        scroll_positions.insert(
            scroll_view::GITHUB_PRS.to_string(),
            state.github.prs_scroll_offset,
        );
        scroll_positions.insert(
            scroll_view::GITHUB_ISSUE_DETAIL.to_string(),
            state.github.issue_detail_scroll_offset,
        );
        scroll_positions.insert(
            scroll_view::GITHUB_PR_DETAIL.to_string(),
            state.github.pr_detail_scroll_offset,
        );

        for (path, (vertical, horizontal)) in &state.diff.scroll_by_path {
            scroll_positions.insert(format!("{DIFF_SCROLL_PREFIX}{path}"), *vertical);
            scroll_positions.insert(format!("{DIFF_H_SCROLL_PREFIX}{path}"), *horizontal);
        }

        let expanded_paths = state
            .file_tree
            .nodes
            .values()
            .filter(|node| node.expanded)
            .map(|node| rel_path_string(repo_root, &node.path))
            .collect();

        let selected_path = state
            .file_tree
            .selected
            .as_ref()
            .map(|path| rel_path_string(repo_root, path));

        Self {
            schema_version: WORKSPACE_SCHEMA_VERSION,
            left_nav_tab: state.navigation.left_tab.label().to_string(),
            main_tab: state.navigation.main_tab.label().to_string(),
            focus: focus_label(state.navigation.focus).to_string(),
            left_width: state.config.app.left_width,
            expanded_paths,
            selected_path,
            scroll_positions,
            command_palette_history: trim_history(state.palette.history.clone()),
        }
    }

    pub fn apply_to_app_state(&self, state: &mut AppState) {
        if let Some(left_tab) = parse_left_nav_tab(&self.left_nav_tab) {
            state.navigation.left_tab = left_tab;
        }
        if let Some(main_tab) = parse_main_tab(&self.main_tab) {
            state.navigation.main_tab = main_tab;
        }
        if let Some(focus) = parse_focus(&self.focus) {
            state.navigation.focus = focus;
        }

        state.config.app.left_width = self.left_width;

        state.workspace_meta.pending_expanded_paths = self
            .expanded_paths
            .iter()
            .map(|rel| abs_path(&state.repo_root, rel))
            .collect();
        for path in &state.workspace_meta.pending_expanded_paths {
            if let Some(node) = state.file_tree.nodes.get_mut(path) {
                node.expanded = true;
            }
        }

        state.workspace_meta.pending_selected_path = None;
        if let Some(rel) = &self.selected_path {
            let path = abs_path(&state.repo_root, rel);
            if state.file_tree.nodes.contains_key(&path) {
                state.file_tree.select(path);
            } else {
                state.workspace_meta.pending_selected_path = Some(path);
            }
        }

        apply_scroll_positions(state, &self.scroll_positions);
        sync_github_left_pane(state);
        state.palette.history = trim_history(self.command_palette_history.clone());
        state.dirty = true;
    }
}

pub fn trim_history(mut history: Vec<String>) -> Vec<String> {
    if history.len() > MAX_PALETTE_HISTORY_ENTRIES {
        let overflow = history.len() - MAX_PALETTE_HISTORY_ENTRIES;
        history.drain(0..overflow);
    }
    history
}

fn apply_scroll_positions(state: &mut AppState, scroll_positions: &HashMap<String, usize>) {
    state.file_tree.scroll_offset = *scroll_positions
        .get(scroll_view::FILE_TREE)
        .unwrap_or(&state.file_tree.scroll_offset);
    state.git.scroll_offset = *scroll_positions
        .get(scroll_view::GIT)
        .unwrap_or(&state.git.scroll_offset);
    state.search.scroll_offset = *scroll_positions
        .get(scroll_view::SEARCH)
        .unwrap_or(&state.search.scroll_offset);
    state.branches.scroll_offset = *scroll_positions
        .get(scroll_view::BRANCHES)
        .unwrap_or(&state.branches.scroll_offset);
    state.preview.scroll_offset = *scroll_positions
        .get(scroll_view::PREVIEW)
        .unwrap_or(&state.preview.scroll_offset);
    state.diff.scroll_offset = *scroll_positions
        .get(scroll_view::DIFF)
        .unwrap_or(&state.diff.scroll_offset);
    state.diff.horizontal_scroll_offset = *scroll_positions
        .get(scroll_view::DIFF_HORIZONTAL)
        .unwrap_or(&state.diff.horizontal_scroll_offset);
    state.github.issues_scroll_offset = *scroll_positions
        .get(scroll_view::GITHUB_ISSUES)
        .unwrap_or(&state.github.issues_scroll_offset);
    state.github.prs_scroll_offset = *scroll_positions
        .get(scroll_view::GITHUB_PRS)
        .unwrap_or(&state.github.prs_scroll_offset);
    state.github.issue_detail_scroll_offset = *scroll_positions
        .get(scroll_view::GITHUB_ISSUE_DETAIL)
        .unwrap_or(&state.github.issue_detail_scroll_offset);
    state.github.pr_detail_scroll_offset = *scroll_positions
        .get(scroll_view::GITHUB_PR_DETAIL)
        .unwrap_or(&state.github.pr_detail_scroll_offset);

    state.diff.scroll_by_path.clear();
    for (key, vertical) in scroll_positions {
        if let Some(rel) = key.strip_prefix(DIFF_SCROLL_PREFIX) {
            let horizontal = scroll_positions
                .get(&format!("{DIFF_H_SCROLL_PREFIX}{rel}"))
                .copied()
                .unwrap_or(0);
            state
                .diff
                .scroll_by_path
                .insert(rel.to_string(), (*vertical, horizontal));
        }
    }
}

fn sync_github_left_pane(state: &mut AppState) {
    state.github.left_pane = match state.navigation.main_tab {
        MainTab::Issues => GitHubLeftPane::Issues,
        MainTab::Prs => GitHubLeftPane::Prs,
        _ => state.github.left_pane,
    };
}

fn rel_path_string(repo_root: &Path, path: &Path) -> String {
    path.strip_prefix(repo_root)
        .unwrap_or(path)
        .display()
        .to_string()
}

fn abs_path(repo_root: &Path, rel: &str) -> PathBuf {
    repo_root.join(rel)
}

const fn focus_label(focus: FocusTarget) -> &'static str {
    match focus {
        FocusTarget::Left => "Left",
        FocusTarget::Main => "Main",
        FocusTarget::CommandPalette => "CommandPalette",
        FocusTarget::Shell => "Shell",
    }
}

fn parse_left_nav_tab(label: &str) -> Option<LeftNavTab> {
    LeftNavTab::ALL.into_iter().find(|tab| tab.label() == label)
}

fn parse_main_tab(label: &str) -> Option<MainTab> {
    MainTab::ALL.into_iter().find(|tab| tab.label() == label)
}

fn parse_focus(label: &str) -> Option<FocusTarget> {
    match label {
        "Left" => Some(FocusTarget::Left),
        "Main" => Some(FocusTarget::Main),
        "CommandPalette" => Some(FocusTarget::CommandPalette),
        "Shell" => Some(FocusTarget::Shell),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::config::ResolvedConfig;
    use crate::layout::compute_layout;
    use crate::state::AppState;
    use crate::theme::capabilities::TerminalCapabilities;
    use crate::theme::loader::load_theme_with_capabilities;

    use super::*;

    fn test_state() -> AppState {
        AppState::from_startup(
            PathBuf::from("/tmp/kiwi"),
            false,
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
    fn default_snapshot_matches_schema_version() {
        let snapshot = WorkspaceSnapshot::default();
        assert_eq!(snapshot.schema_version, WORKSPACE_SCHEMA_VERSION);
        assert!(snapshot.is_compatible());
    }

    #[test]
    fn snapshot_round_trips_through_json() {
        let snapshot = WorkspaceSnapshot {
            left_nav_tab: "Git".to_string(),
            main_tab: "Diff".to_string(),
            focus: "Left".to_string(),
            left_width: 25,
            expanded_paths: vec!["src".to_string()],
            selected_path: Some("src/main.rs".to_string()),
            scroll_positions: HashMap::from([
                (scroll_view::FILE_TREE.to_string(), 3),
                (scroll_view::GIT.to_string(), 7),
            ]),
            command_palette_history: vec!["quit".to_string()],
            ..WorkspaceSnapshot::default()
        };

        let json = serde_json::to_string(&snapshot).expect("serialize");
        let decoded: WorkspaceSnapshot = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(decoded, snapshot);
    }

    #[test]
    fn legacy_palette_history_field_deserializes() {
        let json = r#"{"schema_version":1,"left_nav_tab":"Files","main_tab":"Agent","focus":"Main","left_width":30,"expanded_paths":[],"selected_path":null,"scroll_positions":{},"palette_history":["quit"]}"#;
        let snapshot: WorkspaceSnapshot = serde_json::from_str(json).expect("deserialize");
        assert_eq!(snapshot.command_palette_history, vec!["quit".to_string()]);
    }

    #[test]
    fn from_app_state_captures_navigation_and_scroll() {
        let mut state = test_state();
        state.navigation.left_tab = LeftNavTab::Git;
        state.navigation.main_tab = MainTab::Diff;
        state.navigation.focus = FocusTarget::Left;
        state.config.app.left_width = 22;
        state.file_tree.scroll_offset = 5;
        state.git.scroll_offset = 9;
        state.palette.history = vec!["git.refresh".to_string()];

        let snapshot = WorkspaceSnapshot::from_app_state(&state);
        assert_eq!(snapshot.left_nav_tab, "Git");
        assert_eq!(snapshot.main_tab, "Diff");
        assert_eq!(snapshot.focus, "Left");
        assert_eq!(snapshot.left_width, 22);
        assert_eq!(
            snapshot.scroll_positions.get(scroll_view::FILE_TREE),
            Some(&5)
        );
        assert_eq!(snapshot.scroll_positions.get(scroll_view::GIT), Some(&9));
        assert_eq!(
            snapshot.command_palette_history,
            vec!["git.refresh".to_string()]
        );
    }

    #[test]
    fn apply_to_app_state_restores_tabs_focus_and_scroll() {
        let mut state = test_state();
        let snapshot = WorkspaceSnapshot {
            left_nav_tab: "GH".to_string(),
            main_tab: "Issues".to_string(),
            focus: "Main".to_string(),
            left_width: 28,
            scroll_positions: HashMap::from([(scroll_view::GITHUB_ISSUES.to_string(), 4)]),
            command_palette_history: vec!["focus.main".to_string()],
            ..WorkspaceSnapshot::default()
        };

        snapshot.apply_to_app_state(&mut state);

        assert_eq!(state.navigation.left_tab, LeftNavTab::Gh);
        assert_eq!(state.navigation.main_tab, MainTab::Issues);
        assert_eq!(state.navigation.focus, FocusTarget::Main);
        assert_eq!(state.config.app.left_width, 28);
        assert_eq!(state.github.issues_scroll_offset, 4);
        assert_eq!(state.github.left_pane, GitHubLeftPane::Issues);
        assert_eq!(state.palette.history, vec!["focus.main".to_string()]);
    }

    #[test]
    fn apply_ignores_unknown_tab_labels() {
        let mut state = test_state();
        state.navigation.left_tab = LeftNavTab::Git;
        state.navigation.main_tab = MainTab::Diff;

        let snapshot = WorkspaceSnapshot {
            left_nav_tab: "NotATab".to_string(),
            main_tab: "AlsoNotATab".to_string(),
            ..WorkspaceSnapshot::default()
        };
        snapshot.apply_to_app_state(&mut state);

        assert_eq!(state.navigation.left_tab, LeftNavTab::Git);
        assert_eq!(state.navigation.main_tab, MainTab::Diff);
    }

    #[test]
    fn trim_history_caps_at_max_entries() {
        let history: Vec<String> = (0..60).map(|index| format!("cmd.{index}")).collect();
        let trimmed = trim_history(history);
        assert_eq!(trimmed.len(), MAX_PALETTE_HISTORY_ENTRIES);
        assert_eq!(trimmed.first().map(String::as_str), Some("cmd.10"));
        assert_eq!(trimmed.last().map(String::as_str), Some("cmd.59"));
    }
}
