use crate::state::GitHubState;

use super::types::{Issue, PrState, PullRequest};

pub fn ensure_issue_selection(state: &mut GitHubState) {
    if state.issues.is_empty() {
        state.selected_issue = None;
        state.issues_scroll_offset = 0;
        return;
    }

    if let Some(number) = state.selected_issue {
        if state
            .issues
            .iter()
            .any(|issue| u64::from(issue.number) == number)
        {
            return;
        }
    }

    state.selected_issue = Some(u64::from(state.issues[0].number));
    state.issues_scroll_offset = 0;
}

pub fn issue_move_selection(state: &mut GitHubState, delta: i32, viewport_rows: usize) {
    if state.issues.is_empty() || viewport_rows == 0 {
        return;
    }

    let current_index = state
        .selected_issue
        .and_then(|number| {
            state
                .issues
                .iter()
                .position(|issue| u64::from(issue.number) == number)
        })
        .unwrap_or(0);

    let next_index = (current_index as i32 + delta)
        .clamp(0, state.issues.len().saturating_sub(1) as i32) as usize;
    let issue = &state.issues[next_index];
    state.selected_issue = Some(u64::from(issue.number));
    state.issues_scroll_offset =
        scroll_offset_for_row(next_index, state.issues_scroll_offset, viewport_rows);
}

pub fn issue_select_row(state: &mut GitHubState, row_index: usize, viewport_rows: usize) {
    if state.issues.get(row_index).is_none() {
        return;
    }

    let issue = &state.issues[row_index];
    state.selected_issue = Some(u64::from(issue.number));
    state.issues_scroll_offset =
        scroll_offset_for_row(row_index, state.issues_scroll_offset, viewport_rows);
}

pub fn issue_at_viewport(state: &GitHubState, viewport_index: usize) -> Option<&Issue> {
    state
        .issues
        .get(state.issues_scroll_offset.saturating_add(viewport_index))
}

pub fn issue_selected_row_index(state: &GitHubState) -> Option<usize> {
    let number = state.selected_issue?;
    state
        .issues
        .iter()
        .position(|issue| u64::from(issue.number) == number)
}

pub fn ensure_pr_selection(state: &mut GitHubState) {
    if state.prs.is_empty() {
        state.selected_pr = None;
        state.prs_scroll_offset = 0;
        return;
    }

    if let Some(number) = state.selected_pr {
        if state.prs.iter().any(|pr| u64::from(pr.number) == number) {
            return;
        }
    }

    state.selected_pr = Some(u64::from(state.prs[0].number));
    state.prs_scroll_offset = 0;
}

pub fn pr_move_selection(state: &mut GitHubState, delta: i32, viewport_rows: usize) {
    if state.prs.is_empty() || viewport_rows == 0 {
        return;
    }

    let current_index = state
        .selected_pr
        .and_then(|number| {
            state
                .prs
                .iter()
                .position(|pr| u64::from(pr.number) == number)
        })
        .unwrap_or(0);

    let next_index =
        (current_index as i32 + delta).clamp(0, state.prs.len().saturating_sub(1) as i32) as usize;
    let pr = &state.prs[next_index];
    state.selected_pr = Some(u64::from(pr.number));
    state.prs_scroll_offset =
        scroll_offset_for_row(next_index, state.prs_scroll_offset, viewport_rows);
}

pub fn pr_select_row(state: &mut GitHubState, row_index: usize, viewport_rows: usize) {
    if state.prs.get(row_index).is_none() {
        return;
    }

    let pr = &state.prs[row_index];
    state.selected_pr = Some(u64::from(pr.number));
    state.prs_scroll_offset =
        scroll_offset_for_row(row_index, state.prs_scroll_offset, viewport_rows);
}

pub fn pr_at_viewport(state: &GitHubState, viewport_index: usize) -> Option<&PullRequest> {
    state
        .prs
        .get(state.prs_scroll_offset.saturating_add(viewport_index))
}

pub fn pr_selected_row_index(state: &GitHubState) -> Option<usize> {
    let number = state.selected_pr?;
    state
        .prs
        .iter()
        .position(|pr| u64::from(pr.number) == number)
}

#[must_use]
pub fn selected_pull_request(state: &GitHubState) -> Option<&PullRequest> {
    let number = u32::try_from(state.selected_pr?).ok()?;
    state.prs.iter().find(|pr| pr.number == number)
}

#[must_use]
pub fn pull_request_is_mergeable(pr: &PullRequest) -> bool {
    pr.state == PrState::Open && !pr.is_draft
}

pub fn scroll_offset_for_row(
    selected_row: usize,
    scroll_offset: usize,
    viewport_rows: usize,
) -> usize {
    if viewport_rows == 0 {
        return 0;
    }

    if selected_row < scroll_offset {
        selected_row
    } else if selected_row >= scroll_offset.saturating_add(viewport_rows) {
        selected_row.saturating_sub(viewport_rows.saturating_sub(1))
    } else {
        scroll_offset
    }
}
