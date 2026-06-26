use std::path::Path;
use std::process::Command;

use super::actions::format_action_failure;
use super::command::command_on_path;
use super::types::{IssueActionResult, PrCreateRequest};

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::github::pr_create::{advance_pr_create_prompt, PrCreatePromptAdvance};
    use crate::state::{GitHubPrCreatePrompt, GitHubPrCreateStep};

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
