use std::path::Path;
use std::process::Command;

use serde::Deserialize;

use super::command::command_on_path;
use super::types::{Issue, IssueListLoadResult, IssueState};

pub const ISSUE_LIST_LIMIT: &str = "100";

#[derive(Debug, Deserialize)]
struct GhIssue {
    number: u32,
    title: String,
    state: String,
    labels: Vec<GhLabel>,
    assignees: Vec<GhUser>,
}

#[derive(Debug, Deserialize)]
struct GhLabel {
    name: String,
}

#[derive(Debug, Deserialize)]
struct GhUser {
    login: String,
}

pub fn load_issue_list(repo_root: &Path, command: &str) -> IssueListLoadResult {
    if !command_on_path(command) {
        return IssueListLoadResult {
            issues: Vec::new(),
            error: Some(format!("GitHub CLI ({command}) not found on PATH")),
        };
    }

    let output = Command::new(command)
        .args([
            "issue",
            "list",
            "--json",
            "number,title,state,labels,assignees",
            "--limit",
            ISSUE_LIST_LIMIT,
        ])
        .current_dir(repo_root)
        .output();

    match output {
        Ok(result) if result.status.success() => match parse_issue_list_json(&result.stdout) {
            Ok(issues) => IssueListLoadResult {
                issues,
                error: None,
            },
            Err(message) => IssueListLoadResult {
                issues: Vec::new(),
                error: Some(message),
            },
        },
        Ok(result) => IssueListLoadResult {
            issues: Vec::new(),
            error: Some(format_issue_list_failure(&result.stderr, &result.stdout)),
        },
        Err(err) => IssueListLoadResult {
            issues: Vec::new(),
            error: Some(format!("Failed to run `{command} issue list`: {err}")),
        },
    }
}

fn parse_issue_list_json(bytes: &[u8]) -> Result<Vec<Issue>, String> {
    let raw: Vec<GhIssue> =
        serde_json::from_slice(bytes).map_err(|err| format!("Invalid gh issue JSON: {err}"))?;

    Ok(raw
        .into_iter()
        .map(|issue| Issue {
            number: issue.number,
            title: issue.title,
            state: IssueState::parse(&issue.state),
            labels: issue.labels.into_iter().map(|label| label.name).collect(),
            assignees: issue.assignees.into_iter().map(|user| user.login).collect(),
        })
        .collect())
}

fn format_issue_list_failure(stderr: &[u8], stdout: &[u8]) -> String {
    let stderr = String::from_utf8_lossy(stderr).trim().to_string();
    let stdout = String::from_utf8_lossy(stdout).trim().to_string();

    if !stderr.is_empty() {
        return stderr;
    }
    if !stdout.is_empty() {
        return stdout;
    }

    "gh issue list failed".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_issue_list_json_maps_fields() {
        let json = r#"[
            {
                "number": 42,
                "title": "Fix bug",
                "state": "OPEN",
                "labels": [{"name": "bug"}],
                "assignees": [{"login": "octocat"}]
            },
            {
                "number": 7,
                "title": "Old task",
                "state": "CLOSED",
                "labels": [],
                "assignees": []
            }
        ]"#;

        let issues = parse_issue_list_json(json.as_bytes()).expect("parse");
        assert_eq!(issues.len(), 2);
        assert_eq!(issues[0].number, 42);
        assert_eq!(issues[0].title, "Fix bug");
        assert_eq!(issues[0].state, IssueState::Open);
        assert_eq!(issues[0].labels, vec!["bug".to_string()]);
        assert_eq!(issues[0].assignees, vec!["octocat".to_string()]);
        assert_eq!(issues[1].state, IssueState::Closed);
    }

    #[test]
    fn parse_issue_list_json_rejects_invalid_payload() {
        let err = parse_issue_list_json(b"{not json}").expect_err("invalid");
        assert!(err.contains("Invalid gh issue JSON"));
    }
}
