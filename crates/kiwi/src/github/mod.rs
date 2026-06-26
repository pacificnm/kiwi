//! TUI-facing GitHub service API; domain logic lives in [`kiwi_core::github`].

#[allow(unused_imports)]
#[cfg(test)]
pub use kiwi_core::github::{
    apply_label_picker_load, browser_target_kind, resolve_browser_target, IssueDetail,
    LabelPickerState, PrDetail, RepoLabelsLoadResult,
};
#[allow(unused_imports)]
pub use kiwi_core::github::{
    issue_at_viewport, issue_selected_row_index, pr_at_viewport, pr_selected_row_index,
    GhContextMenuState, GhContextTarget, GitHubAuthErrorKind, GitHubLeftPane, Issue,
    IssueActionResult, IssueState, PrState, PullRequest, RepoLabel,
};
#[allow(unused_imports)]
pub use kiwi_core::github::{
    spawn_github_auth_check, spawn_github_issue_comment, spawn_github_issue_create_branch,
    spawn_github_issue_detail_load, spawn_github_issue_label_apply, spawn_github_issue_list_load,
    spawn_github_open_browser, spawn_github_pr_create, spawn_github_pr_detail_load,
    spawn_github_pr_list_load, spawn_github_pr_merge, spawn_github_repo_labels_load,
};
