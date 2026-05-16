use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::provider::{ChatRequest, ChatResponse, LlmError, LlmProvider, Role};

/// Cloudflare Workers AI provider. Talks to:
/// `POST https://api.cloudflare.com/client/v4/accounts/{account_id}/ai/run/{model}`
/// with `Authorization: Bearer <api_token>`.
///
/// `base_url` is the Cloudflare API root (default
/// `https://api.cloudflare.com/client/v4`); `account_id` and `api_token` are
/// required and validated by `GodsyConfig`.
#[derive(Debug)]
pub struct CloudflareProvider {
    base_url: String,
    account_id: String,
    api_token: String,
    client: reqwest::Client,
}

impl CloudflareProvider {
    pub fn new(
        base_url: impl Into<String>,
        account_id: impl Into<String>,
        api_token: impl Into<String>,
        timeout: std::time::Duration,
    ) -> Self {
        Self {
            base_url: base_url.into(),
            account_id: account_id.into(),
            api_token: api_token.into(),
            client: reqwest::Client::builder().timeout(timeout).build().expect("reqwest client"),
        }
    }
}

#[derive(Serialize)]
struct CfChatRequest<'a> {
    messages: Vec<CfMessage<'a>>,
    stream: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    response_format: Option<CfResponseFormat>,
}

#[derive(Serialize)]
struct CfResponseFormat {
    #[serde(rename = "type")]
    ty: &'static str,
}

#[derive(Serialize)]
struct CfMessage<'a> {
    role: &'a str,
    content: &'a str,
}

#[derive(Deserialize)]
struct CfEnvelope {
    result: CfResult,
    #[serde(default)]
    success: bool,
    #[serde(default)]
    errors: Vec<CfError>,
}

#[derive(Deserialize)]
struct CfResult {
    #[serde(default)]
    response: String,
}

#[derive(Deserialize)]
struct CfError {
    #[serde(default)]
    message: String,
}

fn role_str(r: Role) -> &'static str {
    match r {
        Role::System => "system",
        Role::User => "user",
        Role::Assistant => "assistant",
    }
}

#[async_trait]
impl LlmProvider for CloudflareProvider {
    fn name(&self) -> &'static str {
        "cloudflare_workers"
    }

    async fn chat(&self, req: ChatRequest) -> Result<ChatResponse, LlmError> {
        if self.account_id.is_empty() {
            return Err(LlmError::ModelUnavailable("cloudflare account_id is empty".into()));
        }
        if self.api_token.is_empty() {
            return Err(LlmError::ModelUnavailable("cloudflare api_token is empty".into()));
        }
        let url = format!(
            "{}/accounts/{}/ai/run/{}",
            self.base_url.trim_end_matches('/'),
            self.account_id,
            req.model
        );
        let messages: Vec<CfMessage<'_>> = req
            .messages
            .iter()
            .map(|m| CfMessage { role: role_str(m.role), content: &m.content })
            .collect();
        let body = CfChatRequest {
            messages,
            stream: false,
            temperature: req.temperature,
            response_format: if req.json_mode {
                Some(CfResponseFormat { ty: "json_object" })
            } else {
                None
            },
        };
        let resp = self
            .client
            .post(&url)
            .bearer_auth(&self.api_token)
            .json(&body)
            .send()
            .await
            .map_err(|e| LlmError::Transport(e.to_string()))?;
        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(LlmError::InvalidResponse(format!("{status}: {text}")));
        }
        let parsed: CfEnvelope =
            resp.json().await.map_err(|e| LlmError::InvalidResponse(e.to_string()))?;
        if !parsed.success && !parsed.errors.is_empty() {
            let msg =
                parsed.errors.iter().map(|e| e.message.as_str()).collect::<Vec<_>>().join("; ");
            return Err(LlmError::InvalidResponse(msg));
        }
        Ok(ChatResponse { content: parsed.result.response, model: req.model })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn rejects_empty_account_id() {
        let p = CloudflareProvider::new(
            "https://api.cloudflare.com/client/v4",
            "",
            "tok",
            std::time::Duration::from_secs(5),
        );
        let req = ChatRequest {
            model: "@cf/meta/llama-3.1-8b-instruct".into(),
            messages: vec![],
            temperature: None,
            json_mode: false,
        };
        assert!(p.chat(req).await.is_err());
    }
}
