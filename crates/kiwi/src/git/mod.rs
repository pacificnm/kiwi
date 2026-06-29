//! TUI-facing git service API; domain logic lives in [`kiwi_core::git`].

#[allow(unused_imports)]
#[cfg(test)]
pub use kiwi_core::git::GitFileStatus;
#[allow(unused_imports)]
pub use kiwi_core::git::{
    branch_row_at_viewport, branch_selected_name, branch_selected_row_index, build_panel_rows,
    checkout_local_branch, git_row_at_viewport, git_selected_row_index, list_local_branches,
    spawn_branch_checkout, spawn_branch_list, spawn_git_refresh, BranchDetail, BranchEntry,
    GitFileEntry, GitPanelRow,
};
