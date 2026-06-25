use std::path::Path;
use std::process::Command;

use super::issue::command_on_path;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IssueActionResult {
    pub success: bool,
    pub error: Option<String>,
    pub detail: Option<String>,
}

pub fn post_issue_comment(
    repo_root: &Path,
    command: &str,
    number: u32,
    body: &str,
) -> IssueActionResult {
    let trimmed = body.trim();
    if trimmed.is_empty() {
        return IssueActionResult {
            success: false,
            error: Some("Comment body cannot be empty".to_string()),
            detail: None,
        };
    }

    if !command_on_path(command) {
        return IssueActionResult {
            success: false,
            error: Some(format!("GitHub CLI ({command}) not found on PATH")),
            detail: None,
        };
    }

    let output = Command::new(command)
        .args(["issue", "comment", &number.to_string(), "--body", trimmed])
        .current_dir(repo_root)
        .output();

    match output {
        Ok(result) if result.status.success() => IssueActionResult {
            success: true,
            error: None,
            detail: None,
        },
        Ok(result) => IssueActionResult {
            success: false,
            error: Some(format_action_failure(
                "gh issue comment",
                &result.stderr,
                &result.stdout,
            )),
            detail: None,
        },
        Err(err) => IssueActionResult {
            success: false,
            error: Some(format!("Failed to run `{command} issue comment`: {err}")),
            detail: None,
        },
    }
}

pub fn create_branch_from_issue(
    repo_root: &Path,
    command: &str,
    number: u32,
) -> IssueActionResult {
    if !command_on_path(command) {
        return IssueActionResult {
            success: false,
            error: Some(format!("GitHub CLI ({command}) not found on PATH")),
            detail: None,
        };
    }

    let output = Command::new(command)
        .args(["issue", "develop", &number.to_string(), "--checkout"])
        .current_dir(repo_root)
        .output();

    match output {
        Ok(result) if result.status.success() => {
            let stdout = String::from_utf8_lossy(&result.stdout).trim().to_string();
            let detail = if stdout.is_empty() {
                Some(format!("Checked out branch for issue #{number}"))
            } else {
                Some(stdout)
            };
            IssueActionResult {
                success: true,
                error: None,
                detail,
            }
        }
        Ok(result) => IssueActionResult {
            success: false,
            error: Some(format_action_failure(
                "gh issue develop",
                &result.stderr,
                &result.stdout,
            )),
            detail: None,
        },
        Err(err) => IssueActionResult {
            success: false,
            error: Some(format!("Failed to run `{command} issue develop`: {err}")),
            detail: None,
        },
    }
}

pub fn add_issue_labels(
    repo_root: &Path,
    command: &str,
    number: u32,
    labels: &[String],
) -> IssueActionResult {
    if labels.is_empty() {
        return IssueActionResult {
            success: false,
            error: Some("Select at least one label".to_string()),
            detail: None,
        };
    }

    if !command_on_path(command) {
        return IssueActionResult {
            success: false,
            error: Some(format!("GitHub CLI ({command}) not found on PATH")),
            detail: None,
        };
    }

    let label_arg = labels.join(",");
    let output = Command::new(command)
        .args([
            "issue",
            "edit",
            &number.to_string(),
            "--add-label",
            &label_arg,
        ])
        .current_dir(repo_root)
        .output();

    match output {
        Ok(result) if result.status.success() => IssueActionResult {
            success: true,
            error: None,
            detail: None,
        },
        Ok(result) => IssueActionResult {
            success: false,
            error: Some(format_action_failure(
                "gh issue edit",
                &result.stderr,
                &result.stdout,
            )),
            detail: None,
        },
        Err(err) => IssueActionResult {
            success: false,
            error: Some(format!("Failed to run `{command} issue edit`: {err}")),
            detail: None,
        },
    }
}

fn format_action_failure(command: &str, stderr: &[u8], stdout: &[u8]) -> String {
    let stderr = String::from_utf8_lossy(stderr).trim().to_string();
    let stdout = String::from_utf8_lossy(stdout).trim().to_string();

    if !stderr.is_empty() {
        return stderr;
    }
    if !stdout.is_empty() {
        return stdout;
    }

    format!("{command} failed")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn post_issue_comment_rejects_empty_body_without_subprocess() {
        let result = post_issue_comment(Path::new("."), "gh-nonexistent-kiwi-test", 1, "   ");
        assert!(!result.success);
        assert_eq!(
            result.error.as_deref(),
            Some("Comment body cannot be empty")
        );
    }

    #[test]
    fn add_issue_labels_rejects_empty_selection() {
        let result = add_issue_labels(Path::new("."), "gh", 1, &[]);
        assert!(!result.success);
        assert_eq!(result.error.as_deref(), Some("Select at least one label"));
    }

    #[test]
    fn create_branch_from_issue_requires_gh_on_path() {
        let result = create_branch_from_issue(Path::new("."), "gh-nonexistent-kiwi-test", 1);
        assert!(!result.success);
        assert!(result
            .error
            .as_ref()
            .is_some_and(|message| message.contains("not found on PATH")));
    }
}
