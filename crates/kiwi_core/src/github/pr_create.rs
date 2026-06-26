use crate::state::{GitHubPrCreatePrompt, GitHubPrCreateStep};

use super::types::PrCreateRequest;

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
