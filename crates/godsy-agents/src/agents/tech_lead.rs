use async_trait::async_trait;
use godsy_core::task::{Complexity, Task};
use godsy_core::Plan;
use serde::Deserialize;

use crate::agent::{AgentContext, AgentResult, PlanningAgent};
use crate::prompts::TECH_LEAD_SYSTEM;

use super::{chat_json, parse_json};

#[derive(Deserialize)]
struct TlOutput {
    tasks: Vec<RawTask>,
}

#[derive(Deserialize)]
struct RawTask {
    id: String,
    order: u32,
    goal: String,
    #[serde(default)]
    component_refs: Vec<String>,
    #[serde(default)]
    inputs: Vec<String>,
    #[serde(default)]
    outputs: Vec<String>,
    #[serde(default)]
    files_touched: Vec<String>,
    #[serde(default)]
    acceptance_criteria: Vec<String>,
    complexity: String,
    #[serde(default)]
    depends_on: Vec<String>,
}

fn parse_complexity(s: &str) -> Complexity {
    match s.to_ascii_lowercase().as_str() {
        "s" => Complexity::S,
        "m" => Complexity::M,
        _ => Complexity::L,
    }
}

#[derive(Debug)]
pub struct TechLeadAgent;

#[async_trait]
impl PlanningAgent for TechLeadAgent {
    fn name(&self) -> &'static str {
        "tech_lead"
    }

    async fn run(&self, ctx: &AgentContext, mut plan: Plan) -> AgentResult<Plan> {
        let components = plan
            .architecture
            .components
            .iter()
            .map(|c| format!("- {} ({}): {}", c.id, c.name, c.responsibility))
            .collect::<Vec<_>>()
            .join("\n");
        let user = format!(
            "Architecture overview:\n{}\n\nComponents:\n{}\n\nStack: {} / {}",
            plan.architecture.overview, components, plan.stack.language, plan.stack.runtime_target
        );
        let resp = chat_json(ctx, "tech_lead", TECH_LEAD_SYSTEM, user).await?;
        let out: TlOutput = parse_json(&resp.content)?;

        plan.tasks = out
            .tasks
            .into_iter()
            .map(|t| Task {
                id: t.id,
                order: t.order,
                goal: t.goal,
                component_refs: t.component_refs,
                inputs: t.inputs,
                outputs: t.outputs,
                files_touched: t.files_touched,
                acceptance_criteria: t.acceptance_criteria,
                complexity: parse_complexity(&t.complexity),
                depends_on: t.depends_on,
            })
            .collect();
        plan.tasks.sort_by_key(|t| t.order);
        Ok(plan)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use godsy_llm::MockProvider;
    use std::sync::Arc;

    #[tokio::test]
    async fn populates_tasks() {
        let resp = r#"{"tasks":[
          {"id":"t1","order":1,"goal":"scaffold","component_refs":["c1"],"complexity":"s","acceptance_criteria":["x"]},
          {"id":"t2","order":2,"goal":"db","component_refs":["c1"],"complexity":"m","depends_on":["t1"]}
        ]}"#;
        let p = Arc::new(MockProvider::new(resp));
        let ctx = AgentContext::new(p, "mock");
        let plan = Plan::new("x");
        let out = TechLeadAgent.run(&ctx, plan).await.unwrap();
        assert_eq!(out.tasks.len(), 2);
        assert_eq!(out.tasks[0].id, "t1");
    }
}
