//! Per-repository workspace persistence (SPEC-017, ADR-016).

mod persistence;
mod snapshot;

pub use persistence::{
    load_palette_history, load_snapshot, load_workspace_file, merge_save_gui, merge_save_tui,
    repo_hash, save_from_reduce_view, save_palette_history, save_snapshot, save_workspace_file,
    try_load_workspace, try_load_workspace_file, try_merge_save_gui, try_save_from_reduce_view,
    workspace_file_path,
};
pub use snapshot::{
    scroll_view, trim_history, GuiWorkspaceSnapshot, TuiWorkspaceSnapshot, WorkspaceFile,
    WorkspaceSnapshot, WORKSPACE_SCHEMA_VERSION, WORKSPACE_SCHEMA_VERSION_V1,
};
