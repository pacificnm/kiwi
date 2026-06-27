use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

use super::symlink::detect_symlink_loop;
use crate::file_tree::{DirectoryEntry, FileNode};
use crate::git::{GitFileEntry, GitFileStatus};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VisibleTreeRow {
    pub path: PathBuf,
    pub depth: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileTreeState {
    pub root: PathBuf,
    pub nodes: HashMap<PathBuf, FileNode>,
    pub children: HashMap<PathBuf, Vec<PathBuf>>,
    pub selected: Option<PathBuf>,
    pub scroll_offset: usize,
    pub loading: HashSet<PathBuf>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExpandAction {
    AlreadyExpanded,
    NeedsLoad,
}

impl FileTreeState {
    #[must_use]
    pub fn at_root(root: PathBuf) -> Self {
        let name = root
            .file_name()
            .map(|value| value.to_string_lossy().into_owned())
            .unwrap_or_else(|| root.display().to_string());

        let node = FileNode {
            path: root.clone(),
            name,
            is_dir: true,
            expanded: false,
            children_loaded: false,
            load_error: None,
            git_status: None,
        };

        let mut nodes = HashMap::new();
        nodes.insert(root.clone(), node);

        Self {
            root,
            nodes,
            children: HashMap::new(),
            selected: None,
            scroll_offset: 0,
            loading: HashSet::new(),
        }
    }

    pub fn expand(&mut self, path: &Path) -> Result<ExpandAction, String> {
        if detect_symlink_loop(&self.root, path).is_some() {
            return Err("symlink loop detected".to_string());
        }

        let Some(node) = self.nodes.get_mut(path) else {
            return Ok(ExpandAction::AlreadyExpanded);
        };

        if !node.is_dir {
            return Ok(ExpandAction::AlreadyExpanded);
        }

        node.expanded = true;
        if node.children_loaded || self.loading.contains(path) {
            return Ok(ExpandAction::AlreadyExpanded);
        }

        self.loading.insert(path.to_path_buf());
        Ok(ExpandAction::NeedsLoad)
    }

    pub fn collapse(&mut self, path: &Path) {
        if let Some(node) = self.nodes.get_mut(path) {
            node.expanded = false;
        }
    }

    pub fn select(&mut self, path: PathBuf) {
        if self.nodes.contains_key(&path) {
            self.selected = Some(path);
        }
    }

    pub fn ensure_selection(&mut self) {
        if self.selected.is_none() {
            if let Some(row) = self.visible_rows().first() {
                self.selected = Some(row.path.clone());
            }
        }
    }

    pub fn visible_rows(&self) -> Vec<VisibleTreeRow> {
        let mut rows = Vec::new();
        self.collect_visible_rows(&self.root, 0, &mut rows);
        rows
    }

    pub fn selected_row_index(&self) -> Option<usize> {
        let selected = self.selected.as_ref()?;
        self.visible_rows()
            .iter()
            .position(|row| &row.path == selected)
    }

    pub fn move_selection(&mut self, delta: i32, viewport_rows: usize) {
        let rows = self.visible_rows();
        if rows.is_empty() {
            self.selected = None;
            return;
        }

        if self.selected.is_none() {
            self.selected = Some(rows[0].path.clone());
        }

        let current = self
            .selected
            .as_ref()
            .and_then(|sel| rows.iter().position(|row| &row.path == sel))
            .unwrap_or(0);
        let len = rows.len() as i32;
        let next = (current as i32 + delta).clamp(0, len - 1) as usize;
        self.selected = Some(rows[next].path.clone());
        self.scroll_offset = self.scroll_offset_for_row(next, viewport_rows);
    }

    pub fn scroll_offset_for_row(&self, row_index: usize, viewport_rows: usize) -> usize {
        if viewport_rows == 0 {
            return 0;
        }
        if row_index < self.scroll_offset {
            row_index
        } else if row_index >= self.scroll_offset.saturating_add(viewport_rows) {
            row_index.saturating_sub(viewport_rows.saturating_sub(1))
        } else {
            self.scroll_offset
        }
    }

    pub fn row_at_viewport_index(&self, viewport_index: usize) -> Option<VisibleTreeRow> {
        self.visible_rows()
            .into_iter()
            .nth(self.scroll_offset.saturating_add(viewport_index))
    }

    /// Keep scroll offset valid when the visible row count shrinks (e.g. after workspace restore).
    pub fn clamp_scroll_to_viewport(&mut self, viewport_rows: usize) {
        let total = self.visible_rows().len();
        if total == 0 {
            self.scroll_offset = 0;
            return;
        }
        let viewport = viewport_rows.max(1);
        let max_offset = total.saturating_sub(viewport);
        if self.scroll_offset > max_offset {
            self.scroll_offset = max_offset;
        }
    }

    pub fn apply_children_loaded(
        &mut self,
        parent: &Path,
        children: Vec<DirectoryEntry>,
        error: Option<String>,
    ) {
        self.loading.remove(parent);

        let Some(node) = self.nodes.get_mut(parent) else {
            return;
        };

        node.children_loaded = true;
        node.load_error = error.clone();

        if error.is_some() {
            self.children.remove(parent);
            return;
        }

        let mut child_paths = Vec::with_capacity(children.len());
        for child in children {
            self.nodes.insert(
                child.path.clone(),
                FileNode {
                    path: child.path.clone(),
                    name: child.name,
                    is_dir: child.is_dir,
                    expanded: false,
                    children_loaded: false,
                    load_error: None,
                    git_status: None,
                },
            );
            child_paths.push(child.path);
        }

        self.children.insert(parent.to_path_buf(), child_paths);
    }

    pub fn invalidate_children(&mut self, path: &Path) {
        if let Some(child_paths) = self.children.remove(path) {
            for child in child_paths {
                if self.nodes.get(&child).is_some_and(|n| n.is_dir) {
                    self.invalidate_children(&child);
                }
                self.nodes.remove(&child);
            }
        }

        if let Some(node) = self.nodes.get_mut(path) {
            node.children_loaded = false;
            node.load_error = None;
        }
        self.loading.remove(path);
    }

    pub fn apply_git_status_patch(
        &mut self,
        repo_root: &Path,
        patch: &crate::git::GitFileStatusPatch,
        show_untracked: bool,
    ) {
        for path in &patch.removed {
            let full_path = repo_root.join(path);
            if let Some(node) = self.nodes.get_mut(&full_path) {
                node.git_status = None;
            }
        }

        for entry in &patch.changed {
            if !show_untracked && entry.status == GitFileStatus::Untracked {
                continue;
            }
            let full_path = repo_root.join(&entry.path);
            if let Some(node) = self.nodes.get_mut(&full_path) {
                if !node.is_dir {
                    node.git_status = Some(entry.status);
                }
            }
        }
    }

    pub fn apply_git_statuses(
        &mut self,
        repo_root: &Path,
        entries: &[GitFileEntry],
        show_untracked: bool,
    ) {
        let mut status_by_path = HashMap::new();
        for entry in entries {
            if !show_untracked && entry.status == GitFileStatus::Untracked {
                continue;
            }
            status_by_path.insert(entry.path.as_str(), entry.status);
        }

        for node in self.nodes.values_mut() {
            if node.is_dir {
                node.git_status = None;
                continue;
            }

            let Some(relative) = relative_repo_path(repo_root, &node.path) else {
                node.git_status = None;
                continue;
            };

            node.git_status = status_by_path.get(relative.as_str()).copied();
        }
    }

    fn collect_visible_rows(&self, path: &Path, depth: usize, rows: &mut Vec<VisibleTreeRow>) {
        let Some(node) = self.nodes.get(path) else {
            return;
        };

        rows.push(VisibleTreeRow {
            path: path.to_path_buf(),
            depth,
        });

        if !node.expanded || !node.children_loaded {
            return;
        }

        if let Some(children) = self.children.get(path) {
            for child in children {
                self.collect_visible_rows(child, depth + 1, rows);
            }
        }
    }

    #[must_use]
    #[cfg_attr(not(test), allow(dead_code))]
    pub fn selected_path_string(&self) -> Option<String> {
        self.selected
            .as_ref()
            .map(|path| path.display().to_string())
    }
}

fn relative_repo_path(repo_root: &Path, path: &Path) -> Option<String> {
    path.strip_prefix(repo_root)
        .ok()
        .map(|relative| relative.to_string_lossy().replace('\\', "/"))
        .filter(|relative| !relative.is_empty())
}

impl Default for FileTreeState {
    fn default() -> Self {
        Self::at_root(PathBuf::from("."))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn at_root_contains_only_root_node() {
        let state = FileTreeState::at_root(PathBuf::from("/tmp/kiwi"));
        assert_eq!(state.nodes.len(), 1);
        assert!(state.children.is_empty());
        assert!(!state.nodes[&state.root].children_loaded);
    }

    #[test]
    fn expand_requests_load_once() {
        let mut state = FileTreeState::at_root(PathBuf::from("/tmp/kiwi"));
        assert_eq!(
            state.expand(&state.root.clone()).expect("expand"),
            ExpandAction::NeedsLoad
        );
        assert!(state.loading.contains(&state.root));
        assert_eq!(
            state.expand(&state.root.clone()).expect("expand again"),
            ExpandAction::AlreadyExpanded
        );
    }

    #[test]
    fn apply_children_loaded_populates_nodes_and_cache() {
        let root = PathBuf::from("/tmp/kiwi");
        let mut state = FileTreeState::at_root(root.clone());
        let _ = state.expand(&root);

        state.apply_children_loaded(
            &root,
            vec![
                DirectoryEntry {
                    path: root.join("src"),
                    name: "src".to_string(),
                    is_dir: true,
                },
                DirectoryEntry {
                    path: root.join("Cargo.toml"),
                    name: "Cargo.toml".to_string(),
                    is_dir: false,
                },
            ],
            None,
        );

        assert!(state.nodes[&root].children_loaded);
        assert_eq!(state.children[&root].len(), 2);
        assert!(state.nodes.contains_key(&root.join("src")));
        assert!(!state.loading.contains(&root));
    }

    #[test]
    fn move_selection_updates_selected_path_and_scroll() {
        let root = PathBuf::from("/tmp/kiwi");
        let mut state = FileTreeState::at_root(root.clone());
        populate_two_level_tree(&mut state, &root);

        state.move_selection(1, 2);
        assert_eq!(state.selected, Some(root.join("src")));
        assert_eq!(state.scroll_offset, 0);

        state.move_selection(1, 1);
        assert_eq!(state.selected, Some(root.join("src/main.rs")));
        assert_eq!(state.scroll_offset, 2);
    }

    #[test]
    fn clamp_scroll_to_viewport_reduces_stale_offset() {
        let root = PathBuf::from("/tmp/kiwi");
        let mut state = FileTreeState::at_root(root.clone());
        state.scroll_offset = 50;
        state.clamp_scroll_to_viewport(5);
        assert_eq!(state.scroll_offset, 0);
    }

    #[test]
    fn visible_rows_flattens_expanded_tree_with_depth() {
        let root = PathBuf::from("/tmp/kiwi");
        let mut state = FileTreeState::at_root(root.clone());
        populate_two_level_tree(&mut state, &root);

        let rows = state.visible_rows();
        assert_eq!(rows.len(), 3);
        assert_eq!(rows[0].depth, 0);
        assert_eq!(rows[1].depth, 1);
        assert_eq!(rows[2].depth, 2);
    }

    fn populate_two_level_tree(state: &mut FileTreeState, root: &Path) {
        let _ = state.expand(root);
        state.apply_children_loaded(
            root,
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
        state.ensure_selection();
    }

    #[test]
    fn visible_entries_flattens_expanded_tree() {
        let root = PathBuf::from("/tmp/kiwi");
        let mut state = FileTreeState::at_root(root.clone());
        populate_two_level_tree(&mut state, &root);

        let names: Vec<_> = state
            .visible_rows()
            .iter()
            .map(|row| state.nodes[&row.path].name.as_str())
            .collect();
        assert_eq!(names, vec!["kiwi", "src", "main.rs"]);
    }

    #[test]
    fn apply_git_status_patch_updates_only_changed_paths() {
        let root = PathBuf::from("/tmp/kiwi");
        let mut state = FileTreeState::at_root(root.clone());
        let _ = state.expand(&root);
        state.apply_children_loaded(
            &root,
            vec![
                DirectoryEntry {
                    path: root.join("src/main.rs"),
                    name: "main.rs".to_string(),
                    is_dir: false,
                },
                DirectoryEntry {
                    path: root.join("src/other.rs"),
                    name: "other.rs".to_string(),
                    is_dir: false,
                },
            ],
            None,
        );
        state
            .nodes
            .get_mut(&root.join("src/main.rs"))
            .unwrap()
            .git_status = Some(GitFileStatus::Modified);
        state
            .nodes
            .get_mut(&root.join("src/other.rs"))
            .unwrap()
            .git_status = Some(GitFileStatus::Modified);

        state.apply_git_status_patch(
            &root,
            &crate::git::GitFileStatusPatch {
                changed: vec![GitFileEntry {
                    path: "src/other.rs".to_string(),
                    status: GitFileStatus::Added,
                }],
                removed: vec!["src/main.rs".to_string()],
            },
            true,
        );

        assert_eq!(state.nodes[&root.join("src/main.rs")].git_status, None);
        assert_eq!(
            state.nodes[&root.join("src/other.rs")].git_status,
            Some(GitFileStatus::Added)
        );
    }

    #[test]
    fn apply_git_statuses_sets_file_badges() {
        let root = PathBuf::from("/tmp/kiwi");
        let mut state = FileTreeState::at_root(root.clone());
        let _ = state.expand(&root);
        state.apply_children_loaded(
            &root,
            vec![
                DirectoryEntry {
                    path: root.join("src/main.rs"),
                    name: "main.rs".to_string(),
                    is_dir: false,
                },
                DirectoryEntry {
                    path: root.join("README.md"),
                    name: "README.md".to_string(),
                    is_dir: false,
                },
            ],
            None,
        );

        state.apply_git_statuses(
            &root,
            &[
                GitFileEntry {
                    path: "src/main.rs".to_string(),
                    status: GitFileStatus::Modified,
                },
                GitFileEntry {
                    path: "README.md".to_string(),
                    status: GitFileStatus::Untracked,
                },
            ],
            true,
        );

        assert_eq!(
            state.nodes[&root.join("src/main.rs")].git_status,
            Some(GitFileStatus::Modified)
        );
        assert_eq!(
            state.nodes[&root.join("README.md")].git_status,
            Some(GitFileStatus::Untracked)
        );
    }

    #[test]
    fn invalidate_children_recurses_without_filesystem_stat() {
        // Verifies that invalidate_children uses in-memory FileNode.is_dir
        // rather than Path::is_dir() (which would be a blocking stat syscall).
        // Paths are non-existent so any real stat would return false and
        // break the recursion — but with the in-memory check it still works.
        let root = PathBuf::from("/nonexistent/repo");
        let mut state = FileTreeState::at_root(root.clone());

        // Simulate a loaded tree: root → src (dir) → main.rs (file)
        state.apply_children_loaded(
            &root,
            vec![DirectoryEntry {
                path: root.join("src"),
                name: "src".to_string(),
                is_dir: true,
            }],
            None,
        );
        state.apply_children_loaded(
            &root.join("src"),
            vec![DirectoryEntry {
                path: root.join("src/main.rs"),
                name: "main.rs".to_string(),
                is_dir: false,
            }],
            None,
        );

        assert!(state.nodes.contains_key(&root.join("src")));
        assert!(state.nodes.contains_key(&root.join("src/main.rs")));
        assert!(state.nodes[&root.join("src")].is_dir);

        state.invalidate_children(&root);

        // src and main.rs nodes removed, children caches cleared
        assert!(!state.nodes.contains_key(&root.join("src")));
        assert!(!state.nodes.contains_key(&root.join("src/main.rs")));
        assert!(!state.children.contains_key(&root));
        assert!(!state.children.contains_key(&root.join("src")));
        // root node itself still exists but marked not loaded
        assert!(!state.nodes[&root].children_loaded);
    }

    #[test]
    fn apply_git_statuses_hides_untracked_when_disabled() {
        let root = PathBuf::from("/tmp/kiwi");
        let mut state = FileTreeState::at_root(root.clone());
        let _ = state.expand(&root);
        state.apply_children_loaded(
            &root,
            vec![DirectoryEntry {
                path: root.join("new.txt"),
                name: "new.txt".to_string(),
                is_dir: false,
            }],
            None,
        );

        state.apply_git_statuses(
            &root,
            &[GitFileEntry {
                path: "new.txt".to_string(),
                status: GitFileStatus::Untracked,
            }],
            false,
        );

        assert_eq!(state.nodes[&root.join("new.txt")].git_status, None);
    }
}
