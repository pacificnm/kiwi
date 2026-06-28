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
}

// Ollama /api/embed response
#[derive(Deserialize)]
struct OllamaEmbedResponse {
    embeddings: Vec<Vec<f32>>,
}

// OpenAI /v1/embeddings response
#[derive(Deserialize)]
struct OpenAiEmbedResponse {
    data: Vec<OpenAiEmbedDatum>,
}

#[derive(Deserialize)]
struct OpenAiEmbedDatum {
    embedding: Vec<f32>,
}

impl EmbedClient {
    /// Build from environment variables.
    ///
    /// | Var                | Default                       |
    /// |--------------------|-------------------------------|
    /// | EMBED_BACKEND      | `ollama`                      |
    /// | OLLAMA_URL         | `http://localhost:11434`      |
    /// | OLLAMA_EMBED_MODEL | `nomic-embed-text`            |
    /// | OPENAI_API_KEY     | (required if backend=openai)  |
    /// | OPENAI_EMBED_MODEL | `text-embedding-3-small`      |
    pub fn from_env() -> Result<Self> {
        let backend_str = std::env::var("EMBED_BACKEND").unwrap_or_else(|_| "ollama".into());
        let backend = match backend_str.to_lowercase().as_str() {
            "ollama" => EmbedBackend::Ollama,
            "openai" => EmbedBackend::OpenAi,
            other => bail!("unknown EMBED_BACKEND '{other}'; expected 'ollama' or 'openai'"),
        };

        let openai_key = if backend == EmbedBackend::OpenAi {
            std::env::var("OPENAI_API_KEY")
                .context("OPENAI_API_KEY is required when EMBED_BACKEND=openai")?
        } else {
            String::new()
        };

        Ok(Self {
            backend,
            ollama_url: std::env::var("OLLAMA_URL")
                .unwrap_or_else(|_| "http://localhost:11434".into()),
            ollama_model: std::env::var("OLLAMA_EMBED_MODEL")
                .unwrap_or_else(|_| "nomic-embed-text".into()),
            openai_key,
            openai_model: std::env::var("OPENAI_EMBED_MODEL")
                .unwrap_or_else(|_| "text-embedding-3-small".into()),
        })
    }

    pub fn embed(&self, text: &str) -> Result<Vec<f32>> {
        match self.backend {
            EmbedBackend::Ollama => self.embed_ollama(text),
            EmbedBackend::OpenAi => self.embed_openai(text),
        }
    }

    /// Vector dimension produced by the configured model.
    /// Used to verify / create the PostgreSQL schema.
    pub fn dim(&self) -> usize {
        match self.backend {
            EmbedBackend::Ollama => 768,  // nomic-embed-text
            EmbedBackend::OpenAi => 1536, // text-embedding-3-small
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
            .with_context(|| format!("error: Ollama embed request to {url} failed"))?
            .into_json()
            .context("error: failed to parse Ollama embed response")?;
        resp.embeddings
            .into_iter()
            .next()
            .ok_or_else(|| anyhow::anyhow!("error: empty embeddings in Ollama response"))
    }

    fn embed_openai(&self, text: &str) -> Result<Vec<f32>> {
        let body =
            serde_json::json!({ "model": self.openai_model, "input": text });
        let resp: OpenAiEmbedResponse = ureq::post("https://api.openai.com/v1/embeddings")
            .set("Authorization", &format!("Bearer {}", self.openai_key))
            .set("Content-Type", "application/json")
            .send_json(&body)
            .context("error: OpenAI embed request failed")?
            .into_json()
            .context("error: failed to parse OpenAI embed response")?;
        resp.data
            .into_iter()
            .next()
            .map(|d| d.embedding)
            .ok_or_else(|| anyhow::anyhow!("error: empty data in OpenAI embed response"))
    }
}
