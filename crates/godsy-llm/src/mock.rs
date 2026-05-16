use std::collections::HashMap;
use std::sync::Mutex;

use async_trait::async_trait;

use crate::provider::{ChatRequest, ChatResponse, LlmError, LlmProvider};

/// Test double. Returns canned responses keyed by a substring found in the last
/// user message. Falls back to `default_response`.
#[derive(Debug)]
pub struct MockProvider {
    rules: Mutex<Vec<(String, String)>>,
    default_response: String,
    pub calls: Mutex<Vec<ChatRequest>>,
}

impl MockProvider {
    pub fn new(default_response: impl Into<String>) -> Self {
        Self {
            rules: Mutex::new(Vec::new()),
            default_response: default_response.into(),
            calls: Mutex::new(Vec::new()),
        }
    }

    #[must_use]
    pub fn when_contains(self, needle: impl Into<String>, response: impl Into<String>) -> Self {
        self.rules.lock().unwrap().push((needle.into(), response.into()));
        self
    }

    pub fn call_count(&self) -> usize {
        self.calls.lock().unwrap().len()
    }

    pub fn json_responses_for(map: HashMap<String, String>) -> Self {
        let p = Self::new("{}");
        for (k, v) in map {
            p.rules.lock().unwrap().push((k, v));
        }
        p
    }
}

#[async_trait]
impl LlmProvider for MockProvider {
    fn name(&self) -> &'static str {
        "mock"
    }

    async fn chat(&self, req: ChatRequest) -> Result<ChatResponse, LlmError> {
        let last_user = req
            .messages
            .iter()
            .rev()
            .find(|m| matches!(m.role, crate::provider::Role::User))
            .map(|m| m.content.clone())
            .unwrap_or_default();

        let resp = {
            let rules = self.rules.lock().unwrap();
            rules
                .iter()
                .find(|(needle, _)| last_user.contains(needle))
                .map_or_else(|| self.default_response.clone(), |(_, r)| r.clone())
        };

        self.calls.lock().unwrap().push(req.clone());
        Ok(ChatResponse { content: resp, model: "mock".to_string() })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::provider::{ChatMessage, ChatRequest};

    #[tokio::test]
    async fn returns_default_when_no_match() {
        let p = MockProvider::new("default");
        let resp = p
            .chat(ChatRequest {
                model: "x".into(),
                messages: vec![ChatMessage::user("anything")],
                temperature: None,
                json_mode: false,
            })
            .await
            .unwrap();
        assert_eq!(resp.content, "default");
    }

    #[tokio::test]
    async fn matches_rule_by_substring() {
        let p = MockProvider::new("default").when_contains("architect", "arch-response");
        let resp = p
            .chat(ChatRequest {
                model: "x".into(),
                messages: vec![ChatMessage::user("please architect this")],
                temperature: None,
                json_mode: false,
            })
            .await
            .unwrap();
        assert_eq!(resp.content, "arch-response");
        assert_eq!(p.call_count(), 1);
    }
}
