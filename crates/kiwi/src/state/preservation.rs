//! Scroll and selection preservation across watcher-driven updates (ADR-007, ADR-011).

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::config::ResolvedConfig;
    use crate::file_tree::{DirectoryEntry, FileTreeState};
    use crate::git::{GitFileEntry, GitFileStatus};
    use crate::layout::compute_layout;
    use crate::layout::FocusTarget;
    use crate::navigation::{LeftNavTab, MainTab, NavigationState};
    use crate::state::domains::{
        AgentState, CommandPaletteState, DiffState, GitHubState, GitState, ShellState,
        StatusBarState, WorkspaceMeta,
    };
    use crate::state::{reduce, AppEvent, AppState, SideEffect};
    use crate::theme::capabilities::TerminalCapabilities;
    use crate::theme::loader::load_theme_with_capabilities;

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
            repo_root: PathBuf::from("/repo"),
            dirty: false,
            file_tree: FileTreeState::at_root(PathBuf::from("/repo")),
            preview: crate::preview::PreviewState::default(),
            search: crate::search::SearchState::default(),
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

    fn modified_entries(prefix: &str, count: usize) -> Vec<GitFileEntry> {
        (0..count)
            .map(|index| GitFileEntry {
                path: format!("{prefix}{index}.rs"),
                status: GitFileStatus::Modified,
            })
            .collect()
    }

    #[test]
    fn git_status_refresh_preserves_git_panel_scroll_and_selection() {
        let mut state = test_state();
        state.git.scroll_offset = 12;
        state.git.selected_path = Some("file5.rs".to_string());
        state.git.file_entries = modified_entries("file", 20);

        reduce(
            &mut state,
            AppEvent::GitStatusUpdated {
                branch: Some("main".to_string()),
                ahead: 0,
                behind: 0,
                file_entries: modified_entries("file", 21),
                error: None,
            },
        );

        assert_eq!(state.git.scroll_offset, 12);
        assert_eq!(state.git.selected_path, Some("file5.rs".to_string()));
    }

    #[test]
    fn git_status_refresh_preserves_file_tree_scroll_offset() {
        let mut state = test_state();
        let root = state.file_tree.root.clone();
        state.file_tree.scroll_offset = 4;
        state.file_tree.selected = Some(root.join("src/main.rs"));
        state.file_tree.nodes.insert(
            root.join("src/main.rs"),
            crate::file_tree::FileNode {
                path: root.join("src/main.rs"),
                name: "main.rs".to_string(),
                is_dir: false,
                expanded: false,
                children_loaded: true,
                load_error: None,
                git_status: None,
            },
        );

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

        assert_eq!(state.file_tree.scroll_offset, 4);
        assert_eq!(state.file_tree.selected, Some(root.join("src/main.rs")));
    }

    #[test]
    fn git_status_refresh_preserves_navigation_focus() {
        let mut state = test_state();
        state.navigation.focus = FocusTarget::Left;
        state.navigation.left_tab = LeftNavTab::Git;
        state.navigation.main_tab = MainTab::Preview;

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

        assert_eq!(state.navigation.focus, FocusTarget::Left);
        assert_eq!(state.navigation.left_tab, LeftNavTab::Git);
        assert_eq!(state.navigation.main_tab, MainTab::Preview);
    }

    #[test]
    fn fs_changed_preserves_file_tree_scroll_selection_and_focus() {
        let mut state = test_state();
        let root = PathBuf::from("/repo");
        state.repo_root = root.clone();
        state.file_tree = FileTreeState::at_root(root.clone());
        state.file_tree.scroll_offset = 3;
        state.navigation.focus = FocusTarget::Left;
        state.navigation.left_tab = LeftNavTab::Files;
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

        reduce(
            &mut state,
            AppEvent::FsChanged {
                paths: vec![root.join("src/new.rs")],
            },
        );

        assert_eq!(state.file_tree.scroll_offset, 3);
        assert_eq!(state.file_tree.selected, Some(root.join("src/main.rs")));
        assert_eq!(state.navigation.focus, FocusTarget::Left);
        assert_eq!(state.navigation.left_tab, LeftNavTab::Files);
    }

    #[test]
    fn fs_changed_preserves_git_panel_scroll_when_refresh_starts() {
        let mut state = test_state();
        state.workspace_meta.is_git_repo = true;
        state.config.git.watch = true;
        state.git.scroll_offset = 6;
        state.git.selected_path = Some("src/main.rs".to_string());

        reduce(
            &mut state,
            AppEvent::FsChanged {
                paths: vec![PathBuf::from("/repo/src/main.rs")],
            },
        );

        assert_eq!(state.git.scroll_offset, 6);
        assert_eq!(state.git.selected_path, Some("src/main.rs".to_string()));
        assert!(state.git.loading);
    }

    #[test]
    fn file_tree_reload_after_fs_changed_preserves_scroll_and_selection() {
        let root = PathBuf::from("/repo");
        let mut state = test_state();
        state.repo_root = root.clone();
        state.file_tree = FileTreeState::at_root(root.clone());
        state.file_tree.scroll_offset = 2;
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

        reduce(
            &mut state,
            AppEvent::FileTreeChildrenLoaded {
                parent: root.join("src"),
                children: vec![
                    DirectoryEntry {
                        path: root.join("src/main.rs"),
                        name: "main.rs".to_string(),
                        is_dir: false,
                    },
                    DirectoryEntry {
                        path: root.join("src/new.rs"),
                        name: "new.rs".to_string(),
                        is_dir: false,
                    },
                ],
                error: None,
            },
        );

        assert_eq!(state.file_tree.scroll_offset, 2);
        assert_eq!(state.file_tree.selected, Some(root.join("src/main.rs")));
        assert!(state.file_tree.nodes.contains_key(&root.join("src/new.rs")));
    }
}
