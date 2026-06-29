mod branch_ops;
mod branch_selection;
mod branches;
mod io;
mod panel;
mod patch;
mod repository;
mod selection;
mod status;

pub use branch_ops::{checkout_local_branch, list_local_branches, load_branch_detail};
pub use branch_selection::{
    branch_move_selection, branch_row_at_viewport, branch_select_row, branch_selected_name,
    branch_selected_row_index, ensure_branch_selection,
};
pub use branches::{BranchDetail, BranchEntry};
pub use io::{
    load_branch_list_snapshot, load_git_snapshot, spawn_branch_checkout, spawn_branch_detail,
    spawn_branch_list, spawn_git_refresh, BranchListSnapshot, GitRefreshSnapshot,
};
pub use panel::{
    adjacent_changed_file, build_panel_rows, changed_file_paths, row_for_path, GitPanelRow,
};
pub use patch::{diff_git_file_entries, patch_git_file_entries, GitFileStatusPatch};
pub use repository::{
    load_branch_info, load_repo_snapshot, parse_remote_repo_slug, GitBranchInfo, GitError,
    GitRepoSnapshot,
};
pub use selection::{
    clamp_git_scroll, ensure_git_selection, git_move_selection, git_row_at_viewport,
    git_select_row, git_selected_row_index,
};
pub use status::{GitFileEntry, GitFileStatus};
