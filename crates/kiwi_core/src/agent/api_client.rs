//! LLM API streaming clients for the native chat agent architecture.
//!
//! - Claude: POSTs to the Anthropic Messages API, reads Anthropic SSE format.
//! - Ollama: POSTs to `/api/chat`, reads NDJSON stream (no `data:` prefix).
//! - OpenAI: POSTs to `/v1/chat/completions`, reads OpenAI SSE format (`data: [DONE]` sentinel).
//!
//! All implementations spawn a background thread and fire `AppEvent` variants into the `EventSender`.

use std::collections::BTreeMap;
use std::io::BufRead;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use serde::Serialize;

use crate::agent::tools::{
    ollama_supports_tools, tools_for_claude, tools_for_openai, OpenAiToolSchema, ToolRegistry,
};
use crate::agent::{AgentId, ChatMessage, ContentBlock, MessageRole};
use crate::events::{AppEvent, EventSender};

use super::stream_event::{ApiStreamEvent, ContentBlockStart, ContentDelta};

const ANTHROPIC_API_URL: &str = "https://api.anthropic.com/v1/messages";
const ANTHROPIC_VERSION: &str = "2023-06-01";
const OPENAI_API_URL: &str = "https://api.openai.com/v1/chat/completions";
const CURSOR_API_URL: &str = "https://api.cursor.sh/v1/chat/completions";
const DEFAULT_MAX_TOKENS: u32 = 8192;

// ---------------------------------------------------------------------------
// Cancel handle
// ---------------------------------------------------------------------------

/// A cheap-to-clone handle for cancelling an in-progress stream.
#[derive(Debug, Clone, Default)]
pub struct StreamCancelHandle(Arc<AtomicBool>);

impl StreamCancelHandle {
    pub fn cancel(&self) {
        self.0.store(true, Ordering::Relaxed);
    }

    pub fn is_cancelled(&self) -> bool {
        self.0.load(Ordering::Relaxed)
    }
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Spawn a background thread that streams a Claude API turn and fires events.
///
/// The thread reads `messages`, builds an Anthropic API request, and fires:
/// - `AgentTokenChunk` for each text token
/// - `AgentToolCallStart` when a tool call block completes
/// - `AgentTurnComplete` when the message finishes
/// - `AgentApiError` on any network or API error
pub fn spawn_claude_stream(
    agent_id: AgentId,
    api_key: String,
    model: String,
    messages: Vec<ChatMessage>,
    cancel: StreamCancelHandle,
    sender: EventSender,
) -> std::thread::JoinHandle<()> {
    std::thread::spawn(move || {
        if let Err(msg) = run_stream(agent_id, &api_key, &model, &messages, &cancel, &sender) {
            if !cancel.is_cancelled() {
                let _ = sender.send(AppEvent::AgentApiError {
                    agent_id,
                    message: msg,
                });
            }
        }
    })
}

// ---------------------------------------------------------------------------
// Ollama public API
// ---------------------------------------------------------------------------

/// Spawn a background thread that streams an Ollama chat turn and fires events.
///
/// Uses Ollama's native `/api/chat` endpoint which returns NDJSON (one JSON object
/// per line, no `data:` prefix). Tool calls are sent for known tool-capable models.
pub fn spawn_ollama_stream(
    agent_id: AgentId,
    api_url: String,
    model: String,
    messages: Vec<ChatMessage>,
    cancel: StreamCancelHandle,
    sender: EventSender,
) -> std::thread::JoinHandle<()> {
    std::thread::spawn(move || {
        if let Err(msg) = run_ollama_stream(agent_id, &api_url, &model, &messages, &cancel, &sender) {
            if !cancel.is_cancelled() {
                let _ = sender.send(AppEvent::AgentApiError { agent_id, message: msg });
            }
        }
    })
}

fn run_ollama_stream(
    agent_id: AgentId,
    api_url: &str,
    model: &str,
    messages: &[ChatMessage],
    cancel: &StreamCancelHandle,
    sender: &EventSender,
) -> Result<(), String> {
    let url = format!("{}/api/chat", api_url.trim_end_matches('/'));
    let include_tools = ollama_supports_tools(model);
    let body = build_ollama_request_body(model, messages, include_tools);

    let client = reqwest::blocking::Client::new();
    let response = client
        .post(&url)
        .header("content-type", "application/json")
        .json(&body)
        .send()
        .map_err(|e| format!("Ollama request failed: {e} — is Ollama running?"))?;

    if !response.status().is_success() {
        let status = response.status().as_u16();
        let body_text = response.text().unwrap_or_default();
        return Err(format!("Ollama error {status}: {body_text}"));
    }

    let reader = std::io::BufReader::new(response);
    process_ollama_lines(agent_id, reader, cancel, sender, include_tools)
}

fn process_ollama_lines(
    agent_id: AgentId,
    reader: impl BufRead,
    cancel: &StreamCancelHandle,
    sender: &EventSender,
    include_tools: bool,
) -> Result<(), String> {
    for raw_line in reader.lines() {
        if cancel.is_cancelled() {
            return Ok(());
        }

        let line = raw_line.map_err(|e| format!("Stream read error: {e}"))?;
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        let obj: serde_json::Value = match serde_json::from_str(line) {
            Ok(v) => v,
            Err(_) => continue,
        };

        if obj.get("error").is_some() {
            let msg = obj["error"].as_str().unwrap_or("Ollama error").to_string();
            return Err(msg);
        }

        if let Some(content) = obj
            .get("message")
            .and_then(|m| m.get("content"))
            .and_then(|c| c.as_str())
        {
            if !content.is_empty() {
                let _ = sender.send(AppEvent::AgentTokenChunk {
                    agent_id,
                    text: content.to_string(),
                });
            }
        }

        if include_tools {
            if obj.get("done").and_then(|d| d.as_bool()).unwrap_or(false) {
                if let Some(tool_calls) = obj
                    .get("message")
                    .and_then(|m| m.get("tool_calls"))
                    .and_then(|calls| calls.as_array())
                {
                    for (index, call) in tool_calls.iter().enumerate() {
                        let Some(name) = call
                            .get("function")
                            .and_then(|f| f.get("name"))
                            .and_then(|n| n.as_str())
                        else {
                            continue;
                        };
                        let arguments = call
                            .get("function")
                            .and_then(|f| f.get("arguments"))
                            .map(|args| {
                                if args.is_string() {
                                    args.as_str().unwrap_or("{}").to_string()
                                } else {
                                    args.to_string()
                                }
                            })
                            .unwrap_or_else(|| "{}".to_string());
                        let tool_use_id = call
                            .get("id")
                            .and_then(|id| id.as_str())
                            .map(str::to_owned)
                            .unwrap_or_else(|| format!("ollama_call_{index}"));
                        let _ = sender.send(AppEvent::AgentToolCallStart {
                            agent_id,
                            tool_use_id,
                            tool_name: name.to_string(),
                            input_json: arguments,
                        });
                    }
                }
                let _ = sender.send(AppEvent::AgentTurnComplete { agent_id });
                return Ok(());
            }
            continue;
        }

        if obj.get("done").and_then(|d| d.as_bool()).unwrap_or(false) {
            let _ = sender.send(AppEvent::AgentTurnComplete { agent_id });
            return Ok(());
        }
    }

    let _ = sender.send(AppEvent::AgentTurnComplete { agent_id });
    Ok(())
}

#[derive(Serialize)]
struct OllamaRequestBody<'a> {
    model: &'a str,
    messages: Vec<OpenAiChatMessage>,
    stream: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    tools: Option<Vec<OpenAiToolSchema>>,
}

#[derive(Serialize)]
struct OpenAiChatMessage {
    role: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tool_calls: Option<Vec<OpenAiWireToolCall>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tool_call_id: Option<String>,
}

#[derive(Serialize)]
struct OpenAiWireToolCall {
    id: String,
    #[serde(rename = "type")]
    kind: &'static str,
    function: OpenAiWireFunctionCall,
}

#[derive(Serialize)]
struct OpenAiWireFunctionCall {
    name: String,
    arguments: String,
}

fn build_ollama_request_body<'a>(
    model: &'a str,
    messages: &[ChatMessage],
    include_tools: bool,
) -> OllamaRequestBody<'a> {
    OllamaRequestBody {
        model,
        messages: if include_tools {
            openai_compatible_messages(messages)
        } else {
            build_flat_openai_messages(messages)
        },
        stream: true,
        tools: if include_tools {
            Some(tools_for_openai(ToolRegistry::all()))
        } else {
            None
        },
    }
}

fn build_flat_openai_messages(messages: &[ChatMessage]) -> Vec<OpenAiChatMessage> {
    let mut out = Vec::with_capacity(messages.len());

    for msg in messages {
        let role = match msg.role {
            MessageRole::User => "user",
            MessageRole::Assistant => "assistant",
        };

        let content: String = msg
            .blocks
            .iter()
            .filter_map(|b| match b {
                ContentBlock::Text(t) => Some(t.as_str()),
                ContentBlock::ToolResult(r) => Some(r.content.as_str()),
                ContentBlock::ToolUse(_) => None,
            })
            .collect::<Vec<_>>()
            .join("\n");

        if !content.is_empty() {
            out.push(OpenAiChatMessage {
                role,
                content: Some(content),
                tool_calls: None,
                tool_call_id: None,
            });
        }
    }

    out
}

fn openai_compatible_messages(messages: &[ChatMessage]) -> Vec<OpenAiChatMessage> {
    let mut out = Vec::new();

    for msg in messages {
        match msg.role {
            MessageRole::User => {
                for block in &msg.blocks {
                    match block {
                        ContentBlock::Text(text) => out.push(OpenAiChatMessage {
                            role: "user",
                            content: Some(text.clone()),
                            tool_calls: None,
                            tool_call_id: None,
                        }),
                        ContentBlock::ToolResult(result) => out.push(OpenAiChatMessage {
                            role: "tool",
                            content: Some(result.content.clone()),
                            tool_calls: None,
                            tool_call_id: Some(result.tool_use_id.clone()),
                        }),
                        ContentBlock::ToolUse(_) => {}
                    }
                }
            }
            MessageRole::Assistant => {
                let mut content_parts = Vec::new();
                let mut tool_calls = Vec::new();
                for block in &msg.blocks {
                    match block {
                        ContentBlock::Text(text) => content_parts.push(text.clone()),
                        ContentBlock::ToolUse(tool) => tool_calls.push(OpenAiWireToolCall {
                            id: tool.id.clone(),
                            kind: "function",
                            function: OpenAiWireFunctionCall {
                                name: tool.name.clone(),
                                arguments: tool.input_json.clone(),
                            },
                        }),
                        ContentBlock::ToolResult(_) => {}
                    }
                }
                let content = if content_parts.is_empty() {
                    None
                } else {
                    Some(content_parts.join("\n"))
                };
                let tool_calls = if tool_calls.is_empty() {
                    None
                } else {
                    Some(tool_calls)
                };
                if content.is_some() || tool_calls.is_some() {
                    out.push(OpenAiChatMessage {
                        role: "assistant",
                        content,
                        tool_calls,
                        tool_call_id: None,
                    });
                }
            }
        }
    }

    out
}

// ---------------------------------------------------------------------------
// OpenAI public API
// ---------------------------------------------------------------------------

/// Spawn a background thread that streams an OpenAI chat turn and fires events.
///
/// Uses the OpenAI `/v1/chat/completions` endpoint with `stream: true`.
/// Token chunks arrive as SSE lines (`data: {...}`); the stream ends with `data: [DONE]`.
pub fn spawn_openai_stream(
    agent_id: AgentId,
    api_key: String,
    model: String,
    messages: Vec<ChatMessage>,
    cancel: StreamCancelHandle,
    sender: EventSender,
) -> std::thread::JoinHandle<()> {
    std::thread::spawn(move || {
        if let Err(msg) = run_openai_stream(agent_id, &api_key, &model, &messages, &cancel, &sender) {
            if !cancel.is_cancelled() {
                let _ = sender.send(AppEvent::AgentApiError { agent_id, message: msg });
            }
        }
    })
}

fn run_openai_stream(
    agent_id: AgentId,
    api_key: &str,
    model: &str,
    messages: &[ChatMessage],
    cancel: &StreamCancelHandle,
    sender: &EventSender,
) -> Result<(), String> {
    let body = build_openai_request_body(model, messages);

    let client = reqwest::blocking::Client::new();
    let response = client
        .post(OPENAI_API_URL)
        .header("authorization", format!("Bearer {api_key}"))
        .header("content-type", "application/json")
        .json(&body)
        .send()
        .map_err(|e| format!("OpenAI request failed: {e}"))?;

    if !response.status().is_success() {
        let status = response.status().as_u16();
        let body_text = response.text().unwrap_or_default();
        let msg = match status {
            401 => "Authentication failed — check OPENAI_API_KEY".to_string(),
            429 => "Rate limit exceeded — please wait and retry".to_string(),
            _ => format!("OpenAI error {status}: {body_text}"),
        };
        return Err(msg);
    }

    let reader = std::io::BufReader::new(response);
    process_openai_sse_lines(agent_id, reader, cancel, sender)
}

fn process_openai_sse_lines(
    agent_id: AgentId,
    reader: impl BufRead,
    cancel: &StreamCancelHandle,
    sender: &EventSender,
) -> Result<(), String> {
    let mut tool_calls: BTreeMap<u64, OpenAiToolCallBuilder> = BTreeMap::new();

    for raw_line in reader.lines() {
        if cancel.is_cancelled() {
            return Ok(());
        }

        let line = raw_line.map_err(|e| format!("Stream read error: {e}"))?;
        let Some(data) = line.strip_prefix("data:") else { continue };
        let data = data.trim();

        if data == "[DONE]" {
            flush_openai_tool_calls(agent_id, &tool_calls, sender);
            let _ = sender.send(AppEvent::AgentTurnComplete { agent_id });
            return Ok(());
        }

        let obj: serde_json::Value = match serde_json::from_str(data) {
            Ok(v) => v,
            Err(_) => continue,
        };

        if let Some(err) = obj.get("error").and_then(|e| e.get("message")).and_then(|m| m.as_str()) {
            return Err(err.to_string());
        }

        if let Some(text) = obj
            .get("choices")
            .and_then(|c| c.get(0))
            .and_then(|c| c.get("delta"))
            .and_then(|d| d.get("content"))
            .and_then(|t| t.as_str())
        {
            if !text.is_empty() {
                let _ = sender.send(AppEvent::AgentTokenChunk {
                    agent_id,
                    text: text.to_string(),
                });
            }
        }

        if let Some(calls) = obj
            .get("choices")
            .and_then(|c| c.get(0))
            .and_then(|c| c.get("delta"))
            .and_then(|d| d.get("tool_calls"))
            .and_then(|calls| calls.as_array())
        {
            for call in calls {
                let index = call.get("index").and_then(|i| i.as_u64()).unwrap_or(0);
                let entry = tool_calls
                    .entry(index)
                    .or_insert_with(OpenAiToolCallBuilder::default);
                if let Some(id) = call.get("id").and_then(|id| id.as_str()) {
                    entry.id = id.to_string();
                }
                if let Some(name) = call
                    .get("function")
                    .and_then(|f| f.get("name"))
                    .and_then(|n| n.as_str())
                {
                    entry.name = name.to_string();
                }
                if let Some(args) = call
                    .get("function")
                    .and_then(|f| f.get("arguments"))
                    .and_then(|a| a.as_str())
                {
                    entry.arguments.push_str(args);
                }
            }
        }

        if obj
            .get("choices")
            .and_then(|c| c.get(0))
            .and_then(|c| c.get("finish_reason"))
            .and_then(|reason| reason.as_str())
            == Some("tool_calls")
        {
            flush_openai_tool_calls(agent_id, &tool_calls, sender);
            tool_calls.clear();
        }
    }

    flush_openai_tool_calls(agent_id, &tool_calls, sender);
    let _ = sender.send(AppEvent::AgentTurnComplete { agent_id });
    Ok(())
}

#[derive(Default)]
struct OpenAiToolCallBuilder {
    id: String,
    name: String,
    arguments: String,
}

fn flush_openai_tool_calls(
    agent_id: AgentId,
    tool_calls: &BTreeMap<u64, OpenAiToolCallBuilder>,
    sender: &EventSender,
) {
    for (index, call) in tool_calls {
        if call.name.is_empty() {
            continue;
        }
        let tool_use_id = if call.id.is_empty() {
            format!("openai_call_{index}")
        } else {
            call.id.clone()
        };
        let _ = sender.send(AppEvent::AgentToolCallStart {
            agent_id,
            tool_use_id,
            tool_name: call.name.clone(),
            input_json: call.arguments.clone(),
        });
    }
}

#[derive(Serialize)]
struct OpenAIRequestBody<'a> {
    model: &'a str,
    messages: Vec<OpenAiChatMessage>,
    stream: bool,
    tools: Vec<OpenAiToolSchema>,
}

fn build_openai_request_body<'a>(model: &'a str, messages: &[ChatMessage]) -> OpenAIRequestBody<'a> {
    OpenAIRequestBody {
        model,
        messages: openai_compatible_messages(messages),
        stream: true,
        tools: tools_for_openai(ToolRegistry::all()),
    }
}

// ---------------------------------------------------------------------------
// Cursor public API
// ---------------------------------------------------------------------------

/// Spawn a background thread that streams a Cursor AI chat turn and fires events.
///
/// Cursor's API is OpenAI-compatible (`/v1/chat/completions`, same SSE format).
pub fn spawn_cursor_stream(
    agent_id: AgentId,
    api_key: String,
    model: String,
    messages: Vec<ChatMessage>,
    cancel: StreamCancelHandle,
    sender: EventSender,
) -> std::thread::JoinHandle<()> {
    std::thread::spawn(move || {
        if let Err(msg) = run_cursor_stream(agent_id, &api_key, &model, &messages, &cancel, &sender) {
            if !cancel.is_cancelled() {
                let _ = sender.send(AppEvent::AgentApiError { agent_id, message: msg });
            }
        }
    })
}

fn run_cursor_stream(
    agent_id: AgentId,
    api_key: &str,
    model: &str,
    messages: &[ChatMessage],
    cancel: &StreamCancelHandle,
    sender: &EventSender,
) -> Result<(), String> {
    let body = build_openai_request_body(model, messages);

    let client = reqwest::blocking::Client::new();
    let response = client
        .post(CURSOR_API_URL)
        .header("authorization", format!("Bearer {api_key}"))
        .header("content-type", "application/json")
        .json(&body)
        .send()
        .map_err(|e| format!("Cursor request failed: {e}"))?;

    if !response.status().is_success() {
        let status = response.status().as_u16();
        let body_text = response.text().unwrap_or_default();
        let msg = match status {
            401 => "Authentication failed — check CURSOR_API_KEY".to_string(),
            429 => "Rate limit exceeded — please wait and retry".to_string(),
            _ => format!("Cursor error {status}: {body_text}"),
        };
        return Err(msg);
    }

    let reader = std::io::BufReader::new(response);
    process_openai_sse_lines(agent_id, reader, cancel, sender)
}

// ---------------------------------------------------------------------------
// Claude internal streaming logic
// ---------------------------------------------------------------------------

fn run_stream(
    agent_id: AgentId,
    api_key: &str,
    model: &str,
    messages: &[ChatMessage],
    cancel: &StreamCancelHandle,
    sender: &EventSender,
) -> Result<(), String> {
    let client = reqwest::blocking::Client::new();
    let body = build_request_body(model, messages)?;

    let response = client
        .post(ANTHROPIC_API_URL)
        .header("x-api-key", api_key)
        .header("anthropic-version", ANTHROPIC_VERSION)
        .header("content-type", "application/json")
        .json(&body)
        .send()
        .map_err(|e| format!("API request failed: {e}"))?;

    if !response.status().is_success() {
        let status = response.status().as_u16();
        let body_text = response.text().unwrap_or_default();
        let msg = match status {
            401 => "Authentication failed — check ANTHROPIC_API_KEY".to_string(),
            429 => "Rate limit exceeded — please wait and retry".to_string(),
            _ => format!("API error {status}: {body_text}"),
        };
        return Err(msg);
    }

    let reader = std::io::BufReader::new(response);
    process_sse_lines(agent_id, reader, cancel, sender)
}

fn process_sse_lines(
    agent_id: AgentId,
    reader: impl BufRead,
    cancel: &StreamCancelHandle,
    sender: &EventSender,
) -> Result<(), String> {
    let mut tool_id = String::new();
    let mut tool_name = String::new();
    let mut tool_input_buf = String::new();
    let mut in_tool_block = false;

    for raw_line in reader.lines() {
        if cancel.is_cancelled() {
            return Ok(());
        }

        let line = raw_line.map_err(|e| format!("Stream read error: {e}"))?;

        // SSE data lines start with "data: "; skip event/comment/empty lines.
        let Some(data) = line.strip_prefix("data:") else {
            continue;
        };
        let data = data.trim();

        let event: ApiStreamEvent = match serde_json::from_str(data) {
            Ok(e) => e,
            Err(_) => continue,
        };

        match event {
            ApiStreamEvent::ContentBlockStart {
                content_block: ContentBlockStart::Text { .. },
                ..
            } => {
                in_tool_block = false;
            }
            ApiStreamEvent::ContentBlockStart {
                content_block: ContentBlockStart::ToolUse { id, name },
                ..
            } => {
                in_tool_block = true;
                tool_id = id;
                tool_name = name;
                tool_input_buf.clear();
            }
            ApiStreamEvent::ContentBlockDelta {
                delta: ContentDelta::TextDelta { text },
                ..
            } => {
                let _ = sender.send(AppEvent::AgentTokenChunk { agent_id, text });
            }
            ApiStreamEvent::ContentBlockDelta {
                delta: ContentDelta::InputJsonDelta { partial_json },
                ..
            } => {
                tool_input_buf.push_str(&partial_json);
            }
            ApiStreamEvent::ContentBlockStop { .. } => {
                if in_tool_block {
                    let _ = sender.send(AppEvent::AgentToolCallStart {
                        agent_id,
                        tool_use_id: tool_id.clone(),
                        tool_name: tool_name.clone(),
                        input_json: std::mem::take(&mut tool_input_buf),
                    });
                    in_tool_block = false;
                }
            }
            ApiStreamEvent::MessageStop => {
                let _ = sender.send(AppEvent::AgentTurnComplete { agent_id });
                return Ok(());
            }
            ApiStreamEvent::Error { error } => {
                return Err(error.message);
            }
            // MessageStart, MessageDelta, Ping — no action needed.
            _ => {}
        }
    }

    // Stream ended without MessageStop (e.g. connection dropped).
    let _ = sender.send(AppEvent::AgentTurnComplete { agent_id });
    Ok(())
}

// ---------------------------------------------------------------------------
// Request body serialisation
// ---------------------------------------------------------------------------

#[derive(Serialize)]
struct RequestBody<'a> {
    model: &'a str,
    max_tokens: u32,
    stream: bool,
    messages: Vec<ApiMessage>,
    tools: Vec<crate::agent::tools::ToolSchema>,
}

#[derive(Serialize)]
struct ApiMessage {
    role: &'static str,
    content: Vec<ApiContent>,
}

#[derive(Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum ApiContent {
    Text {
        text: String,
    },
    ToolUse {
        id: String,
        name: String,
        input: serde_json::Value,
    },
    ToolResult {
        tool_use_id: String,
        content: String,
    },
}

fn build_request_body<'a>(model: &'a str, messages: &[ChatMessage]) -> Result<RequestBody<'a>, String> {
    let mut api_messages = Vec::with_capacity(messages.len());

    for msg in messages {
        let role = match msg.role {
            MessageRole::User => "user",
            MessageRole::Assistant => "assistant",
        };

        let mut content = Vec::with_capacity(msg.blocks.len());
        for block in &msg.blocks {
            match block {
                ContentBlock::Text(text) => {
                    content.push(ApiContent::Text { text: text.clone() });
                }
                ContentBlock::ToolUse(tool) => {
                    let input: serde_json::Value = serde_json::from_str(&tool.input_json)
                        .unwrap_or(serde_json::Value::Object(serde_json::Map::new()));
                    content.push(ApiContent::ToolUse {
                        id: tool.id.clone(),
                        name: tool.name.clone(),
                        input,
                    });
                }
                ContentBlock::ToolResult(result) => {
                    content.push(ApiContent::ToolResult {
                        tool_use_id: result.tool_use_id.clone(),
                        content: result.content.clone(),
                    });
                }
            }
        }

        api_messages.push(ApiMessage { role, content });
    }

    Ok(RequestBody {
        model,
        max_tokens: DEFAULT_MAX_TOKENS,
        stream: true,
        messages: api_messages,
        tools: tools_for_claude(ToolRegistry::all()),
    })
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent::chat::{ChatMessage, ContentBlock, MessageRole, ToolResult, ToolUse};

    #[test]
    fn build_request_body_user_text() {
        let messages = vec![ChatMessage {
            role: MessageRole::User,
            blocks: vec![ContentBlock::Text("hello".to_string())],
        }];
        let body = build_request_body("claude-opus-4-8", &messages).unwrap();
        let json = serde_json::to_value(&body).unwrap();

        assert_eq!(json["model"], "claude-opus-4-8");
        assert_eq!(json["stream"], true);
        assert_eq!(json["messages"][0]["role"], "user");
        assert_eq!(json["messages"][0]["content"][0]["type"], "text");
        assert_eq!(json["messages"][0]["content"][0]["text"], "hello");
    }

    #[test]
    fn build_request_body_tool_use() {
        let messages = vec![ChatMessage {
            role: MessageRole::Assistant,
            blocks: vec![ContentBlock::ToolUse(ToolUse::new(
                "tu_1".to_string(),
                "file.read".to_string(),
                r#"{"path":"src/main.rs"}"#.to_string(),
            ))],
        }];
        let body = build_request_body("claude-opus-4-8", &messages).unwrap();
        let json = serde_json::to_value(&body).unwrap();

        assert_eq!(json["messages"][0]["role"], "assistant");
        assert_eq!(json["messages"][0]["content"][0]["type"], "tool_use");
        assert_eq!(json["messages"][0]["content"][0]["name"], "file.read");
        assert_eq!(json["messages"][0]["content"][0]["input"]["path"], "src/main.rs");
    }

    #[test]
    fn build_request_body_tool_result() {
        let messages = vec![ChatMessage {
            role: MessageRole::User,
            blocks: vec![ContentBlock::ToolResult(ToolResult::ok(
                "tu_1".to_string(),
                "file contents".to_string(),
            ))],
        }];
        let body = build_request_body("claude-opus-4-8", &messages).unwrap();
        let json = serde_json::to_value(&body).unwrap();

        assert_eq!(json["messages"][0]["role"], "user");
        assert_eq!(json["messages"][0]["content"][0]["type"], "tool_result");
        assert_eq!(json["messages"][0]["content"][0]["tool_use_id"], "tu_1");
        assert_eq!(json["messages"][0]["content"][0]["content"], "file contents");
    }

    #[test]
    fn sse_text_delta_fires_token_chunk() {
        use crate::agent::AgentId;
        use crate::events::EventChannel;

        let mut channel = EventChannel::new();
        let sender = channel.sender();
        let id = AgentId::from_u32(1);
        let cancel = StreamCancelHandle::default();

        let sse = concat!(
            "data: {\"type\":\"message_start\"}\n",
            "\n",
            "data: {\"type\":\"content_block_start\",\"index\":0,\"content_block\":{\"type\":\"text\",\"text\":\"\"}}\n",
            "\n",
            "data: {\"type\":\"content_block_delta\",\"index\":0,\"delta\":{\"type\":\"text_delta\",\"text\":\"Hello\"}}\n",
            "\n",
            "data: {\"type\":\"content_block_stop\",\"index\":0}\n",
            "\n",
            "data: {\"type\":\"message_stop\"}\n",
        );

        let reader = std::io::BufReader::new(sse.as_bytes());
        process_sse_lines(id, reader, &cancel, &sender).unwrap();

        let events: Vec<_> = channel.drain_coalesced();
        let has_chunk = events.iter().any(|e| {
            matches!(e, AppEvent::AgentTokenChunk { text, .. } if text == "Hello")
        });
        let has_complete = events.iter().any(|e| {
            matches!(e, AppEvent::AgentTurnComplete { .. })
        });
        assert!(has_chunk, "expected AgentTokenChunk");
        assert!(has_complete, "expected AgentTurnComplete");
    }

    #[test]
    fn sse_tool_call_fires_tool_start() {
        use crate::agent::AgentId;
        use crate::events::EventChannel;

        let mut channel = EventChannel::new();
        let sender = channel.sender();
        let id = AgentId::from_u32(1);
        let cancel = StreamCancelHandle::default();

        let sse = concat!(
            "data: {\"type\":\"content_block_start\",\"index\":0,\"content_block\":{\"type\":\"tool_use\",\"id\":\"tu_abc\",\"name\":\"file.read\"}}\n",
            "\n",
            "data: {\"type\":\"content_block_delta\",\"index\":0,\"delta\":{\"type\":\"input_json_delta\",\"partial_json\":\"{\\\"path\\\":\"}}\n",
            "\n",
            "data: {\"type\":\"content_block_delta\",\"index\":0,\"delta\":{\"type\":\"input_json_delta\",\"partial_json\":\"\\\"foo.rs\\\"}\" }}\n",
            "\n",
            "data: {\"type\":\"content_block_stop\",\"index\":0}\n",
            "\n",
            "data: {\"type\":\"message_stop\"}\n",
        );

        let reader = std::io::BufReader::new(sse.as_bytes());
        process_sse_lines(id, reader, &cancel, &sender).unwrap();

        let events: Vec<_> = channel.drain_coalesced();
        let tool_event = events.iter().find(|e| matches!(e, AppEvent::AgentToolCallStart { .. }));
        assert!(tool_event.is_some(), "expected AgentToolCallStart");
        if let Some(AppEvent::AgentToolCallStart { tool_use_id, tool_name, input_json, .. }) = tool_event {
            assert_eq!(tool_use_id, "tu_abc");
            assert_eq!(tool_name, "file.read");
            assert!(input_json.contains("foo.rs"));
        }
    }

    #[test]
    fn cancel_mid_stream_suppresses_api_error() {
        use crate::agent::AgentId;
        use crate::events::EventChannel;

        let mut channel = EventChannel::new();
        let sender = channel.sender();
        let id = AgentId::from_u32(1);
        let cancel = StreamCancelHandle::default();
        cancel.cancel();

        let sse = "data: {\"type\":\"content_block_delta\",\"index\":0,\"delta\":{\"type\":\"text_delta\",\"text\":\"x\"}}\n";
        let reader = std::io::BufReader::new(sse.as_bytes());
        process_sse_lines(id, reader, &cancel, &sender).unwrap();

        let events: Vec<_> = channel.drain_coalesced();
        assert!(events.is_empty(), "cancelled stream should produce no events");
    }

    #[test]
    fn build_openai_request_body_includes_tools() {
        let messages = vec![ChatMessage {
            role: MessageRole::User,
            blocks: vec![ContentBlock::Text("hello".to_string())],
        }];
        let body = build_openai_request_body("gpt-4o", &messages);
        let json = serde_json::to_value(&body).unwrap();
        assert_eq!(json["tools"][0]["type"], "function");
        assert_eq!(json["tools"][0]["function"]["name"], "file.read");
    }

    #[test]
    fn openai_sse_tool_call_fires_tool_start() {
        use crate::agent::AgentId;
        use crate::events::EventChannel;

        let mut channel = EventChannel::new();
        let sender = channel.sender();
        let id = AgentId::from_u32(1);
        let cancel = StreamCancelHandle::default();

        let sse = concat!(
            "data: {\"choices\":[{\"delta\":{\"tool_calls\":[{\"index\":0,\"id\":\"call_1\",\"type\":\"function\",\"function\":{\"name\":\"file.read\",\"arguments\":\"{\\\"path\\\":\"}}]}}]}\n",
            "\n",
            "data: {\"choices\":[{\"delta\":{\"tool_calls\":[{\"index\":0,\"function\":{\"arguments\":\"\\\"src/main.rs\\\"}\"}}]}}]}\n",
            "\n",
            "data: [DONE]\n",
        );

        let reader = std::io::BufReader::new(sse.as_bytes());
        process_openai_sse_lines(id, reader, &cancel, &sender).unwrap();

        let events: Vec<_> = channel.drain_coalesced();
        let tool_event = events.iter().find(|e| matches!(e, AppEvent::AgentToolCallStart { .. }));
        assert!(tool_event.is_some(), "expected AgentToolCallStart");
        if let Some(AppEvent::AgentToolCallStart { tool_name, input_json, .. }) = tool_event {
            assert_eq!(tool_name, "file.read");
            assert!(input_json.contains("src/main.rs"));
        }
    }

    #[test]
    fn ollama_done_chunk_with_tool_calls_fires_tool_start() {
        use crate::agent::AgentId;
        use crate::events::EventChannel;

        let mut channel = EventChannel::new();
        let sender = channel.sender();
        let id = AgentId::from_u32(1);
        let cancel = StreamCancelHandle::default();

        let line = concat!(
            r#"{"message":{"role":"assistant","content":"","tool_calls":[{"function":{"name":"file.read","arguments":{"path":"src/main.rs"}}}]},"done":true}"#,
            "\n",
        );

        let reader = std::io::BufReader::new(line.as_bytes());
        process_ollama_lines(id, reader, &cancel, &sender, true).unwrap();

        let events: Vec<_> = channel.drain_coalesced();
        assert!(events.iter().any(|e| matches!(e, AppEvent::AgentToolCallStart { .. })));
        assert!(events.iter().any(|e| matches!(e, AppEvent::AgentTurnComplete { .. })));
    }

    /// Integration test — requires ANTHROPIC_API_KEY in the environment.
    /// Run with: ANTHROPIC_API_KEY=sk-... cargo test -- --ignored live_stream
    #[test]
    #[ignore]
    fn live_stream_hello() {
        let api_key = std::env::var("ANTHROPIC_API_KEY").expect("ANTHROPIC_API_KEY required");
        use crate::agent::{AgentId, ChatMessage, ContentBlock, MessageRole};
        use crate::events::EventChannel;

        let mut channel = EventChannel::new();
        let sender = channel.sender();
        let id = AgentId::from_u32(1);
        let cancel = StreamCancelHandle::default();

        let messages = vec![ChatMessage {
            role: MessageRole::User,
            blocks: vec![ContentBlock::Text("Say exactly: hello".to_string())],
        }];

        let handle = spawn_claude_stream(
            id,
            api_key,
            "claude-opus-4-8".to_string(),
            messages,
            cancel,
            sender,
        );
        handle.join().expect("stream thread panicked");

        let events = channel.drain_coalesced();
        assert!(events.iter().any(|e| matches!(e, AppEvent::AgentTurnComplete { .. })));
        assert!(events.iter().any(|e| matches!(e, AppEvent::AgentTokenChunk { .. })));
    }
}
