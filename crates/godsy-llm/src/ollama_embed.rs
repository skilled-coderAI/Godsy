use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::embedding::{EmbeddingError, EmbeddingProvider, EmbeddingRequest, EmbeddingResponse};

/// Ollama embedding endpoint: `POST {base_url}/api/embeddings`
/// `{ "model": "...", "prompt": "..." }` -> `{ "embedding": [..] }`.
#[derive(Debug)]
pub struct OllamaEmbedder {
    base_url: String,
    api_key: Option<String>,
    client: reqwest::Client,
}

impl OllamaEmbedder {
    pub fn new(base_url: impl Into<String>, timeout: std::time::Duration) -> Self {
        Self {
            base_url: base_url.into(),
            api_key: None,
            client: reqwest::Client::builder().timeout(timeout).build().expect("reqwest client"),
        }
    }

    pub fn with_api_key(
        base_url: impl Into<String>,
        api_key: impl Into<String>,
        timeout: std::time::Duration,
    ) -> Self {
        let key = api_key.into();
        Self {
            base_url: base_url.into(),
            api_key: if key.is_empty() { None } else { Some(key) },
            client: reqwest::Client::builder().timeout(timeout).build().expect("reqwest client"),
        }
    }
}

#[derive(Serialize)]
struct OllamaEmbedRequest<'a> {
    model: &'a str,
    prompt: &'a str,
}

#[derive(Deserialize)]
struct OllamaEmbedResponse {
    embedding: Vec<f32>,
}

#[async_trait]
impl EmbeddingProvider for OllamaEmbedder {
    fn name(&self) -> &'static str {
        "ollama_embed"
    }

    async fn embed(&self, req: EmbeddingRequest) -> Result<EmbeddingResponse, EmbeddingError> {
        let url = format!("{}/api/embeddings", self.base_url.trim_end_matches('/'));
        let body = OllamaEmbedRequest { model: &req.model, prompt: &req.input };
        let mut request = self.client.post(&url).json(&body);
        if let Some(key) = &self.api_key {
            request = request.bearer_auth(key);
        }
        let resp = request.send().await.map_err(|e| EmbeddingError::Transport(e.to_string()))?;
        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(EmbeddingError::InvalidResponse(format!("{status}: {text}")));
        }
        let parsed: OllamaEmbedResponse =
            resp.json().await.map_err(|e| EmbeddingError::InvalidResponse(e.to_string()))?;
        if parsed.embedding.is_empty() {
            return Err(EmbeddingError::InvalidResponse("empty embedding vector".into()));
        }
        Ok(EmbeddingResponse { vector: parsed.embedding, model: req.model })
    }
}
