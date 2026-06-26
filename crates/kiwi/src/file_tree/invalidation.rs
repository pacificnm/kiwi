use std::collections::HashSet;
use std::path::{Component, Path, PathBuf};

use super::state::{ExpandAction, FileTreeState};

fn is_git_internal_path(path: &Path) -> bool {
    path.components()
        .any(|component| matches!(component, Component::Normal(name) if name == ".git"))
}

/// Maps watcher paths to directory caches that should be invalidated per ADR-011.
#[must_use]
pub fn directories_to_invalidate(
    repo_root: &Path,
    changed_paths: &[PathBuf],
    is_directory: impl Fn(&Path) -> bool,
) -> Vec<PathBuf> {
    let mut dirs = HashSet::new();

    for path in changed_paths {
        if is_git_internal_path(path) || !path.starts_with(repo_root) {
            continue;
        }

        if path == repo_root {
            dirs.insert(repo_root.to_path_buf());
            continue;
        }

        if let Some(parent) = path.parent() {
            if parent.starts_with(repo_root) || parent == repo_root {
                dirs.insert(parent.to_path_buf());
            }
        }

        if is_directory(path) {
            dirs.insert(path.to_path_buf());
        }
    }

    let mut sorted: Vec<PathBuf> = dirs.into_iter().collect();
    sorted.sort_by_key(|path| path.components().count());
    sorted
}

impl FileTreeState {
    /// Invalidates affected directory caches and returns expanded dirs that need reload.
    pub fn apply_fs_invalidation(
        &mut self,
        repo_root: &Path,
        changed_paths: &[PathBuf],
    ) -> Vec<PathBuf> {
        let mut reload_dirs = Vec::new();
        let invalidation_dirs = directories_to_invalidate(repo_root, changed_paths, |path| {
            self.nodes.get(path).is_some_and(|node| node.is_dir) || path.is_dir()
        });

        for dir in invalidation_dirs {
            if !self.nodes.contains_key(&dir) {
                continue;
            }

            let should_reload = self
                .nodes
                .get(&dir)
                .is_some_and(|node| node.expanded && node.children_loaded);

            self.invalidate_children(&dir);

            if should_reload && matches!(self.expand(&dir), Ok(ExpandAction::NeedsLoad)) {
                reload_dirs.push(dir);
            }
        }

        reload_dirs
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;
    use crate::file_tree::{DirectoryEntry, FileTreeState};

    #[test]
    fn directories_to_invalidate_targets_parent_for_files() {
        let root = PathBuf::from("/repo");
        let dirs = directories_to_invalidate(&root, &[root.join("src/main.rs")], |_| false);
        assert_eq!(dirs, vec![root.join("src")]);
    }

    #[test]
    fn directories_to_invalidate_targets_directory_and_parent() {
        let temp = std::env::temp_dir().join(format!("kiwi-invalidate-dir-{}", std::process::id()));
        let _ = fs::remove_dir_all(&temp);
        fs::create_dir_all(temp.join("src")).expect("mkdir");

        let dirs = directories_to_invalidate(&temp, &[temp.join("src")], |path| path.is_dir());
        assert!(dirs.contains(&temp));
        assert!(dirs.contains(&temp.join("src")));

        let _ = fs::remove_dir_all(temp);
    }

    #[test]
    fn directories_to_invalidate_skips_git_internal_paths() {
        let root = PathBuf::from("/repo");
        let dirs = directories_to_invalidate(&root, &[root.join(".git/index")], |_| false);
        assert!(dirs.is_empty());
    }

    #[test]
    fn apply_fs_invalidation_reloads_expanded_parent() {
        let root = PathBuf::from("/repo");
        let mut state = FileTreeState::at_root(root.clone());
        let _ = state.expand(&root);
        state.apply_children_loaded(
            &root,
            vec![DirectoryEntry {
                path: root.join("src"),
                name: "src".to_string(),
                is_dir: true,
            }],
            None,
        );
        state
            .nodes
            .get_mut(&root.join("src"))
            .expect("src")
            .expanded = true;
        state.apply_children_loaded(
            &root.join("src"),
            vec![DirectoryEntry {
                path: root.join("src/main.rs"),
                name: "main.rs".to_string(),
                is_dir: false,
            }],
            None,
        );
        state.select(root.join("src/main.rs"));

        let reload_dirs = state.apply_fs_invalidation(&root, &[root.join("src/new.rs")]);

        assert_eq!(reload_dirs, vec![root.join("src")]);
        assert!(!state.nodes[&root.join("src")].children_loaded);
        assert_eq!(state.selected, Some(root.join("src/main.rs")));
    }

    #[test]
    fn apply_fs_invalidation_skips_collapsed_directories() {
        let root = PathBuf::from("/repo");
        let mut state = FileTreeState::at_root(root.clone());
        let _ = state.expand(&root);
        state.apply_children_loaded(
            &root,
            vec![DirectoryEntry {
                path: root.join("src"),
                name: "src".to_string(),
                is_dir: true,
            }],
            None,
        );

        let reload_dirs = state.apply_fs_invalidation(&root, &[root.join("src/new.rs")]);

        assert!(reload_dirs.is_empty());
        assert!(state.nodes[&root].children_loaded);
        assert!(!state.nodes[&root.join("src")].children_loaded);
    }

    #[test]
    fn apply_fs_invalidation_evicts_deleted_file_nodes() {
        let root = PathBuf::from("/repo");
        let mut state = FileTreeState::at_root(root.clone());
        let _ = state.expand(&root);
        state.apply_children_loaded(
            &root,
            vec![DirectoryEntry {
                path: root.join("src"),
                name: "src".to_string(),
                is_dir: true,
            }],
            None,
        );
        state
            .nodes
            .get_mut(&root.join("src"))
            .expect("src")
            .expanded = true;
        state.apply_children_loaded(
            &root.join("src"),
            vec![DirectoryEntry {
                path: root.join("src/main.rs"),
                name: "main.rs".to_string(),
                is_dir: false,
            }],
            None,
        );
        assert!(state.nodes.contains_key(&root.join("src/main.rs")));

        let reload_dirs = state.apply_fs_invalidation(&root, &[root.join("src/main.rs")]);

        assert_eq!(reload_dirs, vec![root.join("src")]);
        assert!(!state.nodes.contains_key(&root.join("src/main.rs")));
        assert!(!state.nodes[&root.join("src")].children_loaded);
    }

    #[test]
    fn directories_to_invalidate_covers_rename_paths() {
        let root = PathBuf::from("/repo");
        let dirs = directories_to_invalidate(
            &root,
            &[
                root.join("src/old.rs"),
                root.join("src/new.rs"),
            ],
            |_| false,
        );
        assert_eq!(dirs, vec![root.join("src")]);
    }
}
