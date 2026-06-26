use std::path::Path;
use std::process::Command;

pub const INSTALL_URL: &str = "https://cli.github.com/";
pub const AUTH_LOGIN_URL: &str = "https://cli.github.com/manual/gh_auth_login";

pub use kiwi_core::github::{GitHubAuthCheckResult, GitHubAuthErrorKind};

pub fn check_github_auth(command: &str) -> GitHubAuthCheckResult {
    if !command_on_path(command) {
        return GitHubAuthCheckResult {
            auth_ok: false,
            error_kind: Some(GitHubAuthErrorKind::NotInstalled),
            message: format!(
                "GitHub CLI ({command}) not found.\n\nInstall gh:\n  {INSTALL_URL}\n\nOr set [github] command in config.toml."
            ),
        };
    }

    let output = Command::new(command).args(["auth", "status"]).output();

    match output {
        Ok(result) if result.status.success() => GitHubAuthCheckResult {
            auth_ok: true,
            error_kind: None,
            message: String::new(),
        },
        Ok(result) => {
            let stderr = String::from_utf8_lossy(&result.stderr);
            let stdout = String::from_utf8_lossy(&result.stdout);
            let detail = format!("{stdout}{stderr}").trim().to_string();
            GitHubAuthCheckResult {
                auth_ok: false,
                error_kind: Some(GitHubAuthErrorKind::NotAuthenticated),
                message: format!(
                    "Not logged in to GitHub.\n\nRun in the shell pane:\n  {command} auth login\n\nDocs:\n  {AUTH_LOGIN_URL}{}",
                    if detail.is_empty() {
                        String::new()
                    } else {
                        format!("\n\n{detail}")
                    }
                ),
            }
        }
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => GitHubAuthCheckResult {
            auth_ok: false,
            error_kind: Some(GitHubAuthErrorKind::NotInstalled),
            message: format!(
                "GitHub CLI ({command}) not found.\n\nInstall gh:\n  {INSTALL_URL}\n\nOr set [github] command in config.toml."
            ),
        },
        Err(err) => GitHubAuthCheckResult {
            auth_ok: false,
            error_kind: Some(GitHubAuthErrorKind::CommandFailed),
            message: format!("Failed to run `{command} auth status`: {err}"),
        },
    }
}

fn command_on_path(command: &str) -> bool {
    let path = Path::new(command);
    if path.components().count() > 1 {
        return path.is_file();
    }

    let Some(paths) = std::env::var_os("PATH") else {
        return false;
    };

    std::env::split_paths(&paths).any(|dir| dir.join(command).is_file())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_command_reports_not_installed() {
        let result = check_github_auth("kiwi-missing-gh-command");
        assert!(!result.auth_ok);
        assert_eq!(result.error_kind, Some(GitHubAuthErrorKind::NotInstalled));
        assert!(result.message.contains(INSTALL_URL));
    }

    #[test]
    fn failing_auth_status_reports_not_authenticated() {
        if !command_on_path("false") {
            return;
        }

        let result = check_github_auth("false");
        assert!(!result.auth_ok);
        assert_eq!(
            result.error_kind,
            Some(GitHubAuthErrorKind::NotAuthenticated)
        );
        assert!(result.message.contains("auth login"));
    }
}
