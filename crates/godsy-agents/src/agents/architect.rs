use async_trait::async_trait;
use godsy_core::plan::{Component, DataModel, Entity, Field};
use godsy_core::{Architecture, Plan, StackDecision};
use serde::Deserialize;

use crate::agent::{AgentContext, AgentResult, PlanningAgent};
use crate::prompts::ARCHITECT_SYSTEM;

use super::{chat_json, parse_json};

#[derive(Deserialize)]
struct ArchitectOutput {
    overview: String,
    components: Vec<RawComponent>,
    mermaid_diagram: String,
    data_model: Vec<RawEntity>,
    stack: RawStack,
}

#[derive(Deserialize)]
struct RawComponent {
    id: String,
    name: String,
    responsibility: String,
    #[serde(default)]
    depends_on: Vec<String>,
}

#[derive(Deserialize)]
struct RawEntity {
    id: String,
    name: String,
    fields: Vec<RawField>,
}

#[derive(Deserialize)]
struct RawField {
    name: String,
    ty: String,
    #[serde(default = "default_true")]
    required: bool,
    #[serde(default)]
    notes: Option<String>,
}

fn default_true() -> bool {
    true
}

#[derive(Deserialize)]
struct RawStack {
    language: String,
    #[serde(default)]
    frameworks: Vec<String>,
    storage: String,
    runtime_target: String,
    rationale: String,
    #[serde(default)]
    citation_ids: Vec<String>,
}

#[derive(Debug)]
pub struct ArchitectAgent;

#[async_trait]
impl PlanningAgent for ArchitectAgent {
    fn name(&self) -> &'static str {
        "architect"
    }

    async fn run(&self, ctx: &AgentContext, mut plan: Plan) -> AgentResult<Plan> {
        let citations_hint: Vec<String> =
            plan.citations.iter().map(|c| format!("- {}: {}", c.id, c.snippet)).collect();
        let user = format!(
            "Problem summary:\n{}\n\nKnown citations (use ids in stack.citation_ids when relevant):\n{}",
            plan.problem.summary,
            citations_hint.join("\n")
        );
        let resp = chat_json(ctx, "architect", ARCHITECT_SYSTEM, user).await?;
        let out: ArchitectOutput = parse_json(&resp.content)?;

        plan.architecture = Architecture {
            overview: out.overview,
            components: out
                .components
                .into_iter()
                .map(|c| Component {
                    id: c.id,
                    name: c.name,
                    responsibility: c.responsibility,
                    depends_on: c.depends_on,
                })
                .collect(),
            mermaid_diagram: out.mermaid_diagram,
        };
        plan.data_model = DataModel {
            entities: out
                .data_model
                .into_iter()
                .map(|e| Entity {
                    id: e.id,
                    name: e.name,
                    fields: e
                        .fields
                        .into_iter()
                        .map(|f| Field {
                            name: f.name,
                            ty: f.ty,
                            required: f.required,
                            notes: f.notes,
                        })
                        .collect(),
                })
                .collect(),
        };
        plan.stack = StackDecision {
            language: out.stack.language,
            frameworks: out.stack.frameworks,
            storage: out.stack.storage,
            runtime_target: out.stack.runtime_target,
            rationale: out.stack.rationale,
            citation_ids: out.stack.citation_ids,
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
    async fn populates_architecture() {
        let resp = r#"{
          "overview":"o",
          "components":[{"id":"c1","name":"UI","responsibility":"r"}],
          "mermaid_diagram":"graph TD; A-->B",
          "data_model":[{"id":"e1","name":"Order","fields":[{"name":"id","ty":"int"}]}],
          "stack":{"language":"Rust","frameworks":[],"storage":"SQLite","runtime_target":"desktop","rationale":"x"}
        }"#;
        let p = Arc::new(MockProvider::new(resp));
        let ctx = AgentContext::new(p, "mock");
        let plan = Plan::new("x");
        let out = ArchitectAgent.run(&ctx, plan).await.unwrap();
        assert_eq!(out.architecture.components.len(), 1);
        assert_eq!(out.data_model.entities.len(), 1);
        assert_eq!(out.stack.language, "Rust");
    }
}
