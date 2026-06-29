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
    /// `Some(false)` after Ollama rejects tool-calling for this model.
    pub chat_tools_supported: Option<bool>,
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

#[derive(Deserialize)]
struct TagsResponse {
    models: Vec<TagModel>,
}

#[derive(Deserialize)]
struct TagModel {
    name: String,
}

/// Prefer instruct/chat tags over `-base` pretrain weights when several tags match.
fn model_preference(name: &str) -> (u8, &str) {
    let lower = name.to_ascii_lowercase();
    if lower.contains("-base") || lower.ends_with(":base") {
        (2, name)
    } else if lower.ends_with(":latest") {
        (0, name)
    } else {
        (1, name)
    }
}

/// Pick an installed Ollama model name matching `requested`.
///
/// Accepts an exact tag match, or a base name like `qwen2.5-coder` when only
/// `qwen2.5-coder:7b` (etc.) is installed.
#[must_use]
pub fn resolve_model_name(requested: &str, available: &[String]) -> Option<String> {
    if available.iter().any(|name| name == requested)
        && !requested.to_ascii_lowercase().contains("-base")
    {
        return Some(requested.to_string());
    }

    let base = requested.split(':').next().unwrap_or(requested);
    let mut matches: Vec<&String> = available
        .iter()
        .filter(|name| name.split(':').next() == Some(base))
        .collect();

    if matches.is_empty() {
        return None;
    }

    matches.sort_unstable_by(|a, b| model_preference(a).cmp(&model_preference(b)));
    Some(matches[0].clone())
}

fn chat_error_tools_unsupported(err: &anyhow::Error) -> bool {
    format!("{err:#}")
        .to_ascii_lowercase()
        .contains("does not support tools")
}

fn format_ureq_error(action: &str, base_url: &str, err: ureq::Error) -> anyhow::Error {
    match err {
        ureq::Error::Status(code, response) => {
            let detail = response.into_string().unwrap_or_default();
            let detail = detail.trim();
            let detail = if detail.is_empty() {
                String::new()
            } else {
                format!(" — {detail}")
            };
            anyhow::anyhow!("Ollama {action} failed (HTTP {code} at {base_url}){detail}")
        }
        ureq::Error::Transport(transport) => {
            anyhow::anyhow!(
                "could not reach Ollama at {base_url} during {action}: {transport}"
            )
        }
    }
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
        Self {
            base_url,
            model,
            embed_model,
            chat_tools_supported: None,
        }
    }

    pub fn model_likely_supports_tools(&self) -> bool {
        !self
            .model
            .to_ascii_lowercase()
            .contains("-base")
    }

    /// List model names reported by `GET /api/tags`.
    pub fn list_models(&self) -> Result<Vec<String>> {
        let url = format!("{}/api/tags", self.base_url);
        let response = ureq::get(&url)
            .call()
            .map_err(|err| format_ureq_error("model list", &self.base_url, err))?;
        let tags: TagsResponse = response
            .into_json()
            .context("error: failed to parse Ollama /api/tags response")?;
        Ok(tags.models.into_iter().map(|m| m.name).collect())
    }

    /// Resolve `model` against installed tags and update the client in place.
    pub fn resolve_chat_model(&mut self) -> Result<()> {
        let available = self.list_models()?;
        let resolved = resolve_model_name(&self.model, &available).ok_or_else(|| {
            anyhow::anyhow!(
                "Ollama chat model `{}` is not installed. Available models: {}",
                self.model,
                available.join(", ")
            )
        })?;
        if resolved != self.model {
            println!(
                "note: using Ollama model `{resolved}` (configured as `{}`)",
                self.model
            );
        }
        self.model = resolved.clone();
        if !self.model_likely_supports_tools() {
            self.chat_tools_supported = Some(false);
            eprintln!(
                "warning: model `{resolved}` is a base/pretrain tag and usually cannot call tools; \
                 MCP memory/context/git context gathering still works"
            );
        }
        Ok(())
    }

    /// Resolve `embed_model` against installed tags when needed.
    pub fn resolve_embed_model(&mut self) -> Result<()> {
        let available = self.list_models()?;
        if let Some(resolved) = resolve_model_name(&self.embed_model, &available) {
            if resolved != self.embed_model {
                println!(
                    "note: using Ollama embed model `{resolved}` (configured as `{}`)",
                    self.embed_model
                );
            }
            self.embed_model = resolved;
        }
        Ok(())
    }

    /// Stream a chat response. Pass `tools` to enable model tool-calling.
    /// Retries without tools when the model rejects tool support.
    pub fn chat_stream(
        &mut self,
        messages: &[ChatMessage],
        tools: Option<&[OllamaTool]>,
    ) -> Result<ChatStream> {
        let use_tools = tools.filter(|t| !t.is_empty() && self.chat_tools_supported != Some(false));

        match self.chat_stream_inner(messages, use_tools) {
            Ok(stream) => {
                if use_tools.is_some() {
                    self.chat_tools_supported = Some(true);
                }
                Ok(stream)
            }
            Err(err) if use_tools.is_some() && chat_error_tools_unsupported(&err) => {
                self.chat_tools_supported = Some(false);
                eprintln!(
                    "warning: model `{}` does not support Ollama tool-calling; \
                     MCP context is still used, but git/GitHub tools are disabled for chat",
                    self.model
                );
                self.chat_stream_inner(messages, None)
            }
            Err(err) => Err(err),
        }
    }

    fn chat_stream_inner(
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
            .map_err(|err| format_ureq_error("chat", &self.base_url, err))?;
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
        let response = ureq::post(&url)
            .send_json(&body)
            .map_err(|err| format_ureq_error("embed", &self.base_url, err))?;
        let response: EmbedResponse = response
            .into_json()
            .context("error: failed to parse embed response from Ollama")?;
        response
            .embeddings
            .into_iter()
            .next()
            .ok_or_else(|| anyhow::anyhow!("error: empty embeddings in Ollama response"))
    }
}

#[cfg(test)]
mod tests {
    use super::resolve_model_name;

    #[test]
    fn resolve_model_name_exact_match() {
        let available = vec!["qwen2.5-coder:7b".to_string()];
        assert_eq!(
            resolve_model_name("qwen2.5-coder:7b", &available).as_deref(),
            Some("qwen2.5-coder:7b")
        );
    }

    #[test]
    fn resolve_model_name_matches_base_name_to_tagged_model() {
        let available = vec![
            "nomic-embed-text:latest".to_string(),
            "qwen2.5-coder:7b".to_string(),
        ];
        assert_eq!(
            resolve_model_name("qwen2.5-coder", &available).as_deref(),
            Some("qwen2.5-coder:7b")
        );
    }

    #[test]
    fn resolve_model_name_prefers_latest_tag() {
        let available = vec![
            "llama3.2:3b".to_string(),
            "llama3.2:latest".to_string(),
        ];
        assert_eq!(
            resolve_model_name("llama3.2", &available).as_deref(),
            Some("llama3.2:latest")
        );
    }

    #[test]
    fn resolve_model_name_prefers_non_base_tag() {
        let available = vec![
            "qwen2.5-coder:1.5b-base".to_string(),
            "qwen2.5-coder:7b".to_string(),
        ];
        assert_eq!(
            resolve_model_name("qwen2.5-coder", &available).as_deref(),
            Some("qwen2.5-coder:7b")
        );
    }

    #[test]
    fn resolve_model_name_returns_none_when_missing() {
        let available = vec!["qwen2.5-coder:7b".to_string()];
        assert!(resolve_model_name("mistral", &available).is_none());
    }
}
