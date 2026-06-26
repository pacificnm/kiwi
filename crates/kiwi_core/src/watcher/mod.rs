mod debounce;
mod io;
mod paths;

pub use debounce::{coalesce_paths, PathDebouncer};
pub use io::RepoWatcher;
pub use paths::{
    is_git_metadata_watch_path, path_matches_file, preview_reload_paths,
    should_emit_fs_changed_event, should_ignore_watch_path,
};
