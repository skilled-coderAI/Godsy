use std::sync::Mutex;

use async_trait::async_trait;

use crate::grounder::{GroundingError, GroundingHit, GroundingProvider, GroundingQuery};

/// Test double. Returns canned hits for every query and records calls.
#[derive(Debug)]
pub struct MockGrounder {
    hits: Vec<GroundingHit>,
    calls: Mutex<Vec<GroundingQuery>>,
}

impl MockGrounder {
    #[must_use]
    pub fn new(hits: Vec<GroundingHit>) -> Self {
        Self { hits, calls: Mutex::new(Vec::new()) }
    }

    pub fn call_count(&self) -> usize {
        self.calls.lock().unwrap().len()
    }
}

#[async_trait]
impl GroundingProvider for MockGrounder {
    fn name(&self) -> &'static str {
        "mock"
    }

    async fn search(&self, q: &GroundingQuery) -> Result<Vec<GroundingHit>, GroundingError> {
        self.calls.lock().unwrap().push(q.clone());
        Ok(self.hits.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn returns_canned_hits_and_records_calls() {
        let g = MockGrounder::new(vec![GroundingHit {
            title: "t".into(),
            url: "http://x".into(),
            snippet: "s".into(),
            score: None,
        }]);
        let hits = g.search(&GroundingQuery::new("q")).await.unwrap();
        assert_eq!(hits.len(), 1);
        assert_eq!(g.call_count(), 1);
    }
}
