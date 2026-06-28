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

impl EmbedClient {
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

    pub fn dim(&self) -> usize {
        match self.backend {
            EmbedBackend::Ollama => 768,
            EmbedBackend::OpenAi => 1536,
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
        let body = serde_json::json!({ "model": self.openai_model, "input": text });
        let resp: OpenAiEmbedResponse = ureq::post("https://api.openai.com/v1/embeddings")
            .set("Authorization", &format!("Bearer {}", self.openai_key))
            .set("Content-Type", "application/json")
            .send_json(&body)
            .context("OpenAI embed request failed")?
            .into_json()
            .context("failed to parse OpenAI embed response")?;
        resp.data
            .into_iter()
            .next()
            .map(|d| d.embedding)
            .ok_or_else(|| anyhow::anyhow!("empty data in OpenAI embed response"))
    }
}
