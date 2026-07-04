//! Shared AI chat completion helpers for CLI and GUI.

use std::path::PathBuf;
use std::sync::mpsc;
use std::thread;

use futures_util::StreamExt;
use nest_agent::{AgentEvent, AgentLoop, CancelToken};
use nest_ai::{AiService, ChatMessage, CompletionRequest};
use nest_error::NestError;
use nest_mcp::McpHub;
use tokio::sync::mpsc as tokio_mpsc;

/// Maps [`nest_ai::AiError`] into a [`NestError`].
pub fn ai_to_nest(error: nest_ai::AiError) -> NestError {
    NestError::network(error.message())
        .with_code(error.nest_code())
        .with_module("nest-ai")
        .with_source(error)
}

/// Incremental chat events delivered to the GUI thread.
#[derive(Debug)]
pub enum ChatStreamEvent {
    /// Assistant text fragment to append.
    Delta(String),
    /// Stream finished successfully or with an error.
    Finished {
        /// Outcome of the request.
        result: Result<(), NestError>,
        /// Provider metrics when available on the final chunk.
        metrics: Option<nest_ai::CompletionMetrics>,
    },
}

/// Agent loop events delivered to the GUI thread.
#[derive(Debug)]
pub enum AgentRunEvent {
    /// Incremental agent loop event.
    Event(AgentEvent),
    /// Agent run finished (success or error).
    Finished(Result<(), NestError>),
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

/// Runs a streaming completion on a background thread (for GUI hosts).
pub fn spawn_stream_complete_messages(
    ai: AiService,
    messages: Vec<ChatMessage>,
    model: Option<String>,
) -> mpsc::Receiver<ChatStreamEvent> {
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        let runtime = tokio::runtime::Runtime::new().expect("tokio runtime");
        runtime.block_on(async {
            let request = completion_request(&messages, model.as_deref());
            let mut stream = match ai.stream_complete(request).await {
                Ok(stream) => stream,
                Err(error) => {
                    let _ = tx.send(ChatStreamEvent::Finished {
                        result: Err(ai_to_nest(error)),
                        metrics: None,
                    });
                    return;
                }
            };

            let mut last_metrics = None;
            while let Some(chunk) = stream.next().await {
                match chunk {
                    Ok(chunk) => {
                        if !chunk.content_delta.is_empty()
                            && tx.send(ChatStreamEvent::Delta(chunk.content_delta)).is_err()
                        {
                            return;
                        }
                        if chunk.metrics.is_some() {
                            last_metrics = chunk.metrics;
                        }
                        if chunk.done {
                            let _ = tx.send(ChatStreamEvent::Finished {
                                result: Ok(()),
                                metrics: last_metrics,
                            });
                            return;
                        }
                    }
                    Err(error) => {
                        let _ = tx.send(ChatStreamEvent::Finished {
                            result: Err(ai_to_nest(error)),
                            metrics: None,
                        });
                        return;
                    }
                }
            }

            let _ = tx.send(ChatStreamEvent::Finished {
                result: Ok(()),
                metrics: last_metrics,
            });
        });
    });
    rx
}

/// Runs an MCP-backed agent loop on a background thread (for GUI hosts).
pub fn spawn_agent_run(
    ai: AiService,
    messages: Vec<ChatMessage>,
    model: Option<String>,
    mcp_config_path: PathBuf,
    mcp_servers: Vec<String>,
    max_steps: u32,
) -> mpsc::Receiver<AgentRunEvent> {
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        let runtime = tokio::runtime::Runtime::new().expect("tokio runtime");
        runtime.block_on(async {
            let result = async {
                let mut hub = McpHub::from_config_file(&mcp_config_path, Some(&mcp_servers))
                    .await
                    .map_err(|error| {
                        NestError::network(format!(
                            "failed to load MCP config {}: {error}",
                            mcp_config_path.display()
                        ))
                        .with_module("nest-mcp")
                    })?;

                let (event_tx, mut event_rx) = tokio_mpsc::channel(32);
                let loop_ = AgentLoop::new(ai, nest_agent::AgentConfig::default().with_max_steps(max_steps));
                let cancel = CancelToken::new();

                let run_handle = tokio::spawn(async move {
                    loop_
                        .run(&mut hub, messages, model, event_tx, cancel)
                        .await
                });

                while let Some(event) = event_rx.recv().await {
                    if tx.send(AgentRunEvent::Event(event.clone())).is_err() {
                        return Ok(());
                    }
                    if matches!(event, AgentEvent::Failed(_)) {
                        break;
                    }
                }

                run_handle.await.map_err(|error| {
                    NestError::network(error.to_string()).with_module("nest-agent")
                })??;

                Ok(())
            }
            .await;

            let _ = tx.send(AgentRunEvent::Finished(result));
        });
    });
    rx
}

fn completion_request(messages: &[ChatMessage], model: Option<&str>) -> CompletionRequest {
    let mut request = CompletionRequest {
        model: None,
        messages: messages.to_vec(),
        format: None,
        tools: Vec::new(),
    };
    if let Some(model) = model.filter(|value| !value.is_empty()) {
        request = request.with_model(model);
    }
    request
}
