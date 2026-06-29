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
use serde_json::{Map, Value};

use crate::agent::tools::{
    kiwi_tool_id_from_openai, normalize_tool_arguments, normalize_tool_arguments_json,
    ollama_split_models, ollama_supports_tools, ollama_uses_native_tool_calls,
    openai_tool_name, parse_ollama_content_tool_calls, tools_for_claude, tools_for_ollama,
    tools_for_openai, OpenAiToolSchema, ToolRegistry,
};
use crate::agent::{AgentId, ChatMessage, ContentBlock, MessageRole};
use crate::config::ProviderSettings;
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
    tool_profile: String,
    cancel: StreamCancelHandle,
    sender: EventSender,
) -> std::thread::JoinHandle<()> {
    std::thread::spawn(move || {
        if let Err(msg) = run_stream(
            agent_id,
            &api_key,
            &model,
            &messages,
            &tool_profile,
            &cancel,
            &sender,
        ) {
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

/// How prior tool calls are replayed in Ollama chat history.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OllamaHistoryFormat {
    /// `llama3` — OpenAI-style `tool_calls` with object `arguments`.
    NativeToolCalls,
    /// `qwen2.5-coder` — JSON blob in assistant `content`.
    ContentJson,
}

/// Resolved model + tool settings for one Ollama stream request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OllamaStreamPlan {
    pub model: String,
    pub include_tools: bool,
    pub history_format: OllamaHistoryFormat,
}

/// Pick orchestration vs synthesis model for split Ollama setups.
pub fn resolve_ollama_stream(
    settings: &ProviderSettings,
    messages: &[ChatMessage],
    tool_profile: &str,
) -> OllamaStreamPlan {
    let tool_model = settings
        .tool_model
        .as_deref()
        .unwrap_or(settings.model.as_str());
    let code_model = settings
        .code_model
        .as_deref()
        .unwrap_or(settings.model.as_str());

    if ollama_split_models(settings) && messages_end_with_tool_result(messages) {
        return OllamaStreamPlan {
            model: code_model.to_string(),
            include_tools: false,
            history_format: OllamaHistoryFormat::ContentJson,
        };
    }

    let tools = tools_for_ollama(tool_profile);
    let include_tools = ollama_supports_tools(tool_model) && !tools.is_empty();
    let history_format = if ollama_uses_native_tool_calls(tool_model) {
        OllamaHistoryFormat::NativeToolCalls
    } else {
        OllamaHistoryFormat::ContentJson
    };

    OllamaStreamPlan {
        model: tool_model.to_string(),
        include_tools,
        history_format,
    }
}

fn messages_end_with_tool_result(messages: &[ChatMessage]) -> bool {
    let Some(last) = messages.last() else {
        return false;
    };
    last.role == MessageRole::User
        && last
            .blocks
            .iter()
            .any(|block| matches!(block, ContentBlock::ToolResult(_)))
}

/// Spawn a background thread that streams an Ollama chat turn and fires events.
///
/// Uses Ollama's native `/api/chat` endpoint which returns NDJSON (one JSON object
/// per line, no `data:` prefix). Tool calls are sent for known tool-capable models.
pub fn spawn_ollama_stream(
    agent_id: AgentId,
    api_url: String,
    plan: OllamaStreamPlan,
    messages: Vec<ChatMessage>,
    tool_profile: String,
    cancel: StreamCancelHandle,
    sender: EventSender,
) -> std::thread::JoinHandle<()> {
    std::thread::spawn(move || {
        if let Err(msg) = run_ollama_stream(
            agent_id,
            &api_url,
            &plan,
            &messages,
            &tool_profile,
            &cancel,
            &sender,
        ) {
            if !cancel.is_cancelled() {
                let _ = sender.send(AppEvent::AgentApiError {
                    agent_id,
                    message: msg,
                });
            }
        }
    })
}

fn run_ollama_stream(
    agent_id: AgentId,
    api_url: &str,
    plan: &OllamaStreamPlan,
    messages: &[ChatMessage],
    tool_profile: &str,
    cancel: &StreamCancelHandle,
    sender: &EventSender,
) -> Result<(), String> {
    let url = format!("{}/api/chat", api_url.trim_end_matches('/'));
    let tools = tools_for_ollama(tool_profile);
    let include_tools = plan.include_tools && !tools.is_empty();
    let body = build_ollama_request_body(
        &plan.model,
        messages,
        include_tools,
        &tools,
        plan.history_format,
    );

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
    let mut pending_tool_calls: Vec<serde_json::Value> = Vec::new();
    let mut buffered_content = String::new();

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
                buffered_content.push_str(content);
                let _ = sender.send(AppEvent::AgentTokenChunk {
                    agent_id,
                    text: content.to_string(),
                });
            }
        }

        if include_tools {
            if let Some(tool_calls) = obj
                .get("message")
                .and_then(|m| m.get("tool_calls"))
                .and_then(|calls| calls.as_array())
            {
                if !tool_calls.is_empty() {
                    pending_tool_calls.clone_from(tool_calls);
                }
            }
        }

        if obj.get("done").and_then(|d| d.as_bool()).unwrap_or(false) {
            if include_tools {
                if let Some(content) = obj
                    .get("message")
                    .and_then(|m| m.get("content"))
                    .and_then(|c| c.as_str())
                {
                    if !content.is_empty() {
                        buffered_content = content.to_string();
                    }
                }
                if !pending_tool_calls.is_empty() {
                    emit_ollama_tool_calls(agent_id, &pending_tool_calls, sender);
                    return Ok(());
                }
                if emit_ollama_content_tool_calls(agent_id, &buffered_content, sender) {
                    return Ok(());
                }
            }
            let _ = sender.send(AppEvent::AgentTurnComplete { agent_id });
            return Ok(());
        }
    }

    if include_tools {
        if !pending_tool_calls.is_empty() {
            emit_ollama_tool_calls(agent_id, &pending_tool_calls, sender);
            return Ok(());
        }
        if emit_ollama_content_tool_calls(agent_id, &buffered_content, sender) {
            return Ok(());
        }
    }

    let _ = sender.send(AppEvent::AgentTurnComplete { agent_id });
    Ok(())
}

fn emit_ollama_tool_calls(
    agent_id: AgentId,
    tool_calls: &[serde_json::Value],
    sender: &EventSender,
) {
    for (index, call) in tool_calls.iter().enumerate() {
        let Some(wire_name) = call
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
                    normalize_tool_arguments_json(args.as_str().unwrap_or("{}"))
                } else {
                    serde_json::to_string(&normalize_tool_arguments(args.clone()))
                        .unwrap_or_else(|_| "{}".to_string())
                }
            })
            .unwrap_or_else(|| "{}".to_string());
        let tool_use_id = call
            .get("id")
            .and_then(|id| id.as_str())
            .map(str::to_owned)
            .unwrap_or_else(|| format!("ollama_call_{index}"));
        emit_ollama_tool_call_start(
            agent_id,
            &tool_use_id,
            wire_name,
            &arguments,
            sender,
        );
    }
}

fn emit_ollama_content_tool_calls(
    agent_id: AgentId,
    content: &str,
    sender: &EventSender,
) -> bool {
    let calls: Vec<_> = parse_ollama_content_tool_calls(content)
        .into_iter()
        .filter(|call| kiwi_tool_id_from_openai(&call.wire_name).is_some())
        .collect();
    if calls.is_empty() {
        return false;
    }
    for (index, call) in calls.iter().enumerate() {
        let input_json = serde_json::to_string(&normalize_tool_arguments(call.arguments.clone()))
            .unwrap_or_else(|_| "{}".to_string());
        let tool_use_id = format!("ollama_content_call_{index}");
        emit_ollama_tool_call_start(
            agent_id,
            &tool_use_id,
            &call.wire_name,
            &input_json,
            sender,
        );
    }
    true
}

fn emit_ollama_tool_call_start(
    agent_id: AgentId,
    tool_use_id: &str,
    wire_name: &str,
    input_json: &str,
    sender: &EventSender,
) {
    let tool_name = kiwi_tool_id_from_openai(wire_name)
        .unwrap_or(wire_name)
        .to_string();
    let _ = sender.send(AppEvent::AgentToolCallStart {
        agent_id,
        tool_use_id: tool_use_id.to_string(),
        tool_name,
        input_json: input_json.to_string(),
    });
}

#[derive(Serialize)]
struct OllamaRequestBody<'a> {
    model: &'a str,
    messages: Vec<OllamaChatMessage>,
    stream: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    tools: Option<Vec<OpenAiToolSchema>>,
}

#[derive(Serialize)]
struct OllamaChatMessage {
    role: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tool_calls: Option<Vec<OllamaWireToolCall>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tool_call_id: Option<String>,
    /// Required by Ollama when returning tool execution results.
    #[serde(skip_serializing_if = "Option::is_none")]
    tool_name: Option<String>,
}

#[derive(Serialize)]
struct OllamaWireToolCall {
    id: String,
    #[serde(rename = "type")]
    kind: &'static str,
    function: OllamaWireFunctionCall,
}

/// Ollama expects `function.arguments` as a JSON object, not a stringified blob.
#[derive(Serialize)]
struct OllamaWireFunctionCall {
    name: String,
    arguments: Value,
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
    /// Required by Ollama when returning tool execution results.
    #[serde(skip_serializing_if = "Option::is_none")]
    tool_name: Option<String>,
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
    tools: &[crate::agent::tools::KiwiToolDef],
    history_format: OllamaHistoryFormat,
) -> OllamaRequestBody<'a> {
    OllamaRequestBody {
        model,
        messages: if include_tools {
            ollama_compatible_messages(messages, history_format)
        } else {
            build_flat_ollama_messages(messages, history_format)
        },
        stream: true,
        tools: if include_tools {
            Some(tools_for_openai(tools))
        } else {
            None
        },
    }
}

fn wire_tool_arguments(input_json: &str) -> Value {
    serde_json::from_str(input_json)
        .map(normalize_tool_arguments)
        .unwrap_or_else(|_| Value::Object(Map::new()))
}

fn build_flat_ollama_messages(
    messages: &[ChatMessage],
    history_format: OllamaHistoryFormat,
) -> Vec<OllamaChatMessage> {
    if history_format == OllamaHistoryFormat::NativeToolCalls {
        return ollama_compatible_messages(messages, history_format);
    }

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
            out.push(OllamaChatMessage {
                role,
                content: Some(content),
                tool_calls: None,
                tool_call_id: None,
                tool_name: None,
            });
        }
    }

    out
}

fn ollama_tool_use_content(tool: &crate::agent::ToolUse) -> String {
    let wire_name = openai_tool_name(&tool.name);
    let arguments = wire_tool_arguments(&tool.input_json);
    serde_json::json!({
        "name": wire_name,
        "arguments": arguments,
    })
    .to_string()
}

fn ollama_compatible_messages(
    messages: &[ChatMessage],
    history_format: OllamaHistoryFormat,
) -> Vec<OllamaChatMessage> {
    match history_format {
        OllamaHistoryFormat::NativeToolCalls => ollama_native_compatible_messages(messages),
        OllamaHistoryFormat::ContentJson => ollama_content_json_compatible_messages(messages),
    }
}

fn ollama_native_compatible_messages(messages: &[ChatMessage]) -> Vec<OllamaChatMessage> {
    let mut out = Vec::new();

    for msg in messages {
        match msg.role {
            MessageRole::User => {
                for block in &msg.blocks {
                    match block {
                        ContentBlock::Text(text) => out.push(OllamaChatMessage {
                            role: "user",
                            content: Some(text.clone()),
                            tool_calls: None,
                            tool_call_id: None,
                            tool_name: None,
                        }),
                        ContentBlock::ToolResult(result) => out.push(OllamaChatMessage {
                            role: "tool",
                            content: Some(result.content.clone()),
                            tool_calls: None,
                            tool_call_id: Some(result.tool_use_id.clone()),
                            tool_name: wire_tool_name_for_id(messages, &result.tool_use_id),
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
                        ContentBlock::ToolUse(tool) => tool_calls.push(OllamaWireToolCall {
                            id: tool.id.clone(),
                            kind: "function",
                            function: OllamaWireFunctionCall {
                                name: openai_tool_name(&tool.name),
                                arguments: wire_tool_arguments(&tool.input_json),
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
                    out.push(OllamaChatMessage {
                        role: "assistant",
                        content,
                        tool_calls,
                        tool_call_id: None,
                        tool_name: None,
                    });
                }
            }
        }
    }

    out
}

fn ollama_content_json_compatible_messages(messages: &[ChatMessage]) -> Vec<OllamaChatMessage> {
    let mut out = Vec::new();

    for msg in messages {
        match msg.role {
            MessageRole::User => {
                for block in &msg.blocks {
                    match block {
                        ContentBlock::Text(text) => out.push(OllamaChatMessage {
                            role: "user",
                            content: Some(text.clone()),
                            tool_calls: None,
                            tool_call_id: None,
                            tool_name: None,
                        }),
                        ContentBlock::ToolResult(result) => out.push(OllamaChatMessage {
                            role: "tool",
                            content: Some(result.content.clone()),
                            tool_calls: None,
                            tool_call_id: Some(result.tool_use_id.clone()),
                            tool_name: wire_tool_name_for_id(messages, &result.tool_use_id),
                        }),
                        ContentBlock::ToolUse(_) => {}
                    }
                }
            }
            MessageRole::Assistant => {
                let mut content_parts = Vec::new();
                for block in &msg.blocks {
                    match block {
                        ContentBlock::Text(text) => content_parts.push(text.clone()),
                        // Qwen/Ollama models emit and expect tool calls as JSON text in content,
                        // not OpenAI-style `tool_calls` arrays in chat history.
                        ContentBlock::ToolUse(tool) => {
                            content_parts.push(ollama_tool_use_content(tool));
                        }
                        ContentBlock::ToolResult(_) => {}
                    }
                }
                if !content_parts.is_empty() {
                    out.push(OllamaChatMessage {
                        role: "assistant",
                        content: Some(content_parts.join("\n")),
                        tool_calls: None,
                        tool_call_id: None,
                        tool_name: None,
                    });
                }
            }
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
                            tool_name: None,
                        }),
                        ContentBlock::ToolResult(result) => out.push(OpenAiChatMessage {
                            role: "tool",
                            content: Some(result.content.clone()),
                            tool_calls: None,
                            tool_call_id: Some(result.tool_use_id.clone()),
                            tool_name: wire_tool_name_for_id(messages, &result.tool_use_id),
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
                                name: openai_tool_name(&tool.name),
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
                        tool_name: None,
                    });
                }
            }
        }
    }

    out
}

fn wire_tool_name_for_id(messages: &[ChatMessage], tool_use_id: &str) -> Option<String> {
    for msg in messages {
        for block in &msg.blocks {
            if let ContentBlock::ToolUse(tool) = block {
                if tool.id == tool_use_id {
                    let kiwi_name =
                        kiwi_tool_id_from_openai(&tool.name).unwrap_or(tool.name.as_str());
                    return Some(openai_tool_name(kiwi_name));
                }
            }
        }
    }
    None
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
    tool_profile: String,
    cancel: StreamCancelHandle,
    sender: EventSender,
) -> std::thread::JoinHandle<()> {
    std::thread::spawn(move || {
        if let Err(msg) = run_openai_stream(
            agent_id,
            &api_key,
            &model,
            &messages,
            &tool_profile,
            &cancel,
            &sender,
        ) {
            if !cancel.is_cancelled() {
                let _ = sender.send(AppEvent::AgentApiError {
                    agent_id,
                    message: msg,
                });
            }
        }
    })
}

fn run_openai_stream(
    agent_id: AgentId,
    api_key: &str,
    model: &str,
    messages: &[ChatMessage],
    tool_profile: &str,
    cancel: &StreamCancelHandle,
    sender: &EventSender,
) -> Result<(), String> {
    let body = build_openai_request_body(model, messages, tool_profile);

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
        let Some(data) = line.strip_prefix("data:") else {
            continue;
        };
        let data = data.trim();

        if data == "[DONE]" {
            let dispatched = flush_openai_tool_calls(agent_id, &tool_calls, sender);
            if !dispatched {
                let _ = sender.send(AppEvent::AgentTurnComplete { agent_id });
            }
            return Ok(());
        }

        let obj: serde_json::Value = match serde_json::from_str(data) {
            Ok(v) => v,
            Err(_) => continue,
        };

        if let Some(err) = obj
            .get("error")
            .and_then(|e| e.get("message"))
            .and_then(|m| m.as_str())
        {
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
            return Ok(());
        }
    }

    let dispatched = flush_openai_tool_calls(agent_id, &tool_calls, sender);
    if !dispatched {
        let _ = sender.send(AppEvent::AgentTurnComplete { agent_id });
    }
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
) -> bool {
    let mut dispatched = false;
    for (index, call) in tool_calls {
        if call.name.is_empty() {
            continue;
        }
        dispatched = true;
        let tool_use_id = if call.id.is_empty() {
            format!("openai_call_{index}")
        } else {
            call.id.clone()
        };
        let tool_name = kiwi_tool_id_from_openai(&call.name)
            .unwrap_or(call.name.as_str())
            .to_string();
        let _ = sender.send(AppEvent::AgentToolCallStart {
            agent_id,
            tool_use_id,
            tool_name,
            input_json: normalize_tool_arguments_json(&call.arguments),
        });
    }
    dispatched
}

#[derive(Serialize)]
struct OpenAIRequestBody<'a> {
    model: &'a str,
    messages: Vec<OpenAiChatMessage>,
    stream: bool,
    tools: Vec<OpenAiToolSchema>,
}

fn build_openai_request_body<'a>(
    model: &'a str,
    messages: &[ChatMessage],
    tool_profile: &str,
) -> OpenAIRequestBody<'a> {
    let tools = ToolRegistry::for_profile(tool_profile);
    OpenAIRequestBody {
        model,
        messages: openai_compatible_messages(messages),
        stream: true,
        tools: tools_for_openai(&tools),
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
    tool_profile: String,
    cancel: StreamCancelHandle,
    sender: EventSender,
) -> std::thread::JoinHandle<()> {
    std::thread::spawn(move || {
        if let Err(msg) = run_cursor_stream(
            agent_id,
            &api_key,
            &model,
            &messages,
            &tool_profile,
            &cancel,
            &sender,
        ) {
            if !cancel.is_cancelled() {
                let _ = sender.send(AppEvent::AgentApiError {
                    agent_id,
                    message: msg,
                });
            }
        }
    })
}

fn run_cursor_stream(
    agent_id: AgentId,
    api_key: &str,
    model: &str,
    messages: &[ChatMessage],
    tool_profile: &str,
    cancel: &StreamCancelHandle,
    sender: &EventSender,
) -> Result<(), String> {
    let body = build_openai_request_body(model, messages, tool_profile);

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
    tool_profile: &str,
    cancel: &StreamCancelHandle,
    sender: &EventSender,
) -> Result<(), String> {
    let client = reqwest::blocking::Client::new();
    let body = build_request_body(model, messages, tool_profile)?;

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
                    let resolved_name = kiwi_tool_id_from_openai(&tool_name)
                        .unwrap_or(tool_name.as_str())
                        .to_string();
                    let _ = sender.send(AppEvent::AgentToolCallStart {
                        agent_id,
                        tool_use_id: tool_id.clone(),
                        tool_name: resolved_name,
                        input_json: normalize_tool_arguments_json(&tool_input_buf),
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

fn build_request_body<'a>(
    model: &'a str,
    messages: &[ChatMessage],
    tool_profile: &str,
) -> Result<RequestBody<'a>, String> {
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
                    let input: serde_json::Value = normalize_tool_arguments(
                        serde_json::from_str(&tool.input_json)
                            .unwrap_or(serde_json::Value::Object(serde_json::Map::new())),
                    );
                    let kiwi_name = kiwi_tool_id_from_openai(&tool.name)
                        .unwrap_or(tool.name.as_str());
                    content.push(ApiContent::ToolUse {
                        id: tool.id.clone(),
                        name: openai_tool_name(kiwi_name),
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
        tools: tools_for_claude(&ToolRegistry::for_profile(tool_profile)),
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
        let body = build_request_body("claude-opus-4-8", &messages, "all").unwrap();
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
        let body = build_request_body("claude-opus-4-8", &messages, "all").unwrap();
        let json = serde_json::to_value(&body).unwrap();

        assert_eq!(json["messages"][0]["role"], "assistant");
        assert_eq!(json["messages"][0]["content"][0]["type"], "tool_use");
        assert_eq!(json["messages"][0]["content"][0]["name"], "file_read");
        assert_eq!(
            json["messages"][0]["content"][0]["input"]["path"],
            "src/main.rs"
        );
        assert_eq!(json["tools"][0]["name"], "file_read");
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
        let body = build_request_body("claude-opus-4-8", &messages, "all").unwrap();
        let json = serde_json::to_value(&body).unwrap();

        assert_eq!(json["messages"][0]["role"], "user");
        assert_eq!(json["messages"][0]["content"][0]["type"], "tool_result");
        assert_eq!(json["messages"][0]["content"][0]["tool_use_id"], "tu_1");
        assert_eq!(
            json["messages"][0]["content"][0]["content"],
            "file contents"
        );
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
        let has_chunk = events
            .iter()
            .any(|e| matches!(e, AppEvent::AgentTokenChunk { text, .. } if text == "Hello"));
        let has_complete = events
            .iter()
            .any(|e| matches!(e, AppEvent::AgentTurnComplete { .. }));
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
            "data: {\"type\":\"content_block_start\",\"index\":0,\"content_block\":{\"type\":\"tool_use\",\"id\":\"tu_abc\",\"name\":\"file_read\"}}\n",
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
        let tool_event = events
            .iter()
            .find(|e| matches!(e, AppEvent::AgentToolCallStart { .. }));
        assert!(tool_event.is_some(), "expected AgentToolCallStart");
        if let Some(AppEvent::AgentToolCallStart {
            tool_use_id,
            tool_name,
            input_json,
            ..
        }) = tool_event
        {
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
        assert!(
            events.is_empty(),
            "cancelled stream should produce no events"
        );
    }

    #[test]
    fn build_ollama_request_body_uses_content_json_for_tool_history() {
        use crate::agent::{ToolResult, ToolUse};

        let messages = vec![
            ChatMessage {
                role: MessageRole::User,
                blocks: vec![ContentBlock::Text("run cargo check".to_string())],
            },
            ChatMessage {
                role: MessageRole::Assistant,
                blocks: vec![ContentBlock::ToolUse(ToolUse::new(
                    "ollama_content_call_0".to_string(),
                    "cargo.check".to_string(),
                    "{}".to_string(),
                ))],
            },
            ChatMessage {
                role: MessageRole::User,
                blocks: vec![ContentBlock::ToolResult(ToolResult::ok(
                    "ollama_content_call_0".to_string(),
                    "Finished dev".to_string(),
                ))],
            },
        ];

        let body = build_ollama_request_body(
            "qwen2.5-coder:7b",
            &messages,
            true,
            &tools_for_ollama("all"),
            OllamaHistoryFormat::ContentJson,
        );
        let json = serde_json::to_value(&body).unwrap();
        let assistant = &json["messages"][1];
        assert!(assistant["tool_calls"].is_null() || assistant["tool_calls"].as_array().is_none());
        let content = assistant["content"].as_str().expect("assistant content");
        assert!(content.contains("\"name\":\"cargo_check\"") || content.contains("\"name\": \"cargo_check\""));
        assert_eq!(json["messages"][2]["tool_name"], "cargo_check");
        let tool_names: Vec<_> = json["tools"]
            .as_array()
            .unwrap()
            .iter()
            .map(|t| t["function"]["name"].as_str().unwrap())
            .collect();
        assert!(!tool_names.contains(&"shell_run"));
    }

    #[test]
    fn build_ollama_request_body_uses_native_tool_calls_for_llama_history() {
        use crate::agent::{ToolResult, ToolUse};

        let messages = vec![
            ChatMessage {
                role: MessageRole::User,
                blocks: vec![ContentBlock::Text("run cargo check".to_string())],
            },
            ChatMessage {
                role: MessageRole::Assistant,
                blocks: vec![ContentBlock::ToolUse(ToolUse::new(
                    "call_1".to_string(),
                    "cargo.check".to_string(),
                    "{}".to_string(),
                ))],
            },
            ChatMessage {
                role: MessageRole::User,
                blocks: vec![ContentBlock::ToolResult(ToolResult::ok(
                    "call_1".to_string(),
                    "Finished dev".to_string(),
                ))],
            },
        ];

        let body = build_ollama_request_body(
            "llama3.1:8b",
            &messages,
            true,
            &tools_for_ollama("all"),
            OllamaHistoryFormat::NativeToolCalls,
        );
        let json = serde_json::to_value(&body).unwrap();
        let args = &json["messages"][1]["tool_calls"][0]["function"]["arguments"];
        assert!(args.is_object());
    }

    #[test]
    fn resolve_ollama_stream_split_mode_picks_models_by_phase() {
        use crate::config::ProviderSettings;

        let settings = ProviderSettings {
            api_key_env: String::new(),
            api_key: None,
            model: "qwen2.5-coder:7b".to_string(),
            api_url: Some("http://localhost:11434".to_string()),
            tool_profile: None,
            tool_model: Some("llama3.1:8b".to_string()),
            code_model: Some("qwen2.5-coder:7b".to_string()),
            embedding_model: None,
        };

        let user_only = vec![ChatMessage {
            role: MessageRole::User,
            blocks: vec![ContentBlock::Text("run cargo check".to_string())],
        }];
        let orchestrate = resolve_ollama_stream(&settings, &user_only, "coding");
        assert_eq!(orchestrate.model, "llama3.1:8b");
        assert!(orchestrate.include_tools);
        assert_eq!(orchestrate.history_format, OllamaHistoryFormat::NativeToolCalls);

        let after_tool = vec![
            user_only[0].clone(),
            ChatMessage {
                role: MessageRole::Assistant,
                blocks: vec![ContentBlock::ToolUse(crate::agent::ToolUse::new(
                    "c1".into(),
                    "cargo.check".into(),
                    "{}".into(),
                ))],
            },
            ChatMessage {
                role: MessageRole::User,
                blocks: vec![ContentBlock::ToolResult(crate::agent::ToolResult::ok(
                    "c1".into(),
                    "ok".into(),
                ))],
            },
        ];
        let synthesize = resolve_ollama_stream(&settings, &after_tool, "coding");
        assert_eq!(synthesize.model, "qwen2.5-coder:7b");
        assert!(!synthesize.include_tools);
        assert_eq!(synthesize.history_format, OllamaHistoryFormat::ContentJson);
    }

    #[test]
    fn tools_for_ollama_excludes_shell_run() {
        let tools = tools_for_ollama("all");
        assert!(!tools.iter().any(|t| t.id == "shell.run"));
        assert!(tools.iter().any(|t| t.id == "cargo.check"));
    }

    #[test]
    fn build_openai_request_body_includes_tools() {
        let messages = vec![ChatMessage {
            role: MessageRole::User,
            blocks: vec![ContentBlock::Text("hello".to_string())],
        }];
        let body = build_openai_request_body("gpt-4o", &messages, "all");
        let json = serde_json::to_value(&body).unwrap();
        assert_eq!(json["tools"][0]["type"], "function");
        assert_eq!(json["tools"][0]["function"]["name"], "file_read");
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
            "data: {\"choices\":[{\"delta\":{\"tool_calls\":[{\"index\":0,\"id\":\"call_1\",\"type\":\"function\",\"function\":{\"name\":\"file_read\",\"arguments\":\"{\\\"path\\\":\"}}]}}]}\n",
            "\n",
            "data: {\"choices\":[{\"delta\":{\"tool_calls\":[{\"index\":0,\"function\":{\"arguments\":\"\\\"src/main.rs\\\"}\"}}]}}]}\n",
            "\n",
            "data: [DONE]\n",
        );

        let reader = std::io::BufReader::new(sse.as_bytes());
        process_openai_sse_lines(id, reader, &cancel, &sender).unwrap();

        let events: Vec<_> = channel.drain_coalesced();
        let tool_event = events
            .iter()
            .find(|e| matches!(e, AppEvent::AgentToolCallStart { .. }));
        assert!(tool_event.is_some(), "expected AgentToolCallStart");
        if let Some(AppEvent::AgentToolCallStart {
            tool_name,
            input_json,
            ..
        }) = tool_event
        {
            assert_eq!(tool_name, "file.read");
            assert!(input_json.contains("src/main.rs"));
        }
    }

    #[test]
    fn ollama_done_chunk_with_null_optional_argument() {
        use crate::agent::AgentId;
        use crate::events::EventChannel;

        let mut channel = EventChannel::new();
        let sender = channel.sender();
        let id = AgentId::from_u32(1);
        let cancel = StreamCancelHandle::default();

        let line = concat!(
            r#"{"message":{"role":"assistant","content":"","tool_calls":[{"function":{"name":"cargo_check","arguments":{"package":null}}}]},"done":true}"#,
            "\n",
        );

        let reader = std::io::BufReader::new(line.as_bytes());
        process_ollama_lines(id, reader, &cancel, &sender, true).unwrap();

        let events: Vec<_> = channel.drain_coalesced();
        let tool_event = events
            .iter()
            .find(|e| matches!(e, AppEvent::AgentToolCallStart { .. }));
        assert!(tool_event.is_some(), "expected AgentToolCallStart");
        if let Some(AppEvent::AgentToolCallStart {
            tool_name,
            input_json,
            ..
        }) = tool_event
        {
            assert_eq!(tool_name, "cargo.check");
            assert_eq!(input_json, "{}");
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
            r#"{"message":{"role":"assistant","content":"","tool_calls":[{"function":{"name":"file_read","arguments":{"path":"src/main.rs"}}}]},"done":true}"#,
            "\n",
        );

        let reader = std::io::BufReader::new(line.as_bytes());
        process_ollama_lines(id, reader, &cancel, &sender, true).unwrap();

        let events: Vec<_> = channel.drain_coalesced();
        assert!(events
            .iter()
            .any(|e| matches!(e, AppEvent::AgentToolCallStart { .. })));
        assert!(
            !events
                .iter()
                .any(|e| matches!(e, AppEvent::AgentTurnComplete { .. })),
            "turn should stay open until tool result follow-up completes"
        );
    }

    #[test]
    fn ollama_content_json_in_stream_fires_tool_start() {
        use crate::agent::AgentId;
        use crate::events::EventChannel;

        let mut channel = EventChannel::new();
        let sender = channel.sender();
        let id = AgentId::from_u32(1);
        let cancel = StreamCancelHandle::default();

        // Simulates qwen2.5-coder: tool JSON streamed in content, tool_calls absent.
        let lines = concat!(
            r#"{"message":{"role":"assistant","content":"{\n"},"done":false}"#,
            "\n",
            r#"{"message":{"role":"assistant","content":"  \"name\": \"cargo_check\",\n"},"done":false}"#,
            "\n",
            r#"{"message":{"role":"assistant","content":"  \"arguments\": {}\n}"},"done":false}"#,
            "\n",
            r#"{"message":{"role":"assistant","content":""},"done":true,"done_reason":"stop"}"#,
            "\n",
        );

        let reader = std::io::BufReader::new(lines.as_bytes());
        process_ollama_lines(id, reader, &cancel, &sender, true).unwrap();

        let events: Vec<_> = channel.drain_coalesced();
        let tool_event = events
            .iter()
            .find(|e| matches!(e, AppEvent::AgentToolCallStart { .. }));
        assert!(tool_event.is_some(), "expected content-json tool call");
        if let Some(AppEvent::AgentToolCallStart {
            tool_name,
            input_json,
            ..
        }) = tool_event
        {
            assert_eq!(tool_name, "cargo.check");
            assert_eq!(input_json, "{}");
        }
        assert!(
            !events
                .iter()
                .any(|e| matches!(e, AppEvent::AgentTurnComplete { .. }))
        );
    }

    #[test]
    fn openai_compatible_messages_include_tool_name_on_tool_results() {
        use crate::agent::{ChatMessage, ContentBlock, MessageRole, ToolResult, ToolUse};

        let messages = vec![
            ChatMessage {
                role: MessageRole::User,
                blocks: vec![ContentBlock::Text("run cargo check".to_string())],
            },
            ChatMessage {
                role: MessageRole::Assistant,
                blocks: vec![ContentBlock::ToolUse(ToolUse::new(
                    "call_1".to_string(),
                    "cargo.check".to_string(),
                    "{}".to_string(),
                ))],
            },
            ChatMessage {
                role: MessageRole::User,
                blocks: vec![ContentBlock::ToolResult(ToolResult::ok(
                    "call_1".to_string(),
                    "Finished dev [unoptimized]".to_string(),
                ))],
            },
        ];

        let wire = openai_compatible_messages(&messages);
        let tool_msg = wire
            .iter()
            .find(|m| m.role == "tool")
            .expect("tool result message");
        assert_eq!(tool_msg.tool_call_id.as_deref(), Some("call_1"));
        assert_eq!(tool_msg.tool_name.as_deref(), Some("cargo_check"));
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
            "all".to_string(),
            cancel,
            sender,
        );
        handle.join().expect("stream thread panicked");

        let events = channel.drain_coalesced();
        assert!(events
            .iter()
            .any(|e| matches!(e, AppEvent::AgentTurnComplete { .. })));
        assert!(events
            .iter()
            .any(|e| matches!(e, AppEvent::AgentTokenChunk { .. })));
    }
}
