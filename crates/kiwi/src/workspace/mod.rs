//! TUI-facing workspace persistence API; domain types live in [`kiwi_core::workspace`].

#[allow(unused_imports)]
pub use kiwi_core::state::MAX_PALETTE_HISTORY_ENTRIES;
#[allow(unused_imports)]
pub use kiwi_core::workspace::{
    load_palette_history, load_snapshot, repo_hash, save_palette_history, save_snapshot,
    scroll_view, trim_history, try_load_workspace, workspace_file_path, WorkspaceSnapshot,
    WORKSPACE_SCHEMA_VERSION,
};

use crate::state::AppState;

#[allow(dead_code)]
pub fn save_workspace_from_state(state: &mut AppState) -> std::io::Result<()> {
    kiwi_core::workspace::save_from_reduce_view(&state.reduce_view())
}

/// Persist current app state when `workspace.persist` is enabled (SPEC-017).
pub fn try_save_workspace_from_state(state: &mut AppState) {
    kiwi_core::workspace::try_save_from_reduce_view(&state.reduce_view());
}
