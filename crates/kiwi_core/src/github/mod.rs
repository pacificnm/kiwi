//! GitHub domain types shared by events, state, and services.

mod browser;
mod context_menu;
mod labels;
mod pr_create;
mod selection;
mod types;

pub use browser::{
    browser_target_kind, missing_browser_target_message, page_scroll_issue_detail,
    resolve_browser_target, scroll_issue_detail, GitHubBrowserKind,
};
pub use context_menu::{format_issue_agent_prompt, format_pr_agent_prompt};
pub use context_menu::{GhContextMenuAction, GhContextMenuState, GhContextTarget};
pub use labels::apply_label_picker_load;
pub use pr_create::{advance_pr_create_prompt, PrCreatePromptAdvance};
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
