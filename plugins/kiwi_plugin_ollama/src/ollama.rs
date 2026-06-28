use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::VecDeque;
use std::io::{BufRead, BufReader};

#[derive(Debug, Clone)]
pub struct OllamaClient {
    pub base_url: String,
    pub model: String,
    pub embed_model: String,
}

/// A single message in the conversation, compatible with Ollama's /api/chat.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
    /// Present on assistant messages when the model calls tools.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<ChatToolCall>>,
}

impl ChatMessage {
    pub fn user(content: impl Into<String>) -> Self {
        Self { role: "user".into(), content: content.into(), tool_calls: None }
    }
    pub fn assistant(content: impl Into<String>) -> Self {
        Self { role: "assistant".into(), content: content.into(), tool_calls: None }
    }
    pub fn system(content: impl Into<String>) -> Self {
        Self { role: "system".into(), content: content.into(), tool_calls: None }
    }
    pub fn tool_result(content: impl Into<String>) -> Self {
        Self { role: "tool".into(), content: content.into(), tool_calls: None }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatToolCall {
    pub function: ChatToolFunction,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatToolFunction {
    pub name: String,
    pub arguments: Value,
}

/// Tool definition passed to Ollama in the `tools` field.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OllamaTool {
    #[serde(rename = "type")]
    pub kind: String,
    pub function: OllamaToolFunction,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OllamaToolFunction {
    pub name: String,
    pub description: String,
    pub parameters: Value,
}

/// An event emitted by `ChatStream`.
pub enum ChatEvent {
    /// A text token streamed from the model.
    Token(String),
    /// The model wants to call a tool.
    ToolCall { name: String, arguments: Value },
}

// ── internal deserialization structs ─────────────────────────────────────────

#[derive(Deserialize)]
struct RawChunk {
    message: RawMessage,
    #[serde(default)]
    done: bool,
}

#[derive(Deserialize)]
struct RawMessage {
    #[serde(default)]
    content: String,
    #[serde(default)]
    tool_calls: Vec<RawToolCall>,
}

#[derive(Deserialize)]
struct RawToolCall {
    function: RawToolFunction,
}

#[derive(Deserialize)]
struct RawToolFunction {
    name: String,
    arguments: Value,
}

#[derive(Deserialize)]
struct EmbedResponse {
    embeddings: Vec<Vec<f32>>,
}

// ── ChatStream ────────────────────────────────────────────────────────────────

pub struct ChatStream {
    reader: BufReader<Box<dyn std::io::Read + Send>>,
    pending: VecDeque<ChatEvent>,
}

impl Iterator for ChatStream {
    type Item = Result<ChatEvent>;

    fn next(&mut self) -> Option<Self::Item> {
        // Drain buffered events (multiple tool calls from one chunk) first
        if let Some(event) = self.pending.pop_front() {
            return Some(Ok(event));
        }

        loop {
            let mut line = String::new();
            match self.reader.read_line(&mut line) {
                Ok(0) => return None,
                Ok(_) => {
                    let trimmed = line.trim();
                    if trimmed.is_empty() {
                        continue;
                    }
                    match serde_json::from_str::<RawChunk>(trimmed) {
                        Err(e) => {
                            return Some(Err(anyhow::anyhow!("stream parse error: {e}")))
                        }
                        Ok(chunk) => {
                            // Tool calls take priority: buffer them all, return first
                            if !chunk.message.tool_calls.is_empty() {
                                for tc in chunk.message.tool_calls {
                                    self.pending.push_back(ChatEvent::ToolCall {
                                        name: tc.function.name,
                                        arguments: tc.function.arguments,
                                    });
                                }
                                if let Some(event) = self.pending.pop_front() {
                                    return Some(Ok(event));
                                }
                            }
                            if chunk.done {
                                return None;
                            }
                            if !chunk.message.content.is_empty() {
                                return Some(Ok(ChatEvent::Token(chunk.message.content)));
                            }
                            // Empty content, not done — read next line
                        }
                    }
                }
                Err(e) => return Some(Err(anyhow::anyhow!("stream read error: {e}"))),
            }
        }
    }
}

// ── OllamaClient ─────────────────────────────────────────────────────────────

impl OllamaClient {
    pub fn new(base_url: String, model: String, embed_model: String) -> Self {
        Self { base_url, model, embed_model }
    }

    /// Stream a chat response. Pass `tools` to enable model tool-calling.
    pub fn chat_stream(
        &self,
        messages: &[ChatMessage],
        tools: Option<&[OllamaTool]>,
    ) -> Result<ChatStream> {
        let url = format!("{}/api/chat", self.base_url);
        let mut body = serde_json::json!({
            "model": self.model,
            "stream": true,
            "messages": messages,
        });
        if let Some(tools) = tools {
            if !tools.is_empty() {
                body["tools"] = serde_json::to_value(tools).unwrap_or_default();
            }
        }
        let response = ureq::post(&url)
            .send_json(&body)
            .with_context(|| {
                format!(
                    "error: cannot connect to Ollama at {} — is the server running?",
                    self.base_url
                )
            })?;
        Ok(ChatStream {
            reader: BufReader::new(Box::new(response.into_reader())),
            pending: VecDeque::new(),
        })
    }

    pub fn embed(&self, text: &str) -> Result<Vec<f32>> {
        let url = format!("{}/api/embed", self.base_url);
        let body = serde_json::json!({
            "model": self.embed_model,
            "input": text,
        });
        let response: EmbedResponse = ureq::post(&url)
            .send_json(&body)
            .with_context(|| {
                format!(
                    "error: embed request to Ollama at {} failed",
                    self.base_url
                )
            })?
            .into_json()
            .context("error: failed to parse embed response from Ollama")?;
        response
            .embeddings
            .into_iter()
            .next()
            .ok_or_else(|| anyhow::anyhow!("error: empty embeddings in Ollama response"))
    }
}
