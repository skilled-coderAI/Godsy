use async_trait::async_trait;
use godsy_core::task::Complexity;
use godsy_core::Plan;
use serde::Deserialize;

use crate::agent::{AgentContext, AgentResult, PlanningAgent};
use crate::prompts::ESTIMATOR_SYSTEM;

use super::{chat_json, parse_json};

#[derive(Deserialize)]
struct EstOutput {
    #[serde(default)]
    oversized_task_ids: Vec<String>,
    #[serde(default)]
    notes: String,
}

/// The Estimator does two things:
///   1. Mechanically flag any L-complexity task as oversized (deterministic).
///   2. Ask the LLM to suggest splits (advisory; recorded in problem.assumptions).
#[derive(Debug)]
pub struct EstimatorAgent;

#[async_trait]
impl PlanningAgent for EstimatorAgent {
    fn name(&self) -> &'static str {
        "estimator"
    }

    async fn run(&self, ctx: &AgentContext, mut plan: Plan) -> AgentResult<Plan> {
        let mechanical: Vec<String> = plan
            .tasks
            .iter()
            .filter(|t| matches!(t.complexity, Complexity::L))
            .map(|t| t.id.clone())
            .collect();

        let task_summary = plan
            .tasks
            .iter()
            .map(|t| {
                format!("- {} (order {}, complexity {:?}): {}", t.id, t.order, t.complexity, t.goal)
            })
            .collect::<Vec<_>>()
            .join("\n");
        let resp = chat_json(ctx, "estimator", ESTIMATOR_SYSTEM, format!("Tasks:\n{task_summary}"))
            .await?;
        let out: EstOutput = parse_json(&resp.content)?;

        let mut all: Vec<String> = mechanical;
        for id in out.oversized_task_ids {
            if !all.contains(&id) {
                all.push(id);
            }
        }

        if !all.is_empty() {
            plan.problem.assumptions.push(format!(
                "Estimator flagged oversized tasks for split: {}. Notes: {}",
                all.join(", "),
                out.notes
            ));
        }
        Ok(plan)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use godsy_core::task::{Complexity, Task};
    use godsy_llm::MockProvider;
    use std::sync::Arc;

    #[tokio::test]
    async fn flags_l_complexity_mechanically() {
        let p = Arc::new(MockProvider::new(r#"{"oversized_task_ids":[],"notes":""}"#));
        let ctx = AgentContext::new(p, "mock");
        let mut plan = Plan::new("x");
        plan.tasks.push(Task {
            id: "big".into(),
            order: 1,
            goal: "g".into(),
            component_refs: vec![],
            inputs: vec![],
            outputs: vec![],
            files_touched: vec![],
            acceptance_criteria: vec![],
            complexity: Complexity::L,
            depends_on: vec![],
        });
        let out = EstimatorAgent.run(&ctx, plan).await.unwrap();
        assert!(out.problem.assumptions.iter().any(|a| a.contains("big")));
    }
}
