use async_trait::async_trait;

use crate::grounder::{GroundingError, GroundingHit, GroundingProvider, GroundingQuery};

/// A grounder that returns no hits. Used when the operator selects
/// `grounding.provider = "none"` and wants the Researcher to rely on
/// LLM priors only.
#[derive(Debug, Default, Clone, Copy)]
pub struct NoopGrounder;

#[async_trait]
impl GroundingProvider for NoopGrounder {
    fn name(&self) -> &'static str {
        "noop"
    }

    async fn search(&self, _q: &GroundingQuery) -> Result<Vec<GroundingHit>, GroundingError> {
        Ok(Vec::new())
    }
}
