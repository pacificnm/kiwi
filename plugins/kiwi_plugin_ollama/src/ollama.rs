use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::io::{BufRead, BufReader};

#[derive(Debug, Clone)]
pub struct OllamaClient {
    pub base_url: String,
    pub model: String,
    pub embed_model: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

#[derive(Deserialize)]
struct ChatStreamChunk {
    message: StreamMessageContent,
    done: bool,
}

#[derive(Deserialize)]
struct StreamMessageContent {
    content: String,
}

#[derive(Deserialize)]
struct EmbedResponse {
    embeddings: Vec<Vec<f32>>,
}

pub struct ChatStream {
    reader: BufReader<Box<dyn std::io::Read + Send>>,
}

impl Iterator for ChatStream {
    type Item = Result<String>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let mut line = String::new();
            match self.reader.read_line(&mut line) {
                Ok(0) => return None,
                Ok(_) => {
                    let trimmed = line.trim();
                    if trimmed.is_empty() {
                        continue;
                    }
                    match serde_json::from_str::<ChatStreamChunk>(trimmed) {
                        Ok(chunk) if chunk.done => return None,
                        Ok(chunk) => return Some(Ok(chunk.message.content)),
                        Err(e) => {
                            return Some(Err(anyhow::anyhow!("stream parse error: {e}")))
                        }
                    }
                }
                Err(e) => return Some(Err(anyhow::anyhow!("stream read error: {e}"))),
            }
        }
    }
}

impl OllamaClient {
    pub fn new(base_url: String, model: String, embed_model: String) -> Self {
        Self {
            base_url,
            model,
            embed_model,
        }
    }

    pub fn chat_stream(&self, messages: &[ChatMessage]) -> Result<ChatStream> {
        let url = format!("{}/api/chat", self.base_url);
        let body = serde_json::json!({
            "model": self.model,
            "stream": true,
            "messages": messages,
        });
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
