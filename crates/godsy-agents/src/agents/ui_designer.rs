use async_trait::async_trait;
use godsy_core::ui_spec::{BusinessLogicAnalysis, UiSpec};
use godsy_core::Plan;
use serde::Deserialize;

use crate::agent::{AgentContext, AgentResult, PlanningAgent};
use crate::prompts::UI_DESIGNER_SYSTEM;

use super::{chat_json, parse_json};

#[derive(Deserialize)]
struct UiOutput {
    ui: UiSpec,
    business_logic: BusinessLogicAnalysis,
}

#[derive(Debug)]
pub struct UiDesignerAgent;

#[async_trait]
impl PlanningAgent for UiDesignerAgent {
    fn name(&self) -> &'static str {
        "ui_designer"
    }

    async fn run(&self, ctx: &AgentContext, mut plan: Plan) -> AgentResult<Plan> {
        let endpoints = plan
            .api
            .endpoints
            .iter()
            .map(|e| format!("- {} {} {} ({})", e.id, e.method.as_str(), e.path, e.summary))
            .collect::<Vec<_>>()
            .join("\n");
        let entities = plan
            .data_model
            .entities
            .iter()
            .map(|e| format!("- {} ({})", e.id, e.name))
            .collect::<Vec<_>>()
            .join("\n");
        let user = format!(
            "Stack: {} / {}.\nAPI endpoints:\n{}\nEntities:\n{}\nArchitecture overview: {}",
            plan.stack.language,
            plan.stack.frameworks.join(", "),
            endpoints,
            entities,
            plan.architecture.overview
        );
        let resp = chat_json(ctx, "ui_designer", UI_DESIGNER_SYSTEM, user).await?;
        let parsed: UiOutput = parse_json(&resp.content)?;
        plan.ui = parsed.ui;
        plan.business_logic = parsed.business_logic;
        Ok(plan)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use godsy_llm::MockProvider;
    use std::sync::Arc;

    #[tokio::test]
    async fn populates_ui_and_business_logic() {
        let resp = r#"{
          "ui":{"framework":"React","state_management":"Zustand","design_notes":"",
                "screens":[{"id":"s1","name":"Home","route":"/","purpose":"land",
                            "user_flow":["open app"]}],
                "shared_components":[]},
          "business_logic":{"overview":"x","rules":[],"workflows":[]}
        }"#;
        let p = Arc::new(MockProvider::new(resp));
        let ctx = AgentContext::new(p, "mock");
        let plan = Plan::new("x");
        let out = UiDesignerAgent.run(&ctx, plan).await.unwrap();
        assert_eq!(out.ui.screens.len(), 1);
        assert_eq!(out.ui.framework, "React");
    }
}
