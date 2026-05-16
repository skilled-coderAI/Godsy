use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use godsy_core::Plan;
use godsy_grounding::{GroundingProvider, NoopGrounder};
use godsy_llm::LlmProvider;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use time::OffsetDateTime;

#[derive(Debug, Error)]
pub enum AgentError {
    #[error("llm error: {0}")]
    Llm(#[from] godsy_llm::LlmError),
    #[error("grounding error: {0}")]
    Grounding(#[from] godsy_grounding::GroundingError),
    #[error("structured output did not parse as JSON: {0}")]
    Parse(String),
    #[error("agent precondition not met: {0}")]
    Precondition(String),
}

pub type AgentResult<T> = std::result::Result<T, AgentError>;

/// A single agent call captured for `--explain` traces. One event per agent
/// invocation; recorded only when an `ExplainRecorder` is attached to the
/// `AgentContext`. The recorder lives behind an `Arc<Mutex<_>>` so the
/// orchestrator and every agent share the same buffer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentTraceEvent {
    pub agent: String,
    #[serde(with = "time::serde::rfc3339")]
    pub at: OffsetDateTime,
    pub prompt_system: String,
    pub prompt_user: String,
    pub response: String,
    pub model: String,
}

#[derive(Debug, Default)]
pub struct ExplainRecorder {
    events: Mutex<Vec<AgentTraceEvent>>,
}

impl ExplainRecorder {
    pub(crate) fn record(&self, event: AgentTraceEvent) {
        self.events.lock().expect("explain recorder poisoned").push(event);
    }

    pub fn drain(&self) -> Vec<AgentTraceEvent> {
        std::mem::take(&mut *self.events.lock().expect("explain recorder poisoned"))
    }
}

#[derive(Clone, Debug)]
pub struct AgentContext {
    pub provider: Arc<dyn LlmProvider>,
    pub grounder: Arc<dyn GroundingProvider>,
    pub model: String,
    pub explain: Option<Arc<ExplainRecorder>>,
}

impl AgentContext {
    /// Build a context with no grounding gateway and no explain recorder —
    /// the default behaviour used by tests and offline runs.
    pub fn new(provider: Arc<dyn LlmProvider>, model: impl Into<String>) -> Self {
        Self { provider, grounder: Arc::new(NoopGrounder), model: model.into(), explain: None }
    }

    #[must_use]
    pub fn with_grounder(mut self, grounder: Arc<dyn GroundingProvider>) -> Self {
        self.grounder = grounder;
        self
    }

    #[must_use]
    pub fn with_explain(mut self, recorder: Arc<ExplainRecorder>) -> Self {
        self.explain = Some(recorder);
        self
    }
}

/// Every Godsy agent reads the current `Plan`, mutates the section it owns,
/// and returns the updated plan. Agents must not write code; they only revise
/// the plan structure (`prd.md` §9).
#[async_trait]
pub trait PlanningAgent: Send + Sync {
    fn name(&self) -> &'static str;
    async fn run(&self, ctx: &AgentContext, plan: Plan) -> AgentResult<Plan>;
}
