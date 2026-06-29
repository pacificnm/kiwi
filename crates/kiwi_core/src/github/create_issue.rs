use std::path::Path;
use std::process::Command;

use super::actions::format_action_failure;
use super::command::command_on_path;
use super::types::{IssueActionResult, IssueCreateRequest, IssueCreateResult};

pub fn create_issue(
    repo_root: &Path,
    command: &str,
    request: &IssueCreateRequest,
) -> IssueCreateResult {
    let title = request.title.trim();
    if title.is_empty() {
        return IssueCreateResult {
            result: IssueActionResult {
                success: false,
                error: Some("Issue title cannot be empty".to_string()),
                detail: None,
            },
            number: None,
        };
    }

    if !command_on_path(command) {
        return IssueCreateResult {
            result: IssueActionResult {
                success: false,
                error: Some(format!("GitHub CLI ({command}) not found on PATH")),
                detail: None,
            },
            number: None,
        };
    }

    let body = request.body.trim();
    let output = Command::new(command)
        .args(["issue", "create", "--title", title, "--body", body])
        .current_dir(repo_root)
        .output();

    match output {
        Ok(result) if result.status.success() => {
            let stdout = String::from_utf8_lossy(&result.stdout).trim().to_string();
            let number = parse_issue_number_from_create_output(&stdout);
            let detail = if stdout.is_empty() {
                number.map(|n| format!("Created issue #{n}"))
            } else {
                Some(stdout)
            };
            IssueCreateResult {
                result: IssueActionResult {
                    success: true,
                    error: None,
                    detail,
                },
                number,
            }
        }
        Ok(result) => IssueCreateResult {
            result: IssueActionResult {
                success: false,
                error: Some(format_action_failure(
                    "gh issue create",
                    &result.stderr,
                    &result.stdout,
                )),
                detail: None,
            },
            number: None,
        },
        Err(err) => {
            let message = format!("Failed to run `{command} issue create`: {err}");
            IssueCreateResult {
                result: IssueActionResult {
                    success: false,
                    error: Some(message),
                    detail: None,
                },
                number: None,
            }
        }
    }
}

fn parse_issue_number_from_create_output(output: &str) -> Option<u32> {
    let trimmed = output.trim();
    if let Some(suffix) = trimmed.rsplit("/issues/").next() {
        if let Ok(number) = suffix.parse::<u32>() {
            return Some(number);
        }
    }
    trimmed.parse().ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_issue_number_from_url() {
        assert_eq!(
            parse_issue_number_from_create_output("https://github.com/owner/repo/issues/42"),
            Some(42)
        );
    }

    #[test]
    fn parse_issue_number_from_plain_number() {
        assert_eq!(parse_issue_number_from_create_output("99"), Some(99));
    }

    #[test]
    fn create_issue_rejects_empty_title_without_subprocess() {
        let result = create_issue(
            Path::new("."),
            "gh-nonexistent-kiwi-test",
            &IssueCreateRequest {
                title: "   ".to_string(),
                body: String::new(),
            },
        );
        assert!(!result.result.success);
        assert_eq!(
            result.result.error.as_deref(),
            Some("Issue title cannot be empty")
        );
    }
}
