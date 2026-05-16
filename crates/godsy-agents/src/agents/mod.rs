pub mod api_designer;
pub mod architect;
pub mod estimator;
pub mod product_manager;
pub mod researcher;
pub mod risk_reviewer;
pub mod tech_lead;
pub mod ui_designer;
pub mod validator;

pub use api_designer::ApiDesignerAgent;
pub use architect::ArchitectAgent;
pub use estimator::EstimatorAgent;
pub use product_manager::ProductManagerAgent;
pub use researcher::ResearcherAgent;
pub use risk_reviewer::RiskReviewerAgent;
pub use tech_lead::TechLeadAgent;
pub use ui_designer::UiDesignerAgent;
pub use validator::ValidatorAgent;

use godsy_llm::{ChatMessage, ChatRequest, ChatResponse};
use time::OffsetDateTime;

use crate::agent::{AgentContext, AgentError, AgentTraceEvent};

pub(crate) async fn chat_json(
    ctx: &AgentContext,
    agent_name: &str,
    system: &str,
    user: String,
) -> Result<ChatResponse, AgentError> {
    let req = ChatRequest {
        model: ctx.model.clone(),
        messages: vec![ChatMessage::system(system), ChatMessage::user(user.clone())],
        temperature: Some(0.2),
        json_mode: true,
    };
    let resp = ctx.provider.chat(req).await?;
    if let Some(rec) = &ctx.explain {
        rec.record(AgentTraceEvent {
            agent: agent_name.to_string(),
            at: OffsetDateTime::now_utc(),
            prompt_system: system.to_string(),
            prompt_user: user,
            response: resp.content.clone(),
            model: resp.model.clone(),
        });
    }
    Ok(resp)
}

pub(crate) fn parse_json<T: serde::de::DeserializeOwned>(s: &str) -> Result<T, AgentError> {
    serde_json::from_str::<T>(s).map_err(|e| AgentError::Parse(e.to_string()))
}
