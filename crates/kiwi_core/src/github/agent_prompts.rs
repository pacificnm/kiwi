use crate::github::IssueDetail;

const BODY_EXCERPT_MAX_CHARS: usize = 400;

#[must_use]
pub fn format_issue_agent_prompt(number: u32, title: &str, body_excerpt: Option<&str>) -> String {
    let mut prompt = format!(
        "Please help me work on GitHub issue #{number}: {title}\n\n\
         Review the issue context and suggest an implementation approach.\n"
    );

    if let Some(body) = body_excerpt.filter(|text| !text.trim().is_empty()) {
        prompt.push_str("\nIssue description:\n");
        prompt.push_str(body.trim());
        prompt.push('\n');
    }

    prompt
}

#[must_use]
pub fn format_pr_agent_prompt(number: u32, title: &str) -> String {
    format!(
        "Please help me review GitHub PR #{number}: {title}\n\n\
         Summarize the changes and suggest improvements.\n"
    )
}

#[must_use]
pub fn issue_body_excerpt_from_detail(detail: &IssueDetail) -> Option<String> {
    let body = detail.body.as_deref()?.trim();
    if body.is_empty() {
        return None;
    }
    Some(truncate_excerpt(body, BODY_EXCERPT_MAX_CHARS))
}

fn truncate_excerpt(text: &str, max_chars: usize) -> String {
    if text.chars().count() <= max_chars {
        return text.to_string();
    }
    let truncated: String = text.chars().take(max_chars).collect();
    format!("{truncated}…")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::github::{IssueState};

    #[test]
    fn format_issue_agent_prompt_uses_title() {
        let prompt = format_issue_agent_prompt(42, "Add context menu", None);
        assert!(prompt.contains("#42"));
        assert!(prompt.contains("Add context menu"));
    }

    #[test]
    fn format_issue_agent_prompt_includes_body_excerpt() {
        let prompt = format_issue_agent_prompt(42, "Add context menu", Some("Fix GH list UX"));
        assert!(prompt.contains("Issue description:"));
        assert!(prompt.contains("Fix GH list UX"));
    }

    #[test]
    fn issue_body_excerpt_from_detail_returns_none_when_no_body() {
        let detail = IssueDetail {
            number: 1,
            title: "Test".to_string(),
            state: IssueState::Open,
            author: "dev".to_string(),
            labels: Vec::new(),
            assignees: Vec::new(),
            body: None,
            display_lines: Vec::new(),
        };
        assert!(issue_body_excerpt_from_detail(&detail).is_none());
    }

    #[test]
    fn issue_body_excerpt_from_detail_returns_none_for_whitespace_body() {
        let detail = IssueDetail {
            number: 1,
            title: "Test".to_string(),
            state: IssueState::Open,
            author: "dev".to_string(),
            labels: Vec::new(),
            assignees: Vec::new(),
            body: Some("   ".to_string()),
            display_lines: Vec::new(),
        };
        assert!(issue_body_excerpt_from_detail(&detail).is_none());
    }

    #[test]
    fn issue_body_excerpt_from_detail_truncates_long_body() {
        let body = "a".repeat(500);
        let detail = IssueDetail {
            number: 1,
            title: "Test".to_string(),
            state: IssueState::Open,
            author: "dev".to_string(),
            labels: Vec::new(),
            assignees: Vec::new(),
            body: Some(body),
            display_lines: Vec::new(),
        };
        let excerpt = issue_body_excerpt_from_detail(&detail).expect("excerpt");
        assert!(excerpt.ends_with('…'));
        assert!(excerpt.chars().count() <= BODY_EXCERPT_MAX_CHARS + 1);
    }

    #[test]
    fn format_pr_agent_prompt_uses_title() {
        let prompt = format_pr_agent_prompt(7, "GUI follow-up");
        assert!(prompt.contains("#7"));
        assert!(prompt.contains("GUI follow-up"));
    }
}
