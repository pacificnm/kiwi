//! Lazy-loaded project file tree for the Explorer sidebar.

use nest_error::NestResult;
use nest_file::{DirEntry, FileService};
use std::collections::HashMap;

/// Explorer panel state.
#[derive(Debug, Clone)]
pub struct ExplorerState {
    /// Project display label.
    pub root_label: String,
    /// Directory names hidden from the tree.
    ignored: Vec<String>,
    /// Root tree node.
    pub tree: TreeNode,
    /// Selected file path relative to the project root.
    pub selected: Option<String>,
    /// Last panel-level error message.
    pub error: Option<String>,
}

/// One node in the explorer tree.
#[derive(Debug, Clone)]
pub struct TreeNode {
    /// Path relative to the project root (`"."` for root).
    pub rel_path: String,
    /// Display name.
    pub name: String,
    /// Whether this node is a directory.
    pub is_dir: bool,
    /// Expanded state for directories.
    pub expanded: bool,
    /// Child nodes after loading.
    pub children: Vec<TreeNode>,
    /// Whether [`TreeNode::children`] has been populated.
    pub children_loaded: bool,
    /// Error loading this directory, if any.
    pub error: Option<String>,
}

impl ExplorerState {
    /// Creates explorer state rooted at the project folder.
    pub fn new(
        root: &std::path::Path,
        root_label: impl Into<String>,
        ignored: Vec<String>,
    ) -> Self {
        let name = root
            .file_name()
            .and_then(|value| value.to_str())
            .unwrap_or("project")
            .to_string();
        Self {
            root_label: root_label.into(),
            ignored,
            tree: TreeNode {
                rel_path: ".".into(),
                name,
                is_dir: true,
                expanded: true,
                children: Vec::new(),
                children_loaded: false,
                error: None,
            },
            selected: None,
            error: None,
        }
    }

    /// Loads the root directory if needed.
    pub fn ensure_root_loaded(&mut self, files: &FileService) -> NestResult<()> {
        if !self.tree.children_loaded {
            load_children(files, &mut self.tree, &self.ignored)?;
        }
        Ok(())
    }

    /// Toggles a directory node, loading children on first expand.
    pub fn toggle_dir(&mut self, rel_path: &str, files: &FileService) -> NestResult<()> {
        let ignored = self.ignored.clone();
        let node = find_node_mut(&mut self.tree, rel_path).ok_or_else(|| {
            nest_error::NestError::validation(format!("unknown explorer node: {rel_path}"))
        })?;
        if !node.is_dir {
            return Ok(());
        }
        if node.expanded {
            node.expanded = false;
            return Ok(());
        }
        if !node.children_loaded {
            load_children(files, node, &ignored)?;
        }
        node.expanded = true;
        Ok(())
    }

    /// Reloads all currently expanded directories.
    pub fn refresh(&mut self, files: &FileService) -> NestResult<()> {
        self.error = None;
        refresh_node(files, &mut self.tree, &self.ignored)?;
        Ok(())
    }

    /// Marks a file as selected.
    pub fn select_file(&mut self, rel_path: impl Into<String>) {
        self.selected = Some(rel_path.into());
    }

    /// Clears expansion state for all directories.
    pub fn collapse_all(&mut self) {
        collapse_node(&mut self.tree);
    }
}

fn refresh_node(files: &FileService, node: &mut TreeNode, ignored: &[String]) -> NestResult<()> {
    if node.is_dir && node.children_loaded {
        load_children(files, node, ignored)?;
        if node.expanded {
            for child in &mut node.children {
                if child.is_dir && child.expanded {
                    refresh_node(files, child, ignored)?;
                }
            }
        }
    }
    Ok(())
}

fn collapse_node(node: &mut TreeNode) {
    if node.rel_path != "." {
        node.expanded = false;
    }
    node.children_loaded = false;
    node.children.clear();
    node.error = None;
}

fn load_children(files: &FileService, node: &mut TreeNode, ignored: &[String]) -> NestResult<()> {
    let previous: HashMap<String, TreeNode> = node
        .children
        .drain(..)
        .map(|child| (child.rel_path.clone(), child))
        .collect();

    let entries = files
        .list_dir(&node.rel_path)
        .map_err(|error| {
            node.error = Some(error.to_string());
            error
        })?;
    node.children = entries
        .into_iter()
        .filter(|entry| !should_ignore(&entry.name, ignored))
        .map(|entry| merge_dir_entry(&node.rel_path, entry, &previous))
        .collect();
    sort_nodes(&mut node.children);
    node.children_loaded = true;
    node.error = None;
    Ok(())
}

fn merge_dir_entry(
    parent_rel: &str,
    entry: DirEntry,
    previous: &HashMap<String, TreeNode>,
) -> TreeNode {
    let rel_path = if parent_rel == "." {
        entry.name.clone()
    } else {
        format!("{parent_rel}/{}", entry.name)
    };

    if let Some(mut old) = previous.get(&rel_path).cloned() {
        old.name = entry.name;
        old.is_dir = entry.metadata.is_dir;
        if !old.is_dir {
            old.expanded = false;
            old.children.clear();
            old.children_loaded = false;
        }
        old
    } else {
        dir_entry_to_node(parent_rel, entry)
    }
}

fn dir_entry_to_node(parent_rel: &str, entry: DirEntry) -> TreeNode {
    let rel_path = if parent_rel == "." {
        entry.name.clone()
    } else {
        format!("{parent_rel}/{}", entry.name)
    };
    TreeNode {
        rel_path,
        name: entry.name,
        is_dir: entry.metadata.is_dir,
        expanded: false,
        children: Vec::new(),
        children_loaded: false,
        error: None,
    }
}

fn sort_nodes(nodes: &mut [TreeNode]) {
    nodes.sort_by(|left, right| {
        match (left.is_dir, right.is_dir) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => left.name.to_lowercase().cmp(&right.name.to_lowercase()),
        }
    });
}

fn should_ignore(name: &str, ignored: &[String]) -> bool {
    ignored.iter().any(|entry| entry == name)
}

fn find_node_mut<'a>(node: &'a mut TreeNode, rel_path: &str) -> Option<&'a mut TreeNode> {
    if node.rel_path == rel_path {
        return Some(node);
    }
    for child in &mut node.children {
        if let Some(found) = find_node_mut(child, rel_path) {
            return Some(found);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::project::DEFAULT_IGNORE;
    use nest_file::{FileService, FileServiceConfig};
    use std::fs;
    use std::path::Path;
    use tempfile::tempdir;

    fn scoped_files(root: &Path) -> FileService {
        FileService::with_config(FileServiceConfig::scoped(root)).unwrap()
    }

    #[test]
    fn ignore_rules_hide_build_dirs() {
        let ignored: Vec<String> = DEFAULT_IGNORE.iter().map(|name| (*name).to_string()).collect();
        assert!(should_ignore("target", &ignored));
        assert!(should_ignore(".git", &ignored));
        assert!(!should_ignore("src", &ignored));
    }

    #[test]
    fn custom_ignore_hides_extra_dirs() {
        let ignored = vec!["vendor".into()];
        assert!(should_ignore("vendor", &ignored));
    }

    #[test]
    fn refresh_preserves_expanded_directories() {
        let dir = tempdir().unwrap();
        fs::create_dir_all(dir.path().join("src")).unwrap();
        fs::create_dir_all(dir.path().join("src/nested")).unwrap();
        fs::write(dir.path().join("Cargo.toml"), "workspace").unwrap();

        let files = scoped_files(dir.path());
        let ignored: Vec<String> = DEFAULT_IGNORE.iter().map(|name| (*name).to_string()).collect();
        let mut state = ExplorerState::new(dir.path(), "demo", ignored);
        state.ensure_root_loaded(&files).unwrap();
        state.toggle_dir("src", &files).unwrap();

        let src = state
            .tree
            .children
            .iter()
            .find(|node| node.name == "src")
            .expect("src directory");
        assert!(src.expanded);

        state.refresh(&files).unwrap();

        let src = state
            .tree
            .children
            .iter()
            .find(|node| node.name == "src")
            .expect("src directory after refresh");
        assert!(src.expanded);
        assert!(src.children_loaded);
    }

    #[test]
    fn lazy_load_lists_sorted_children() {
        let dir = tempdir().unwrap();
        fs::create_dir_all(dir.path().join("src")).unwrap();
        fs::write(dir.path().join("Cargo.toml"), "workspace").unwrap();
        fs::create_dir_all(dir.path().join("target")).unwrap();

        let files = scoped_files(dir.path());
        let ignored: Vec<String> = DEFAULT_IGNORE.iter().map(|name| (*name).to_string()).collect();
        let mut state = ExplorerState::new(dir.path(), "demo", ignored);
        state.ensure_root_loaded(&files).unwrap();

        let names: Vec<_> = state.tree.children.iter().map(|node| node.name.as_str()).collect();
        assert_eq!(names, vec!["src", "Cargo.toml"]);
    }
}
