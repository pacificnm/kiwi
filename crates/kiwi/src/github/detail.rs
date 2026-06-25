use std::path::Path;
use std::process::Command;

use serde::Deserialize;

use super::issue::{command_on_path, IssueState};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IssueComment {
    pub author: String,
    pub body: String,
    pub created_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IssueDetail {
    pub number: u32,
    pub title: String,
    pub state: IssueState,
    pub author: String,
    pub labels: Vec<String>,
    pub assignees: Vec<String>,
    pub display_lines: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IssueDetailLoadResult {
    pub detail: Option<IssueDetail>,
    pub error: Option<String>,
}

#[derive(Debug, Deserialize)]
struct GhIssueDetail {
    number: u32,
    title: String,
    body: String,
    state: String,
    labels: Vec<GhLabel>,
    assignees: Vec<GhUser>,
    author: GhUser,
    comments: Vec<GhComment>,
}

#[derive(Debug, Deserialize)]
struct GhComment {
    author: GhUser,
    body: String,
    #[serde(rename = "createdAt")]
    created_at: String,
}

#[derive(Debug, Deserialize)]
struct GhLabel {
    name: String,
}

#[derive(Debug, Deserialize)]
struct GhUser {
    login: String,
}

pub fn load_issue_detail(repo_root: &Path, command: &str, number: u32) -> IssueDetailLoadResult {
    if !command_on_path(command) {
        return IssueDetailLoadResult {
            detail: None,
            error: Some(format!("GitHub CLI ({command}) not found on PATH")),
        };
    }

    let output = Command::new(command)
        .args([
            "issue",
            "view",
            &number.to_string(),
            "--json",
            "number,title,body,state,labels,assignees,comments,author,createdAt",
        ])
        .current_dir(repo_root)
        .output();

    match output {
        Ok(result) if result.status.success() => match parse_issue_detail_json(&result.stdout) {
            Ok(detail) => IssueDetailLoadResult {
                detail: Some(detail),
                error: None,
            },
            Err(message) => IssueDetailLoadResult {
                detail: None,
                error: Some(message),
            },
        },
        Ok(result) => IssueDetailLoadResult {
            detail: None,
            error: Some(format_issue_detail_failure(&result.stderr, &result.stdout)),
        },
        Err(err) => IssueDetailLoadResult {
            detail: None,
            error: Some(format!("Failed to run `{command} issue view`: {err}")),
        },
    }
}

fn parse_issue_detail_json(bytes: &[u8]) -> Result<IssueDetail, String> {
    let raw: GhIssueDetail =
        serde_json::from_slice(bytes).map_err(|err| format!("Invalid gh issue JSON: {err}"))?;

    let labels: Vec<String> = raw.labels.into_iter().map(|label| label.name).collect();
    let assignees: Vec<String> = raw.assignees.into_iter().map(|user| user.login).collect();
    let comments: Vec<IssueComment> = raw
        .comments
        .into_iter()
        .map(|comment| IssueComment {
            author: comment.author.login,
            body: comment.body,
            created_at: format_comment_date(&comment.created_at),
        })
        .collect();
    let state = IssueState::parse(&raw.state);
    let draft = IssueDetailDraft {
        number: raw.number,
        title: raw.title,
        state,
        author: raw.author.login,
        labels,
        assignees,
        body: raw.body,
        comments,
    };
    let display_lines = build_display_lines(&draft);

    Ok(IssueDetail {
        number: draft.number,
        title: draft.title,
        state: draft.state,
        author: draft.author,
        labels: draft.labels,
        assignees: draft.assignees,
        display_lines,
    })
}

struct IssueDetailDraft {
    number: u32,
    title: String,
    state: IssueState,
    author: String,
    labels: Vec<String>,
    assignees: Vec<String>,
    body: String,
    comments: Vec<IssueComment>,
}

fn build_display_lines(detail: &IssueDetailDraft) -> Vec<String> {
    let mut lines = vec![
        format!("#{} {}", detail.number, detail.title),
        format!(
            "State: {} · Author: {}",
            detail.state.label(),
            detail.author
        ),
    ];

    if !detail.labels.is_empty() {
        lines.push(format!("Labels: {}", detail.labels.join(", ")));
    }

    if !detail.assignees.is_empty() {
        lines.push(format!("Assignees: {}", detail.assignees.join(", ")));
    }

    lines.push(String::new());
    lines.push("— Body —".to_string());
    lines.push(String::new());
    extend_body_lines(&mut lines, &detail.body);

    lines.push(String::new());
    if detail.comments.is_empty() {
        lines.push("— Comments —".to_string());
        lines.push("(none)".to_string());
    } else {
        lines.push(format!("— Comments ({}) —", detail.comments.len()));
        for comment in &detail.comments {
            lines.push(String::new());
            lines.push(format!("@{} · {}", comment.author, comment.created_at));
            extend_body_lines(&mut lines, &comment.body);
        }
    }

    lines
}

fn extend_body_lines(lines: &mut Vec<String>, body: &str) {
    if body.trim().is_empty() {
        lines.push("(empty)".to_string());
        return;
    }

    for line in body.lines() {
        lines.push(line.to_string());
    }
}

fn format_comment_date(raw: &str) -> String {
    raw.split('T').next().unwrap_or(raw).to_string()
}

fn format_issue_detail_failure(stderr: &[u8], stdout: &[u8]) -> String {
    let stderr = String::from_utf8_lossy(stderr).trim().to_string();
    let stdout = String::from_utf8_lossy(stdout).trim().to_string();

    if !stderr.is_empty() {
        return stderr;
    }
    if !stdout.is_empty() {
        return stdout;
    }

    "gh issue view failed".to_string()
}

pub fn scroll_issue_detail(
    scroll_offset: &mut usize,
    delta: i32,
    line_count: usize,
    viewport_rows: usize,
) {
    if viewport_rows == 0 {
        return;
    }

    let max_offset = line_count.saturating_sub(viewport_rows);
    let current = *scroll_offset as i32;
    let next = (current + delta).clamp(0, max_offset as i32);
    *scroll_offset = usize::try_from(next).unwrap_or(0);
}

pub fn page_scroll_issue_detail(
    scroll_offset: &mut usize,
    delta: i32,
    line_count: usize,
    viewport_rows: usize,
) {
    if viewport_rows == 0 {
        return;
    }

    let page = i32::try_from(viewport_rows.saturating_sub(1).max(1)).unwrap_or(1);
    scroll_issue_detail(scroll_offset, delta * page, line_count, viewport_rows);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::github::IssueState;

    #[test]
    fn parse_issue_detail_json_maps_body_and_comments() {
        let json = r#"{
            "number": 56,
            "title": "Issue detail view",
            "body": "First line\nSecond line",
            "state": "OPEN",
            "labels": [{"name": "bug"}],
            "assignees": [{"login": "octocat"}],
            "author": {"login": "pacificnm"},
            "comments": [
                {
                    "author": {"login": "reviewer"},
                    "body": "Looks good",
                    "createdAt": "2026-06-24T12:00:00Z"
                }
            ]
        }"#;

        let detail = parse_issue_detail_json(json.as_bytes()).expect("parse");
        assert_eq!(detail.number, 56);
        assert_eq!(detail.title, "Issue detail view");
        assert_eq!(detail.state, IssueState::Open);
        assert_eq!(detail.author, "pacificnm");
        assert!(detail
            .display_lines
            .iter()
            .any(|line| line.contains("First line")));
        assert!(detail
            .display_lines
            .iter()
            .any(|line| line.contains("@reviewer")));
        assert!(detail
            .display_lines
            .iter()
            .any(|line| line.contains("Looks good")));
        assert!(detail
            .display_lines
            .iter()
            .any(|line| line.contains("Comments (1)")));
    }

    #[test]
    fn parse_issue_detail_json_rejects_invalid_payload() {
        let err = parse_issue_detail_json(b"{not json}").expect_err("invalid");
        assert!(err.contains("Invalid gh issue JSON"));
    }

    #[test]
    fn scroll_issue_detail_clamps_to_content() {
        let mut offset = 0;
        scroll_issue_detail(&mut offset, 100, 10, 4);
        assert_eq!(offset, 6);
        scroll_issue_detail(&mut offset, -100, 10, 4);
        assert_eq!(offset, 0);
    }
}
