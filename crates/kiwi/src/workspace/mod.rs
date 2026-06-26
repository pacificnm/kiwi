mod persistence;
mod snapshot;

pub const MAX_PALETTE_HISTORY_ENTRIES: usize = 50;

pub use persistence::{load_palette_history, save_palette_history};
#[allow(unused_imports)]
pub use persistence::{load_snapshot, repo_hash, save_snapshot, workspace_file_path};
#[allow(unused_imports)]
pub use snapshot::{scroll_view, trim_history, WorkspaceSnapshot, WORKSPACE_SCHEMA_VERSION};
