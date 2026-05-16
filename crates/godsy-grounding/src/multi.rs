use std::sync::Arc;

use async_trait::async_trait;

use crate::grounder::{GroundingError, GroundingHit, GroundingProvider, GroundingQuery};

/// Fan-out a single query to several providers and merge their hits in
/// declaration order, capped at `q.max_hits` total. A failure in any single
/// provider is logged via `tracing::warn!` and skipped — the remaining
/// providers continue, so a flaky web gateway does not nuke local KB results.
#[derive(Debug)]
pub struct MultiGrounder {
    providers: Vec<Arc<dyn GroundingProvider>>,
}

impl MultiGrounder {
    #[must_use]
    pub fn new(providers: Vec<Arc<dyn GroundingProvider>>) -> Self {
        Self { providers }
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.providers.is_empty()
    }
}

#[async_trait]
impl GroundingProvider for MultiGrounder {
    fn name(&self) -> &'static str {
        "multi"
    }

    async fn search(&self, q: &GroundingQuery) -> Result<Vec<GroundingHit>, GroundingError> {
        let mut out: Vec<GroundingHit> = Vec::new();
        for p in &self.providers {
            match p.search(q).await {
                Ok(mut hits) => out.append(&mut hits),
                Err(e) => {
                    tracing::warn!(provider = p.name(), error = %e, "grounding provider failed");
                }
            }
            if out.len() >= q.max_hits {
                break;
            }
        }
        if out.len() > q.max_hits {
            out.truncate(q.max_hits);
        }
        Ok(out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mock::MockGrounder;

    #[tokio::test]
    async fn merges_in_declaration_order_and_caps_total() {
        let a = Arc::new(MockGrounder::new(vec![
            GroundingHit {
                title: "a1".into(),
                url: "u1".into(),
                snippet: String::new(),
                score: None,
            },
            GroundingHit {
                title: "a2".into(),
                url: "u2".into(),
                snippet: String::new(),
                score: None,
            },
        ]));
        let b = Arc::new(MockGrounder::new(vec![GroundingHit {
            title: "b1".into(),
            url: "v1".into(),
            snippet: String::new(),
            score: None,
        }]));
        let m = MultiGrounder::new(vec![a, b]);
        let q = GroundingQuery::new("anything").with_max_hits(2);
        let hits = m.search(&q).await.unwrap();
        assert_eq!(hits.len(), 2);
        assert_eq!(hits[0].title, "a1");
        assert_eq!(hits[1].title, "a2");
    }
}
