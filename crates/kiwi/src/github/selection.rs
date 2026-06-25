use crate::state::GitHubState;

use super::issue::Issue;

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::github::IssueState;

    fn sample_issues() -> Vec<Issue> {
        vec![
            Issue {
                number: 1,
                title: "First".to_string(),
                state: IssueState::Open,
                labels: Vec::new(),
                assignees: Vec::new(),
            },
            Issue {
                number: 2,
                title: "Second".to_string(),
                state: IssueState::Open,
                labels: Vec::new(),
                assignees: Vec::new(),
            },
        ]
    }

    #[test]
    fn move_selection_updates_selected_issue() {
        let mut state = GitHubState {
            issues: sample_issues(),
            selected_issue: Some(1),
            ..GitHubState::default()
        };

        issue_move_selection(&mut state, 1, 10);
        assert_eq!(state.selected_issue, Some(2));
    }

    #[test]
    fn ensure_issue_selection_preserves_valid_selection() {
        let mut state = GitHubState {
            issues: sample_issues(),
            selected_issue: Some(2),
            issues_scroll_offset: 4,
            ..GitHubState::default()
        };

        ensure_issue_selection(&mut state);
        assert_eq!(state.selected_issue, Some(2));
        assert_eq!(state.issues_scroll_offset, 4);
    }
}
