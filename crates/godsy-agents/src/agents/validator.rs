use async_trait::async_trait;
use godsy_core::confidence::{ConfidenceReport, SectionConfidence, DEFAULT_CONFIDENCE_THRESHOLD};
use godsy_core::verify::{verify_citations, verify_structure};
use godsy_core::Plan;
use serde::Deserialize;

use crate::agent::{AgentContext, AgentError, AgentResult, PlanningAgent};
use crate::prompts::VALIDATOR_SYSTEM;

use super::{chat_json, parse_json};

#[derive(Deserialize)]
struct VlOutput {
    #[serde(default = "default_threshold")]
    threshold: f32,
    sections: Vec<RawSection>,
}

fn default_threshold() -> f32 {
    DEFAULT_CONFIDENCE_THRESHOLD
}

#[derive(Deserialize)]
struct RawSection {
    section: String,
    score: f32,
    #[serde(default)]
    citation_ids: Vec<String>,
    #[serde(default)]
    notes: Option<String>,
}

/// The Validator does two passes:
///   1. Mechanical structural verification (Layer 3 of `prd.md` §10). If it
///      fails, the plan is rejected immediately — no LLM call.
///   2. LLM confidence scoring per section (Layer 4).
#[derive(Debug)]
pub struct ValidatorAgent;

#[async_trait]
impl PlanningAgent for ValidatorAgent {
    fn name(&self) -> &'static str {
        "validator"
    }

    async fn run(&self, ctx: &AgentContext, mut plan: Plan) -> AgentResult<Plan> {
        if let Err(e) = verify_structure(&plan) {
            return Err(AgentError::Precondition(format!("structural: {e}")));
        }
        if let Err(e) = verify_citations(&plan) {
            return Err(AgentError::Precondition(format!("citations: {e}")));
        }

        let snapshot = serde_json::to_string(&plan).unwrap_or_default();
        let truncated = if snapshot.len() > 8_000 { &snapshot[..8_000] } else { &snapshot };
        let resp =
            chat_json(ctx, "validator", VALIDATOR_SYSTEM, format!("Plan JSON:\n{truncated}"))
                .await?;
        let out: VlOutput = parse_json(&resp.content)?;

        plan.confidence = ConfidenceReport {
            threshold: out.threshold.clamp(0.0, 1.0),
            sections: out
                .sections
                .into_iter()
                .map(|s| {
                    let mut sc = SectionConfidence::new(s.section, s.score);
                    sc.citation_ids = s.citation_ids;
                    sc.notes = s.notes;
                    sc
                })
                .collect(),
        };
        Ok(plan)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use godsy_core::plan::Component;
    use godsy_core::task::{Complexity, Task};
    use godsy_llm::MockProvider;
    use std::sync::Arc;

    fn fixture_plan() -> Plan {
        let mut p = Plan::new("x");
        p.architecture.components.push(Component {
            id: "c1".into(),
            name: "n".into(),
            responsibility: "r".into(),
            depends_on: vec![],
        });
        p.tasks.push(Task {
            id: "t1".into(),
            order: 1,
            goal: "g".into(),
            component_refs: vec!["c1".into()],
            inputs: vec![],
            outputs: vec![],
            files_touched: vec![],
            acceptance_criteria: vec!["x".into()],
            complexity: Complexity::S,
            depends_on: vec![],
        });
        p
    }

    #[tokio::test]
    async fn passes_when_structure_ok() {
        let resp = r#"{"threshold":0.8,"sections":[{"section":"problem","score":0.9}]}"#;
        let p = Arc::new(MockProvider::new(resp));
        let ctx = AgentContext::new(p, "mock");
        let out = ValidatorAgent.run(&ctx, fixture_plan()).await.unwrap();
        assert!(out.confidence.passes());
    }

    #[tokio::test]
    async fn rejects_when_structure_broken() {
        let resp = r#"{"threshold":0.8,"sections":[]}"#;
        let p = Arc::new(MockProvider::new(resp));
        let ctx = AgentContext::new(p, "mock");
        let mut bad = fixture_plan();
        bad.tasks[0].component_refs = vec!["nope".into()];
        let err = ValidatorAgent.run(&ctx, bad).await.unwrap_err();
        assert!(matches!(err, AgentError::Precondition(_)));
    }
}
