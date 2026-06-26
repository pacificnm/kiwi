//! Per-repository workspace persistence (SPEC-017, ADR-016).

mod persistence;
mod snapshot;

pub use persistence::{
    load_palette_history, load_snapshot, repo_hash, save_from_reduce_view, save_palette_history,
    save_snapshot, try_load_workspace, try_save_from_reduce_view, workspace_file_path,
};
pub use snapshot::{scroll_view, trim_history, WorkspaceSnapshot, WORKSPACE_SCHEMA_VERSION};
