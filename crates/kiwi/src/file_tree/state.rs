use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

use super::loader::detect_symlink_loop;
use super::node::{DirectoryEntry, FileNode};

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
                },
            );
            child_paths.push(child.path);
        }

        self.children.insert(parent.to_path_buf(), child_paths);
    }

    pub fn invalidate_children(&mut self, path: &Path) {
        if let Some(child_paths) = self.children.remove(path) {
            for child in child_paths {
                if child.is_dir() {
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

    #[cfg_attr(not(test), allow(dead_code))]
    pub fn visible_entries(&self) -> Vec<&FileNode> {
        let mut visible = Vec::new();
        self.collect_visible(&self.root, &mut visible);
        visible
    }

    #[cfg_attr(not(test), allow(dead_code))]
    fn collect_visible<'a>(&'a self, path: &Path, visible: &mut Vec<&'a FileNode>) {
        let Some(node) = self.nodes.get(path) else {
            return;
        };

        visible.push(node);
        if !node.expanded || !node.children_loaded {
            return;
        }

        if let Some(children) = self.children.get(path) {
            for child in children {
                self.collect_visible(child, visible);
            }
        }
    }

    #[must_use]
    pub fn selected_path_string(&self) -> Option<String> {
        self.selected
            .as_ref()
            .map(|path| path.display().to_string())
    }
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
    fn visible_entries_flattens_expanded_tree() {
        let root = PathBuf::from("/tmp/kiwi");
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

        let names: Vec<_> = state
            .visible_entries()
            .iter()
            .map(|node| node.name.as_str())
            .collect();
        assert_eq!(names, vec!["kiwi", "src", "main.rs"]);
    }
}
