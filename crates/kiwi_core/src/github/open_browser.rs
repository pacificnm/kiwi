use std::path::Path;
use std::process::Command;

use super::command::command_on_path;
use super::types::{GitHubBrowserTarget, IssueActionResult};

pub fn open_in_browser(
    repo_root: &Path,
    command: &str,
    target: GitHubBrowserTarget,
) -> IssueActionResult {
    if !command_on_path(command) {
        return IssueActionResult {
            success: false,
            error: Some(format!("GitHub CLI ({command}) not found on PATH")),
            detail: None,
        };
    }

    let (kind, number) = match target {
        GitHubBrowserTarget::Issue(number) => ("issue", number),
        GitHubBrowserTarget::PullRequest(number) => ("pr", number),
    };

    let output = Command::new(command)
        .args([kind, "view", &number.to_string(), "--web"])
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
            error: Some(format_browser_failure(kind, &result.stderr, &result.stdout)),
            detail: None,
        },
        Err(err) => IssueActionResult {
            success: false,
            error: Some(format!(
                "Failed to run `{command} {kind} view --web`: {err}"
            )),
            detail: None,
        },
    }
}

fn format_browser_failure(kind: &str, stderr: &[u8], stdout: &[u8]) -> String {
    let stderr = String::from_utf8_lossy(stderr).trim().to_string();
    let stdout = String::from_utf8_lossy(stdout).trim().to_string();

    if !stderr.is_empty() {
        return stderr;
    }
    if !stdout.is_empty() {
        return stdout;
    }

    format!("gh {kind} view --web failed")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn open_in_browser_requires_gh_on_path() {
        let result = open_in_browser(
            Path::new("."),
            "gh-nonexistent-kiwi-test",
            GitHubBrowserTarget::Issue(1),
        );
        assert!(!result.success);
        assert!(result
            .error
            .as_ref()
            .is_some_and(|message| message.contains("not found on PATH")));
    }
}
