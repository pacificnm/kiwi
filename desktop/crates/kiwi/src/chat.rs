//! Shared AI chat completion helpers for the Kiwi CLI.

use nest_ai::{AiService, ChatMessage, CompletionRequest};
use nest_error::NestError;

/// Maps [`nest_ai::AiError`] into a [`NestError`].
pub fn ai_to_nest(error: nest_ai::AiError) -> NestError {
    NestError::network(error.message())
        .with_code(error.nest_code())
        .with_module("nest-ai")
        .with_source(error)
}

/// Sends a single user message and returns assistant text.
pub async fn complete_user_message(
    ai: &AiService,
    message: &str,
    model: Option<&str>,
) -> Result<String, NestError> {
    complete_messages(ai, &[ChatMessage::user(message)], model).await
}

/// Sends a multi-turn chat history and returns assistant text.
pub async fn complete_messages(
    ai: &AiService,
    messages: &[ChatMessage],
    model: Option<&str>,
) -> Result<String, NestError> {
    let request = completion_request(messages, model);
    let response = ai.complete(request).await.map_err(ai_to_nest)?;
    Ok(response.content)
}

fn completion_request(messages: &[ChatMessage], model: Option<&str>) -> CompletionRequest {
    CompletionRequest {
        messages: messages.to_vec(),
        model: model.map(str::to_string),
        format: None,
        tools: Vec::new(),
    }
}
