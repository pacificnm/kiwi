//! LLM API streaming clients for the native chat agent architecture.
//!
//! - Claude: POSTs to the Anthropic Messages API, reads Anthropic SSE format.
//! - Ollama: POSTs to `/api/chat`, reads NDJSON stream (no `data:` prefix).
//!
//! Both spawn a background thread and fire `AppEvent` variants into the `EventSender`.

use std::io::BufRead;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use serde::Serialize;

use crate::agent::{AgentId, ChatMessage, ContentBlock, MessageRole};
use crate::events::{AppEvent, EventSender};

use super::stream_event::{ApiStreamEvent, ContentBlockStart, ContentDelta};

const ANTHROPIC_API_URL: &str = "https://api.anthropic.com/v1/messages";
const ANTHROPIC_VERSION: &str = "2023-06-01";
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
/// per line, no `data:` prefix). Tool calls are not supported; text-only streaming.
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
    let body = build_ollama_request_body(model, messages);

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
    process_ollama_lines(agent_id, reader, cancel, sender)
}

fn process_ollama_lines(
    agent_id: AgentId,
    reader: impl BufRead,
    cancel: &StreamCancelHandle,
    sender: &EventSender,
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

        // Each chunk has `message.content` with the next token fragment.
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

        // `done: true` marks the end of the response.
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
    messages: Vec<OllamaMessage>,
    stream: bool,
}

#[derive(Serialize)]
struct OllamaMessage {
    role: &'static str,
    content: String,
}

fn build_ollama_request_body<'a>(model: &'a str, messages: &[ChatMessage]) -> OllamaRequestBody<'a> {
    let mut ollama_messages = Vec::with_capacity(messages.len());

    for msg in messages {
        let role = match msg.role {
            MessageRole::User => "user",
            MessageRole::Assistant => "assistant",
        };

        // Flatten all content blocks to plain text for Ollama.
        let content: String = msg.blocks.iter().filter_map(|b| match b {
            ContentBlock::Text(t) => Some(t.as_str()),
            // Tool results are rendered as plain text so context is preserved.
            ContentBlock::ToolResult(r) => Some(r.content.as_str()),
            // Tool use blocks have no text representation to send to Ollama.
            ContentBlock::ToolUse(_) => None,
        }).collect::<Vec<_>>().join("\n");

        if !content.is_empty() {
            ollama_messages.push(OllamaMessage { role, content });
        }
    }

    OllamaRequestBody { model, messages: ollama_messages, stream: true }
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
        tools: crate::agent::tools::KiwiTool::all_schemas(),
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
                "read_file".to_string(),
                r#"{"path":"src/main.rs"}"#.to_string(),
            ))],
        }];
        let body = build_request_body("claude-opus-4-8", &messages).unwrap();
        let json = serde_json::to_value(&body).unwrap();

        assert_eq!(json["messages"][0]["role"], "assistant");
        assert_eq!(json["messages"][0]["content"][0]["type"], "tool_use");
        assert_eq!(json["messages"][0]["content"][0]["name"], "read_file");
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
            "data: {\"type\":\"content_block_start\",\"index\":0,\"content_block\":{\"type\":\"tool_use\",\"id\":\"tu_abc\",\"name\":\"read_file\"}}\n",
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
            assert_eq!(tool_name, "read_file");
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
