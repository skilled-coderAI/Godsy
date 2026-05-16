use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::provider::{ChatRequest, ChatResponse, LlmError, LlmProvider, Role};

#[derive(Debug)]
pub struct OllamaProvider {
    base_url: String,
    api_key: Option<String>,
    client: reqwest::Client,
}

impl OllamaProvider {
    /// Build a provider with an explicit request timeout. Use this when you
    /// have a real config; tests should call [`OllamaProvider::with_default_timeout`].
    pub fn new(base_url: impl Into<String>, timeout: std::time::Duration) -> Self {
        Self {
            base_url: base_url.into(),
            api_key: None,
            client: reqwest::Client::builder().timeout(timeout).build().expect("reqwest client"),
        }
    }

    /// Build a provider that authenticates against a hosted Ollama (Ollama
    /// Cloud at `https://ollama.com` or any reverse-proxied deployment) using
    /// a bearer API key. Empty keys are treated as "no auth" so callers can
    /// pass through optional config values without branching.
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

    #[must_use]
    pub fn with_default_timeout(base_url: impl Into<String>) -> Self {
        Self::new(base_url, std::time::Duration::from_secs(200))
    }

    #[must_use]
    pub fn local() -> Self {
        Self::with_default_timeout("http://localhost:11434")
    }
}

#[derive(Serialize)]
struct OllamaChatRequest<'a> {
    model: &'a str,
    messages: Vec<OllamaMessage<'a>>,
    stream: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    format: Option<&'a str>,
    options: OllamaOptions,
}

#[derive(Serialize)]
struct OllamaOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
}

#[derive(Serialize)]
struct OllamaMessage<'a> {
    role: &'a str,
    content: &'a str,
}

#[derive(Deserialize)]
struct OllamaChatResponse {
    message: OllamaResponseMessage,
    model: String,
}

#[derive(Deserialize)]
struct OllamaResponseMessage {
    content: String,
}

fn role_str(r: Role) -> &'static str {
    match r {
        Role::System => "system",
        Role::User => "user",
        Role::Assistant => "assistant",
    }
}

#[async_trait]
impl LlmProvider for OllamaProvider {
    fn name(&self) -> &'static str {
        "ollama"
    }

    async fn chat(&self, req: ChatRequest) -> Result<ChatResponse, LlmError> {
        let url = format!("{}/api/chat", self.base_url.trim_end_matches('/'));
        let messages: Vec<OllamaMessage<'_>> = req
            .messages
            .iter()
            .map(|m| OllamaMessage { role: role_str(m.role), content: &m.content })
            .collect();

        let body = OllamaChatRequest {
            model: &req.model,
            messages,
            stream: false,
            format: if req.json_mode { Some("json") } else { None },
            options: OllamaOptions { temperature: req.temperature },
        };

        let mut request = self.client.post(&url).json(&body);
        if let Some(key) = &self.api_key {
            request = request.bearer_auth(key);
        }

        let resp = request.send().await.map_err(|e| LlmError::Transport(e.to_string()))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(LlmError::InvalidResponse(format!("{status}: {text}")));
        }

        let parsed: OllamaChatResponse =
            resp.json().await.map_err(|e| LlmError::InvalidResponse(e.to_string()))?;

        Ok(ChatResponse { content: parsed.message.content, model: parsed.model })
    }
}
