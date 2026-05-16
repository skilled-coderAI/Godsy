use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::grounder::{GroundingError, GroundingHit, GroundingProvider, GroundingQuery};

/// Configuration the operator must supply when pointing Godsy at a Vane
/// (Perplexica) instance. Vane's `/api/search` requires both the chat model
/// and the embedding model that *Vane itself* uses internally; this is
/// independent of the planning model Godsy drives via `godsy-llm`.
#[derive(Debug, Clone)]
pub struct VaneSettings {
    pub base_url: String,
    pub focus_mode: String,
    pub chat_provider: String,
    pub chat_model: String,
    pub embedding_provider: String,
    pub embedding_model: String,
    pub optimization_mode: String,
}

impl VaneSettings {
    /// Sensible defaults for a local Vane + Ollama stack.
    #[must_use]
    pub fn local_ollama(base_url: impl Into<String>) -> Self {
        Self {
            base_url: base_url.into(),
            focus_mode: "webSearch".to_string(),
            chat_provider: "ollama".to_string(),
            chat_model: "llama3.1".to_string(),
            embedding_provider: "ollama".to_string(),
            embedding_model: "nomic-embed-text".to_string(),
            optimization_mode: "balanced".to_string(),
        }
    }
}

#[derive(Debug)]
pub struct VaneGrounder {
    settings: VaneSettings,
    client: reqwest::Client,
}

impl VaneGrounder {
    pub fn new(settings: VaneSettings, timeout: std::time::Duration) -> Self {
        Self {
            settings,
            client: reqwest::Client::builder().timeout(timeout).build().expect("reqwest client"),
        }
    }

    #[must_use]
    pub fn with_default_timeout(settings: VaneSettings) -> Self {
        Self::new(settings, std::time::Duration::from_secs(90))
    }
}

#[derive(Serialize)]
struct VxModel<'a> {
    provider: &'a str,
    model: &'a str,
}

#[derive(Serialize)]
struct VxRequest<'a> {
    #[serde(rename = "chatModel")]
    chat_model: VxModel<'a>,
    #[serde(rename = "embeddingModel")]
    embedding_model: VxModel<'a>,
    #[serde(rename = "optimizationMode")]
    optimization_mode: &'a str,
    #[serde(rename = "focusMode")]
    focus_mode: &'a str,
    query: &'a str,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    history: Vec<Vec<&'a str>>,
}

#[derive(Deserialize)]
struct VxResponse {
    #[serde(default)]
    sources: Vec<VxSource>,
}

#[derive(Deserialize)]
struct VxSource {
    #[serde(default)]
    #[serde(rename = "pageContent")]
    page_content: String,
    metadata: VxMetadata,
}

#[derive(Deserialize)]
struct VxMetadata {
    #[serde(default)]
    title: String,
    url: String,
}

pub(crate) fn parse_vane_response(json: &str) -> Result<Vec<GroundingHit>, GroundingError> {
    let raw: VxResponse = serde_json::from_str(json)
        .map_err(|e| GroundingError::InvalidResponse(format!("vane: {e}")))?;
    Ok(raw
        .sources
        .into_iter()
        .map(|s| GroundingHit {
            title: s.metadata.title,
            url: s.metadata.url,
            snippet: s.page_content,
            score: None,
        })
        .collect())
}

#[async_trait]
impl GroundingProvider for VaneGrounder {
    fn name(&self) -> &'static str {
        "vane"
    }

    async fn search(&self, q: &GroundingQuery) -> Result<Vec<GroundingHit>, GroundingError> {
        if self.settings.base_url.is_empty() {
            return Err(GroundingError::Misconfigured("vane base_url is empty".into()));
        }
        let url = format!("{}/api/search", self.settings.base_url.trim_end_matches('/'));
        let body = VxRequest {
            chat_model: VxModel {
                provider: &self.settings.chat_provider,
                model: &self.settings.chat_model,
            },
            embedding_model: VxModel {
                provider: &self.settings.embedding_provider,
                model: &self.settings.embedding_model,
            },
            optimization_mode: &self.settings.optimization_mode,
            focus_mode: &self.settings.focus_mode,
            query: &q.text,
            history: Vec::new(),
        };
        let resp = self
            .client
            .post(&url)
            .json(&body)
            .send()
            .await
            .map_err(|e| GroundingError::Transport(e.to_string()))?;
        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(GroundingError::InvalidResponse(format!("{status}: {text}")));
        }
        let body = resp.text().await.map_err(|e| GroundingError::Transport(e.to_string()))?;
        let mut hits = parse_vane_response(&body)?;
        if hits.len() > q.max_hits {
            hits.truncate(q.max_hits);
        }
        Ok(hits)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_vane_shape() {
        let json = r#"{
          "message":"SQLite is...",
          "sources":[
            {"pageContent":"SQLite is an embedded database",
             "metadata":{"title":"SQLite","url":"https://sqlite.org"}},
            {"pageContent":"Tauri builds desktop apps",
             "metadata":{"title":"Tauri","url":"https://tauri.app"}}
          ]
        }"#;
        let hits = parse_vane_response(json).unwrap();
        assert_eq!(hits.len(), 2);
        assert_eq!(hits[1].url, "https://tauri.app");
        assert_eq!(hits[0].title, "SQLite");
    }

    #[test]
    fn rejects_malformed_json() {
        assert!(parse_vane_response("not json").is_err());
    }
}
