mod branch_selection;
mod branches;
mod panel;
mod patch;
mod selection;
mod status;

pub use branch_selection::{
    branch_move_selection, branch_row_at_viewport, branch_select_row, branch_selected_name,
    branch_selected_row_index, ensure_branch_selection,
};
pub use branches::BranchEntry;
pub use panel::{
    adjacent_changed_file, build_panel_rows, changed_file_paths, row_for_path, GitPanelRow,
};
pub use patch::{diff_git_file_entries, patch_git_file_entries, GitFileStatusPatch};
pub use selection::{
    ensure_git_selection, git_move_selection, git_row_at_viewport, git_select_row,
    git_selected_row_index,
};
pub use status::{GitFileEntry, GitFileStatus};
