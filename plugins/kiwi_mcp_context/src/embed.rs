use anyhow::{bail, Context, Result};
use serde::Deserialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EmbedBackend {
    Ollama,
    OpenAi,
}

#[derive(Debug, Clone)]
pub struct EmbedClient {
    backend: EmbedBackend,
    ollama_url: String,
    ollama_model: String,
    openai_key: String,
    openai_model: String,
    openai_dimensions: usize,
}

#[derive(Deserialize)]
struct OllamaEmbedResponse {
    embeddings: Vec<Vec<f32>>,
}

#[derive(Deserialize)]
struct OpenAiEmbedResponse {
    data: Vec<OpenAiEmbedDatum>,
}

#[derive(Deserialize)]
struct OpenAiEmbedDatum {
    embedding: Vec<f32>,
}

fn openai_key_from_env() -> Option<String> {
    std::env::var("OPENAI_API_KEY")
        .ok()
        .filter(|value| !value.trim().is_empty())
}

fn default_backend() -> EmbedBackend {
    match std::env::var("EMBED_BACKEND")
        .ok()
        .map(|value| value.to_lowercase())
        .as_deref()
    {
        Some("openai") => EmbedBackend::OpenAi,
        Some("ollama") => EmbedBackend::Ollama,
        _ if openai_key_from_env().is_some() => EmbedBackend::OpenAi,
        _ => EmbedBackend::Ollama,
    }
}

impl EmbedClient {
    pub fn from_env() -> Result<Self> {
        let backend = default_backend();

        let openai_key = if backend == EmbedBackend::OpenAi {
            openai_key_from_env()
                .context("OPENAI_API_KEY is required when EMBED_BACKEND=openai")?
        } else {
            String::new()
        };

        let openai_dimensions = std::env::var("OPENAI_EMBED_DIMENSIONS")
            .ok()
            .and_then(|value| value.parse().ok())
            .unwrap_or(1536);

        Ok(Self {
            backend,
            ollama_url: std::env::var("OLLAMA_URL")
                .unwrap_or_else(|_| "http://127.0.0.1:11434".into()),
            ollama_model: std::env::var("OLLAMA_EMBED_MODEL")
                .unwrap_or_else(|_| "nomic-embed-text".into()),
            openai_key,
            openai_model: std::env::var("OPENAI_EMBED_MODEL")
                .unwrap_or_else(|_| "text-embedding-3-small".into()),
            openai_dimensions,
        })
    }

    pub fn align_to_table_dim(mut self, table_dim: usize) -> Result<Self> {
        match self.backend {
            EmbedBackend::Ollama => {
                if table_dim != self.dim() {
                    if let Some(key) = openai_key_from_env() {
                        self.backend = EmbedBackend::OpenAi;
                        self.openai_key = key;
                        self.openai_dimensions = table_dim;
                        return Ok(self);
                    }
                    bail!(
                        "agent_context_memory uses {table_dim}-dim vectors but Ollama produces 768-dim; \
                         set OPENAI_API_KEY and EMBED_BACKEND=openai to reuse the existing table"
                    );
                }
            }
            EmbedBackend::OpenAi => {
                if table_dim != self.openai_dimensions {
                    if self.openai_model.contains("text-embedding-3") && table_dim <= 1536 {
                        self.openai_dimensions = table_dim;
                    } else {
                        bail!(
                            "agent_context_memory uses {table_dim}-dim vectors but OpenAI model `{}` \
                             is configured for {}-dim output",
                            self.openai_model,
                            self.openai_dimensions
                        );
                    }
                }
            }
        }
        Ok(self)
    }

    pub fn embed(&self, text: &str) -> Result<Vec<f32>> {
        match self.backend {
            EmbedBackend::Ollama => self.embed_ollama(text),
            EmbedBackend::OpenAi => self.embed_openai(text),
        }
    }

    pub fn dim(&self) -> usize {
        match self.backend {
            EmbedBackend::Ollama => 768,
            EmbedBackend::OpenAi => self.openai_dimensions,
        }
    }

    pub fn backend_name(&self) -> &'static str {
        match self.backend {
            EmbedBackend::Ollama => "ollama",
            EmbedBackend::OpenAi => "openai",
        }
    }

    fn embed_ollama(&self, text: &str) -> Result<Vec<f32>> {
        let url = format!("{}/api/embed", self.ollama_url);
        let body = serde_json::json!({ "model": self.ollama_model, "input": text });
        let resp: OllamaEmbedResponse = ureq::post(&url)
            .send_json(&body)
            .with_context(|| format!("Ollama embed request to {url} failed"))?
            .into_json()
            .context("failed to parse Ollama embed response")?;
        resp.embeddings
            .into_iter()
            .next()
            .ok_or_else(|| anyhow::anyhow!("empty embeddings in Ollama response"))
    }

    fn embed_openai(&self, text: &str) -> Result<Vec<f32>> {
        let mut body = serde_json::json!({
            "model": self.openai_model,
            "input": text,
        });
        if self.openai_model.contains("text-embedding-3") {
            body["dimensions"] = serde_json::json!(self.openai_dimensions);
        }

        let resp: OpenAiEmbedResponse = ureq::post("https://api.openai.com/v1/embeddings")
            .set("Authorization", &format!("Bearer {}", self.openai_key))
            .set("Content-Type", "application/json")
            .send_json(&body)
            .context("OpenAI embed request failed")?
            .into_json()
            .context("failed to parse OpenAI embed response")?;
        let embedding = resp
            .data
            .into_iter()
            .next()
            .map(|d| d.embedding)
            .ok_or_else(|| anyhow::anyhow!("empty data in OpenAI embed response"))?;
        if embedding.len() != self.openai_dimensions {
            bail!(
                "OpenAI returned {}-dim embedding but {}-dim is required",
                embedding.len(),
                self.openai_dimensions
            );
        }
        Ok(embedding)
    }
}
