use async_trait::async_trait;
use godsy_core::{Citation, CitationKind, Plan};
use godsy_grounding::GroundingQuery;
use serde::Deserialize;
use uuid::Uuid;

use crate::agent::{AgentContext, AgentResult, PlanningAgent};
use crate::prompts::RESEARCHER_SYSTEM;

use super::{chat_json, parse_json};

#[derive(Deserialize)]
struct ResearcherOutput {
    findings: Vec<Finding>,
}

#[derive(Deserialize)]
struct Finding {
    id: String,
    source_url: String,
    snippet: String,
}

/// The Researcher follows a two-step process when grounding is enabled:
///   1. Ask the configured `GroundingProvider` for web hits and seed them
///      into `plan.citations` as `CitationKind::Web` — these are *real*,
///      url-bearing citations the downstream Architect can reference.
///   2. Ask the LLM to add any further *prior-art* findings on top, using the
///      grounded hits as context to anchor reasoning.
///
/// When the grounder is `NoopGrounder` (the offline default), step 1 is a
/// no-op and only the LLM provides candidate citations.
#[derive(Debug)]
pub struct ResearcherAgent;

#[async_trait]
impl PlanningAgent for ResearcherAgent {
    fn name(&self) -> &'static str {
        "researcher"
    }

    async fn run(&self, ctx: &AgentContext, mut plan: Plan) -> AgentResult<Plan> {
        let mut grounded_block = String::new();
        let query = GroundingQuery::new(format!("{} {}", plan.user_request, plan.problem.summary))
            .with_max_hits(6);
        let hits = ctx.grounder.search(&query).await?;
        if !hits.is_empty() {
            grounded_block.push_str("\n\nGrounded web hits (use these ids if you cite them):\n");
            for h in &hits {
                let id = format!("g-{}", Uuid::new_v4().simple());
                grounded_block.push_str(&format!("- id={id} url={} title={}\n", h.url, h.title));
                plan.citations.push(Citation::new(
                    id,
                    CitationKind::Web { url: h.url.clone() },
                    h.snippet.clone(),
                ));
            }
        }

        let user = format!(
            "Problem:\n{}\n\nSummary:\n{}{}",
            plan.user_request, plan.problem.summary, grounded_block
        );
        let resp = chat_json(ctx, "researcher", RESEARCHER_SYSTEM, user).await?;
        let out: ResearcherOutput = parse_json(&resp.content)?;
        for f in out.findings {
            if f.source_url.is_empty() {
                continue;
            }
            plan.citations.push(Citation::new(
                f.id,
                CitationKind::Web { url: f.source_url },
                f.snippet,
            ));
        }
        Ok(plan)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use godsy_grounding::{GroundingHit, MockGrounder};
    use godsy_llm::MockProvider;
    use std::sync::Arc;

    #[tokio::test]
    async fn adds_citations_from_llm_only_when_no_grounding() {
        let resp = r#"{"findings":[{"id":"c1","source_url":"https://x","snippet":"y"}]}"#;
        let provider = Arc::new(MockProvider::new(resp));
        let ctx = AgentContext::new(provider, "mock");
        let plan = Plan::new("r");
        let out = ResearcherAgent.run(&ctx, plan).await.unwrap();
        assert_eq!(out.citations.len(), 1);
    }

    #[tokio::test]
    async fn merges_grounded_hits_with_llm_findings() {
        let resp = r#"{"findings":[{"id":"c1","source_url":"https://llm","snippet":"y"}]}"#;
        let provider = Arc::new(MockProvider::new(resp));
        let grounder = Arc::new(MockGrounder::new(vec![
            GroundingHit {
                title: "SQLite".into(),
                url: "https://sqlite.org".into(),
                snippet: "embedded".into(),
                score: None,
            },
            GroundingHit {
                title: "Tauri".into(),
                url: "https://tauri.app".into(),
                snippet: "desktop".into(),
                score: None,
            },
        ]));
        let ctx = AgentContext::new(provider, "mock").with_grounder(grounder.clone());
        let plan = Plan::new("r");
        let out = ResearcherAgent.run(&ctx, plan).await.unwrap();
        assert_eq!(out.citations.len(), 3, "2 grounded + 1 llm");
        assert_eq!(grounder.call_count(), 1);
        assert!(out.citations.iter().any(
            |c| matches!(&c.source, CitationKind::Web { url } if url == "https://sqlite.org")
        ));
    }

    #[tokio::test]
    async fn drops_llm_findings_with_empty_url() {
        let resp = r#"{"findings":[{"id":"c1","source_url":"","snippet":"y"}]}"#;
        let provider = Arc::new(MockProvider::new(resp));
        let ctx = AgentContext::new(provider, "mock");
        let plan = Plan::new("r");
        let out = ResearcherAgent.run(&ctx, plan).await.unwrap();
        assert_eq!(out.citations.len(), 0);
    }
}
