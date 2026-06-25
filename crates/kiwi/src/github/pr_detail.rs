use std::path::Path;
use std::process::Command;

use serde::Deserialize;

use super::issue::command_on_path;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PrState {
    Open,
    Draft,
    Merged,
    Closed,
}

impl PrState {
    #[must_use]
    pub fn parse(raw: &str, is_draft: bool) -> Self {
        if is_draft {
            return Self::Draft;
        }

        match raw {
            "MERGED" => Self::Merged,
            "CLOSED" => Self::Closed,
            _ => Self::Open,
        }
    }

    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Open => "open",
            Self::Draft => "draft",
            Self::Merged => "merged",
            Self::Closed => "closed",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PrDetail {
    pub number: u32,
    pub title: String,
    pub state: PrState,
    pub author: String,
    pub display_lines: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PrDetailLoadResult {
    pub detail: Option<PrDetail>,
    pub error: Option<String>,
}

#[derive(Debug, Deserialize)]
struct GhPrDetail {
    number: u32,
    title: String,
    body: String,
    state: String,
    #[serde(rename = "isDraft")]
    is_draft: bool,
    author: GhUser,
    #[serde(rename = "baseRefName")]
    base_ref_name: String,
    #[serde(rename = "headRefName")]
    head_ref_name: String,
    additions: u32,
    deletions: u32,
    #[serde(rename = "changedFiles")]
    changed_files: u32,
    commits: Vec<GhCommit>,
    #[serde(default, rename = "statusCheckRollup")]
    status_checks: Vec<GhStatusCheck>,
}

#[derive(Debug, Deserialize)]
struct GhCommit {
    #[serde(rename = "messageHeadline")]
    message_headline: String,
    #[serde(rename = "committedDate")]
    committed_date: String,
    authors: Vec<GhUser>,
}

#[derive(Debug, Deserialize)]
struct GhStatusCheck {
    name: Option<String>,
    context: Option<String>,
    status: Option<String>,
    conclusion: Option<String>,
    state: Option<String>,
}

#[derive(Debug, Deserialize)]
struct GhUser {
    login: String,
}

pub fn load_pr_detail(repo_root: &Path, command: &str, number: u32) -> PrDetailLoadResult {
    if !command_on_path(command) {
        return PrDetailLoadResult {
            detail: None,
            error: Some(format!("GitHub CLI ({command}) not found on PATH")),
        };
    }

    let output = Command::new(command)
        .args([
            "pr",
            "view",
            &number.to_string(),
            "--json",
            "number,title,body,state,author,isDraft,baseRefName,headRefName,additions,deletions,changedFiles,commits,statusCheckRollup",
        ])
        .current_dir(repo_root)
        .output();

    match output {
        Ok(result) if result.status.success() => match parse_pr_detail_json(&result.stdout) {
            Ok(detail) => PrDetailLoadResult {
                detail: Some(detail),
                error: None,
            },
            Err(message) => PrDetailLoadResult {
                detail: None,
                error: Some(message),
            },
        },
        Ok(result) => PrDetailLoadResult {
            detail: None,
            error: Some(format_pr_detail_failure(&result.stderr, &result.stdout)),
        },
        Err(err) => PrDetailLoadResult {
            detail: None,
            error: Some(format!("Failed to run `{command} pr view`: {err}")),
        },
    }
}

fn parse_pr_detail_json(bytes: &[u8]) -> Result<PrDetail, String> {
    let raw: GhPrDetail =
        serde_json::from_slice(bytes).map_err(|err| format!("Invalid gh pr JSON: {err}"))?;

    let state = PrState::parse(&raw.state, raw.is_draft);
    let draft = PrDetailDraft {
        number: raw.number,
        title: raw.title,
        state,
        author: raw.author.login,
        base_ref_name: raw.base_ref_name,
        head_ref_name: raw.head_ref_name,
        additions: raw.additions,
        deletions: raw.deletions,
        changed_files: raw.changed_files,
        body: raw.body,
        commits: raw.commits,
        status_checks: raw.status_checks,
    };
    let display_lines = build_display_lines(&draft);

    Ok(PrDetail {
        number: draft.number,
        title: draft.title,
        state: draft.state,
        author: draft.author,
        display_lines,
    })
}

struct PrDetailDraft {
    number: u32,
    title: String,
    state: PrState,
    author: String,
    base_ref_name: String,
    head_ref_name: String,
    additions: u32,
    deletions: u32,
    changed_files: u32,
    body: String,
    commits: Vec<GhCommit>,
    status_checks: Vec<GhStatusCheck>,
}

fn build_display_lines(detail: &PrDetailDraft) -> Vec<String> {
    let mut lines = vec![
        format!("#{} {}", detail.number, detail.title),
        format!(
            "State: {} · Author: {}",
            detail.state.label(),
            detail.author
        ),
        format!(
            "Branch: {} → {} · +{} -{} · {} files",
            detail.head_ref_name,
            detail.base_ref_name,
            detail.additions,
            detail.deletions,
            detail.changed_files
        ),
    ];

    lines.push(String::new());
    lines.push("— Description —".to_string());
    lines.push(String::new());
    extend_body_lines(&mut lines, &detail.body);

    lines.push(String::new());
    if detail.commits.is_empty() {
        lines.push("— Commits —".to_string());
        lines.push("(none)".to_string());
    } else {
        lines.push(format!("— Commits ({}) —", detail.commits.len()));
        for commit in &detail.commits {
            let author = commit
                .authors
                .first()
                .map(|user| user.login.as_str())
                .unwrap_or("unknown");
            let date = format_commit_date(&commit.committed_date);
            lines.push(format!(
                "· {} · @{} · {}",
                commit.message_headline, author, date
            ));
        }
    }

    lines.push(String::new());
    if detail.status_checks.is_empty() {
        lines.push("— Checks —".to_string());
        lines.push("(none reported)".to_string());
    } else {
        lines.push(format!("— Checks ({}) —", detail.status_checks.len()));
        for check in &detail.status_checks {
            lines.push(format_check_line(check));
        }
    }

    lines
}

fn format_check_line(check: &GhStatusCheck) -> String {
    let name = check
        .name
        .as_deref()
        .or(check.context.as_deref())
        .unwrap_or("check");
    let status = check
        .conclusion
        .as_deref()
        .or(check.state.as_deref())
        .or(check.status.as_deref())
        .unwrap_or("unknown");
    format!("· {name}: {status}")
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

fn format_commit_date(raw: &str) -> String {
    raw.split('T').next().unwrap_or(raw).to_string()
}

fn format_pr_detail_failure(stderr: &[u8], stdout: &[u8]) -> String {
    let stderr = String::from_utf8_lossy(stderr).trim().to_string();
    let stdout = String::from_utf8_lossy(stdout).trim().to_string();

    if !stderr.is_empty() {
        return stderr;
    }
    if !stdout.is_empty() {
        return stdout;
    }

    "gh pr view failed".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_pr_detail_json_maps_body_commits_and_checks() {
        let json = r#"{
            "number": 60,
            "title": "PR detail view",
            "body": "Adds PR detail pane",
            "state": "OPEN",
            "isDraft": false,
            "author": {"login": "pacificnm"},
            "baseRefName": "main",
            "headRefName": "60-pr-detail-view",
            "additions": 120,
            "deletions": 4,
            "changedFiles": 8,
            "commits": [
                {
                    "messageHeadline": "Add PR detail loader",
                    "committedDate": "2026-06-25T10:00:00Z",
                    "authors": [{"login": "pacificnm"}]
                }
            ],
            "statusCheckRollup": [
                {
                    "name": "ci",
                    "status": "COMPLETED",
                    "conclusion": "SUCCESS"
                }
            ]
        }"#;

        let detail = parse_pr_detail_json(json.as_bytes()).expect("parse");
        assert_eq!(detail.number, 60);
        assert_eq!(detail.title, "PR detail view");
        assert_eq!(detail.state, PrState::Open);
        assert_eq!(detail.author, "pacificnm");
        assert!(detail
            .display_lines
            .iter()
            .any(|line| line.contains("Adds PR detail pane")));
        assert!(detail
            .display_lines
            .iter()
            .any(|line| line.contains("Add PR detail loader")));
        assert!(detail
            .display_lines
            .iter()
            .any(|line| line.contains("ci: SUCCESS")));
    }

    #[test]
    fn parse_pr_detail_json_marks_draft_state() {
        let json = r#"{
            "number": 1,
            "title": "WIP",
            "body": "",
            "state": "OPEN",
            "isDraft": true,
            "author": {"login": "dev"},
            "baseRefName": "main",
            "headRefName": "wip",
            "additions": 0,
            "deletions": 0,
            "changedFiles": 0,
            "commits": [],
            "statusCheckRollup": []
        }"#;

        let detail = parse_pr_detail_json(json.as_bytes()).expect("parse");
        assert_eq!(detail.state, PrState::Draft);
        assert!(detail
            .display_lines
            .iter()
            .any(|line| line.contains("State: draft")));
    }

    #[test]
    fn parse_pr_detail_json_rejects_invalid_payload() {
        let err = parse_pr_detail_json(b"{not json}").expect_err("invalid");
        assert!(err.contains("Invalid gh pr JSON"));
    }
}
