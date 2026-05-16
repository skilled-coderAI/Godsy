use std::sync::Arc;

use async_trait::async_trait;
use godsy_grounding::{GroundingError, GroundingHit, GroundingProvider, GroundingQuery};
use godsy_llm::{EmbeddingProvider, EmbeddingRequest};

use crate::store::KbStore;

/// Adapts the local SQLite KB to the gateway-agnostic `GroundingProvider`
/// trait, so the planning pipeline can consume KB chunks alongside web hits
/// without any agent code knowing about embeddings.
#[derive(Debug)]
pub struct KbGrounder {
    store: Arc<KbStore>,
    embedder: Arc<dyn EmbeddingProvider>,
    embedding_model: String,
    top_k: usize,
}

impl KbGrounder {
    pub fn new(
        store: Arc<KbStore>,
        embedder: Arc<dyn EmbeddingProvider>,
        embedding_model: impl Into<String>,
        top_k: usize,
    ) -> Self {
        Self { store, embedder, embedding_model: embedding_model.into(), top_k }
    }
}

#[async_trait]
impl GroundingProvider for KbGrounder {
    fn name(&self) -> &'static str {
        "kb"
    }

    async fn search(&self, q: &GroundingQuery) -> Result<Vec<GroundingHit>, GroundingError> {
        let resp = self
            .embedder
            .embed(EmbeddingRequest { model: self.embedding_model.clone(), input: q.text.clone() })
            .await
            .map_err(|e| GroundingError::Transport(format!("kb embed: {e}")))?;
        let cap = self.top_k.min(q.max_hits.max(1));
        let hits = self
            .store
            .search(&resp.vector, cap)
            .map_err(|e| GroundingError::InvalidResponse(format!("kb store: {e}")))?;
        Ok(hits
            .into_iter()
            .map(|h| GroundingHit {
                title: format!("{} #{}", h.document_title, h.ordinal),
                url: format!("kb://{}/{}", h.document_id, h.id),
                snippet: h.text,
                score: Some(h.score),
            })
            .collect())
    }
}
