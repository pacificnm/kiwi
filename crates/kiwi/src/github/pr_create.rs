use std::path::Path;
use std::process::Command;

use crate::state::{GitHubPrCreatePrompt, GitHubPrCreateStep};

use super::issue::command_on_path;
use super::IssueActionResult;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PrCreateRequest {
    pub title: String,
    pub body: String,
    pub base: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PrCreatePromptAdvance {
    Continue(GitHubPrCreatePrompt),
    Submit(PrCreateRequest),
}

pub fn advance_pr_create_prompt(
    mut prompt: GitHubPrCreatePrompt,
    input: &str,
) -> Result<PrCreatePromptAdvance, &'static str> {
    match prompt.step {
        GitHubPrCreateStep::Title => {
            let title = input.trim();
            if title.is_empty() {
                return Err("PR title cannot be empty");
            }
            prompt.title = title.to_string();
            prompt.step = GitHubPrCreateStep::Body;
            Ok(PrCreatePromptAdvance::Continue(prompt))
        }
        GitHubPrCreateStep::Body => {
            prompt.body = input.to_string();
            prompt.step = GitHubPrCreateStep::Base;
            Ok(PrCreatePromptAdvance::Continue(prompt))
        }
        GitHubPrCreateStep::Base => {
            let base = input.trim();
            Ok(PrCreatePromptAdvance::Submit(PrCreateRequest {
                title: prompt.title,
                body: prompt.body,
                base: if base.is_empty() {
                    None
                } else {
                    Some(base.to_string())
                },
            }))
        }
    }
}

pub fn create_pull_request(
    repo_root: &Path,
    command: &str,
    request: &PrCreateRequest,
) -> IssueActionResult {
    if request.title.trim().is_empty() {
        return IssueActionResult {
            success: false,
            error: Some("PR title cannot be empty".to_string()),
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

    let mut args = vec![
        "pr".to_string(),
        "create".to_string(),
        "--title".to_string(),
        request.title.clone(),
        "--body".to_string(),
        request.body.clone(),
    ];
    if let Some(base) = &request.base {
        args.push("--base".to_string());
        args.push(base.clone());
    }

    let output = Command::new(command)
        .args(&args)
        .current_dir(repo_root)
        .output();

    match output {
        Ok(result) if result.status.success() => {
            let stdout = String::from_utf8_lossy(&result.stdout).trim().to_string();
            IssueActionResult {
                success: true,
                error: None,
                detail: if stdout.is_empty() {
                    Some("Pull request created".to_string())
                } else {
                    Some(stdout)
                },
            }
        }
        Ok(result) => IssueActionResult {
            success: false,
            error: Some(format_action_failure(
                "gh pr create",
                &result.stderr,
                &result.stdout,
            )),
            detail: None,
        },
        Err(err) => IssueActionResult {
            success: false,
            error: Some(format!("Failed to run `{command} pr create`: {err}")),
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
    fn advance_prompt_steps_through_title_body_base() {
        let prompt = GitHubPrCreatePrompt::default();
        let PrCreatePromptAdvance::Continue(body_step) =
            advance_pr_create_prompt(prompt, "Fix login").expect("title")
        else {
            panic!("expected continue after title");
        };
        assert_eq!(body_step.step, GitHubPrCreateStep::Body);
        assert_eq!(body_step.title, "Fix login");

        let PrCreatePromptAdvance::Continue(base_step) =
            advance_pr_create_prompt(body_step, "Fixes #42").expect("body")
        else {
            panic!("expected continue after body");
        };
        assert_eq!(base_step.step, GitHubPrCreateStep::Base);

        let PrCreatePromptAdvance::Submit(request) =
            advance_pr_create_prompt(base_step, "main").expect("base")
        else {
            panic!("expected submit after base");
        };
        assert_eq!(request.title, "Fix login");
        assert_eq!(request.body, "Fixes #42");
        assert_eq!(request.base.as_deref(), Some("main"));
    }

    #[test]
    fn advance_prompt_allows_empty_base() {
        let prompt = GitHubPrCreatePrompt {
            step: GitHubPrCreateStep::Base,
            title: "Title".to_string(),
            body: "Body".to_string(),
        };
        let PrCreatePromptAdvance::Submit(request) =
            advance_pr_create_prompt(prompt, "   ").expect("submit")
        else {
            panic!("expected submit");
        };
        assert!(request.base.is_none());
    }

    #[test]
    fn create_pull_request_rejects_empty_title_without_subprocess() {
        let result = create_pull_request(
            Path::new("."),
            "gh-nonexistent-kiwi-test",
            &PrCreateRequest {
                title: "   ".to_string(),
                body: String::new(),
                base: None,
            },
        );
        assert!(!result.success);
    }
}
