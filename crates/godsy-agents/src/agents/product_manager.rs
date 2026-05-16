use async_trait::async_trait;
use godsy_core::{Plan, ProblemStatement};
use serde::Deserialize;

use crate::agent::{AgentContext, AgentResult, PlanningAgent};
use crate::prompts::PRODUCT_MANAGER_SYSTEM;

use super::{chat_json, parse_json};

#[derive(Deserialize)]
struct PmOutput {
    summary: String,
    success_criteria: Vec<String>,
    assumptions: Vec<String>,
    clarifications: Vec<String>,
}

#[derive(Debug)]
pub struct ProductManagerAgent;

#[async_trait]
impl PlanningAgent for ProductManagerAgent {
    fn name(&self) -> &'static str {
        "product_manager"
    }

    async fn run(&self, ctx: &AgentContext, mut plan: Plan) -> AgentResult<Plan> {
        let resp = chat_json(
            ctx,
            "product_manager",
            PRODUCT_MANAGER_SYSTEM,
            format!("Business request:\n\n{}", plan.user_request),
        )
        .await?;
        let out: PmOutput = parse_json(&resp.content)?;
        plan.problem = ProblemStatement {
            summary: out.summary,
            success_criteria: out.success_criteria,
            assumptions: out.assumptions,
            clarifications: out.clarifications,
        };
        Ok(plan)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use godsy_llm::MockProvider;
    use std::sync::Arc;

    #[tokio::test]
    async fn populates_problem_section() {
        let resp = r#"{"summary":"track trucks","success_criteria":["A"],"assumptions":["B"],"clarifications":["C"]}"#;
        let provider = Arc::new(MockProvider::new(resp));
        let ctx = AgentContext::new(provider, "mock");
        let plan = Plan::new("track deliveries");
        let out = ProductManagerAgent.run(&ctx, plan).await.unwrap();
        assert_eq!(out.problem.summary, "track trucks");
        assert_eq!(out.problem.success_criteria, vec!["A"]);
    }
}
