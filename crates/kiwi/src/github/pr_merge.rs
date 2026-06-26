use std::path::Path;
use std::process::Command;

use super::actions::{format_action_failure, IssueActionResult};
use super::issue::command_on_path;

pub fn merge_pull_request(repo_root: &Path, command: &str, number: u32) -> IssueActionResult {
    if !command_on_path(command) {
        return IssueActionResult {
            success: false,
            error: Some(format!("GitHub CLI ({command}) not found on PATH")),
            detail: None,
        };
    }

    let output = Command::new(command)
        .args(["pr", "merge", &number.to_string(), "--merge"])
        .current_dir(repo_root)
        .output();

    match output {
        Ok(result) if result.status.success() => {
            let stdout = String::from_utf8_lossy(&result.stdout).trim().to_string();
            IssueActionResult {
                success: true,
                error: None,
                detail: if stdout.is_empty() {
                    Some(format!("Merged pull request #{number}"))
                } else {
                    Some(stdout)
                },
            }
        }
        Ok(result) => IssueActionResult {
            success: false,
            error: Some(format_action_failure(
                "gh pr merge",
                &result.stderr,
                &result.stdout,
            )),
            detail: None,
        },
        Err(err) => IssueActionResult {
            success: false,
            error: Some(format!("Failed to run `{command} pr merge`: {err}")),
            detail: None,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn merge_pull_request_requires_gh_on_path() {
        let result = merge_pull_request(Path::new("."), "gh-nonexistent-kiwi-test", 42);
        assert!(!result.success);
        assert!(result
            .error
            .as_ref()
            .is_some_and(|message| message.contains("not found on PATH")));
    }
}
