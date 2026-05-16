use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum GroundingError {
    #[error("transport error: {0}")]
    Transport(String),
    #[error("provider returned invalid response: {0}")]
    InvalidResponse(String),
    #[error("grounding gateway misconfigured: {0}")]
    Misconfigured(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroundingQuery {
    pub text: String,
    /// Upper bound on hits the caller can absorb. Implementations should treat
    /// this as a soft cap, never a hard guarantee.
    pub max_hits: usize,
}

impl GroundingQuery {
    #[must_use]
    pub fn new(text: impl Into<String>) -> Self {
        Self { text: text.into(), max_hits: 8 }
    }

    #[must_use]
    pub fn with_max_hits(mut self, n: usize) -> Self {
        self.max_hits = n;
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GroundingHit {
    pub title: String,
    pub url: String,
    pub snippet: String,
    pub score: Option<f32>,
}

/// Gateway-agnostic web grounding. Both `SearxngGrounder` and
/// `PerplexicaGrounder` implement this; agents consume them through a trait
/// object so the choice is a runtime config decision (`prd.md` §8).
#[async_trait]
pub trait GroundingProvider: Send + Sync + std::fmt::Debug {
    fn name(&self) -> &'static str;
    async fn search(&self, q: &GroundingQuery) -> Result<Vec<GroundingHit>, GroundingError>;
}
