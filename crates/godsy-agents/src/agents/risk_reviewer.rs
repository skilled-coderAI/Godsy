use async_trait::async_trait;
use godsy_core::plan::{RiskItem, Severity};
use godsy_core::Plan;
use serde::Deserialize;

use crate::agent::{AgentContext, AgentResult, PlanningAgent};
use crate::prompts::RISK_REVIEWER_SYSTEM;

use super::{chat_json, parse_json};

#[derive(Deserialize)]
struct RrOutput {
    risks: Vec<RawRisk>,
}

#[derive(Deserialize)]
struct RawRisk {
    id: String,
    description: String,
    severity: String,
    mitigation: String,
}

fn parse_sev(s: &str) -> Severity {
    match s.to_ascii_lowercase().as_str() {
        "high" => Severity::High,
        "low" => Severity::Low,
        _ => Severity::Medium,
    }
}

#[derive(Debug)]
pub struct RiskReviewerAgent;

#[async_trait]
impl PlanningAgent for RiskReviewerAgent {
    fn name(&self) -> &'static str {
        "risk_reviewer"
    }

    async fn run(&self, ctx: &AgentContext, mut plan: Plan) -> AgentResult<Plan> {
        let user = format!(
            "Stack: {} on {}.\nArchitecture: {}\nTasks count: {}",
            plan.stack.language,
            plan.stack.runtime_target,
            plan.architecture.overview,
            plan.tasks.len()
        );
        let resp = chat_json(ctx, "risk_reviewer", RISK_REVIEWER_SYSTEM, user).await?;
        let out: RrOutput = parse_json(&resp.content)?;
        plan.risks = out
            .risks
            .into_iter()
            .map(|r| RiskItem {
                id: r.id,
                description: r.description,
                severity: parse_sev(&r.severity),
                mitigation: r.mitigation,
            })
            .collect();
        Ok(plan)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use godsy_llm::MockProvider;
    use std::sync::Arc;

    #[tokio::test]
    async fn captures_risks() {
        let resp =
            r#"{"risks":[{"id":"r1","description":"d","severity":"high","mitigation":"m"}]}"#;
        let p = Arc::new(MockProvider::new(resp));
        let ctx = AgentContext::new(p, "mock");
        let out = RiskReviewerAgent.run(&ctx, Plan::new("x")).await.unwrap();
        assert_eq!(out.risks.len(), 1);
        assert_eq!(out.risks[0].severity, Severity::High);
    }
}
