use std::path::PathBuf;

use crate::git::GitFileStatus;

mod classify;
mod ignore;
mod invalidation;
mod io;
mod loader;
mod state;
mod symlink;

pub use classify::{file_type_category, FileTypeCategory};
pub use ignore::{is_default_ignored, DEFAULT_IGNORED_NAMES};
pub use invalidation::directories_to_invalidate;
pub use io::spawn_directory_load;
pub use loader::{read_directory_children, sort_directory_entries, DirectoryLoadResult};
pub use state::{ExpandAction, FileTreeState, VisibleTreeRow};
pub use symlink::{detect_symlink_loop, MAX_EXPAND_DEPTH};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DirectoryEntry {
    pub path: PathBuf,
    pub name: String,
    pub is_dir: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileNode {
    pub path: PathBuf,
    pub name: String,
    pub is_dir: bool,
    pub expanded: bool,
    pub children_loaded: bool,
    pub load_error: Option<String>,
    pub git_status: Option<GitFileStatus>,
}
