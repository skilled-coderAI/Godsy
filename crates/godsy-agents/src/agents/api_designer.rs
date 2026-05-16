use async_trait::async_trait;
use godsy_core::api_spec::ApiSpec;
use godsy_core::Plan;

use crate::agent::{AgentContext, AgentResult, PlanningAgent};
use crate::prompts::API_DESIGNER_SYSTEM;

use super::{chat_json, parse_json};

#[derive(Debug)]
pub struct ApiDesignerAgent;

#[async_trait]
impl PlanningAgent for ApiDesignerAgent {
    fn name(&self) -> &'static str {
        "api_designer"
    }

    async fn run(&self, ctx: &AgentContext, mut plan: Plan) -> AgentResult<Plan> {
        let components = plan
            .architecture
            .components
            .iter()
            .map(|c| format!("- {} ({}): {}", c.id, c.name, c.responsibility))
            .collect::<Vec<_>>()
            .join("\n");
        let entities = plan
            .data_model
            .entities
            .iter()
            .map(|e| {
                let fields = e
                    .fields
                    .iter()
                    .map(|f| format!("{}:{}", f.name, f.ty))
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("- {} ({}): {fields}", e.id, e.name)
            })
            .collect::<Vec<_>>()
            .join("\n");
        let user = format!(
            "Stack: {} on {}.\nComponents:\n{}\nEntities:\n{}",
            plan.stack.language, plan.stack.runtime_target, components, entities
        );
        let resp = chat_json(ctx, "api_designer", API_DESIGNER_SYSTEM, user).await?;
        let parsed: ApiSpec = parse_json(&resp.content)?;
        plan.api = parsed;
        Ok(plan)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use godsy_llm::MockProvider;
    use std::sync::Arc;

    #[tokio::test]
    async fn populates_api_spec() {
        let resp = r#"{
          "base_url":"http://localhost:8080",
          "auth":null,
          "endpoints":[{
            "id":"e1","method":"POST","path":"/deliveries","summary":"create",
            "auth_required":false,"statuses":[{"code":201,"description":"created"}]
          }]
        }"#;
        let p = Arc::new(MockProvider::new(resp));
        let ctx = AgentContext::new(p, "mock");
        let plan = Plan::new("x");
        let out = ApiDesignerAgent.run(&ctx, plan).await.unwrap();
        assert_eq!(out.api.endpoints.len(), 1);
        assert_eq!(out.api.endpoints[0].path, "/deliveries");
    }
}
