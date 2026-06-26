#[cfg(test)]
pub use kiwi_core::git::GitFileStatus;
pub use kiwi_core::git::{
    branch_row_at_viewport, branch_selected_row_index, build_panel_rows, git_row_at_viewport,
    git_selected_row_index, BranchEntry, GitFileEntry, GitPanelRow,
};

mod branches;
mod io;
mod patch;
mod repository;
mod status;

pub use io::{spawn_branch_checkout, spawn_branch_list, spawn_git_refresh};
