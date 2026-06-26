mod actions;
mod auth;
mod browser;
mod command;
mod context_menu;
mod create_pr;
mod detail;
mod io;
mod issue;
mod labels;
mod open_browser;
mod pr;
mod pr_create;
mod pr_detail;
mod pr_merge;
mod repo_labels;
mod selection;
mod types;

pub use actions::{add_issue_labels, create_branch_from_issue, post_issue_comment};
pub use auth::{check_github_auth, AUTH_LOGIN_URL, INSTALL_URL};
pub use browser::{
    browser_target_kind, missing_browser_target_message, page_scroll_issue_detail,
    resolve_browser_target, scroll_issue_detail, GitHubBrowserKind,
};
pub use context_menu::{format_issue_agent_prompt, format_pr_agent_prompt};
pub use context_menu::{GhContextMenuAction, GhContextMenuState, GhContextTarget};
pub use create_pr::create_pull_request;
pub use detail::load_issue_detail;
pub use io::{
    spawn_github_auth_check, spawn_github_issue_comment, spawn_github_issue_create_branch,
    spawn_github_issue_detail_load, spawn_github_issue_label_apply, spawn_github_issue_list_load,
    spawn_github_open_browser, spawn_github_pr_create, spawn_github_pr_detail_load,
    spawn_github_pr_list_load, spawn_github_pr_merge, spawn_github_repo_labels_load,
};
pub use issue::load_issue_list;
pub use labels::apply_label_picker_load;
pub use open_browser::open_in_browser;
pub use pr::load_pr_list;
pub use pr_create::{advance_pr_create_prompt, PrCreatePromptAdvance};
pub use pr_detail::load_pr_detail;
pub use pr_merge::merge_pull_request;
pub use repo_labels::load_repo_labels;
pub use selection::{
    ensure_issue_selection, ensure_pr_selection, issue_at_viewport, issue_move_selection,
    issue_select_row, issue_selected_row_index, pr_at_viewport, pr_move_selection, pr_select_row,
    pr_selected_row_index, pull_request_is_mergeable, selected_pull_request,
};
pub use types::{
    GitHubAuthCheckResult, GitHubAuthErrorKind, GitHubBrowserTarget, GitHubLeftPane, Issue,
    IssueActionResult, IssueComment, IssueDetail, IssueDetailLoadResult, IssueListLoadResult,
    IssueState, LabelPickerState, PrCreateRequest, PrDetail, PrDetailLoadResult, PrListLoadResult,
    PrState, PullRequest, RepoLabel, RepoLabelsLoadResult, ISSUE_LIST_CACHE_SECS,
    PR_LIST_CACHE_SECS,
};
