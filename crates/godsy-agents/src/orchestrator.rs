use godsy_core::Plan;
use tracing::{info, warn};

use crate::agent::{AgentContext, AgentError, PlanningAgent};
use crate::agents::{
    ApiDesignerAgent, ArchitectAgent, EstimatorAgent, ProductManagerAgent, ResearcherAgent,
    RiskReviewerAgent, TechLeadAgent, UiDesignerAgent, ValidatorAgent,
};

#[derive(Debug)]
pub struct OrchestratorConfig {
    pub max_validator_retries: u32,
}

impl Default for OrchestratorConfig {
    fn default() -> Self {
        Self { max_validator_retries: 1 }
    }
}

#[derive(Debug)]
pub struct Orchestrator {
    cfg: OrchestratorConfig,
}

#[derive(Debug)]
pub struct OrchestratorOutcome {
    pub plan: Plan,
    pub retries: u32,
    pub validator_passed: bool,
}

impl Orchestrator {
    pub fn new(cfg: OrchestratorConfig) -> Self {
        Self { cfg }
    }

    pub async fn run(
        &self,
        ctx: &AgentContext,
        user_request: impl Into<String>,
    ) -> Result<OrchestratorOutcome, AgentError> {
        let mut plan = Plan::new(user_request);

        info!("orchestrator: product_manager");
        plan = ProductManagerAgent.run(ctx, plan).await?;
        info!("orchestrator: researcher");
        plan = ResearcherAgent.run(ctx, plan).await?;

        let mut retries = 0;
        let mut validator_passed = false;

        loop {
            info!("orchestrator: architect (attempt {})", retries + 1);
            plan = ArchitectAgent.run(ctx, plan).await?;
            info!("orchestrator: api_designer");
            plan = ApiDesignerAgent.run(ctx, plan).await?;
            info!("orchestrator: ui_designer");
            plan = UiDesignerAgent.run(ctx, plan).await?;
            info!("orchestrator: tech_lead");
            plan = TechLeadAgent.run(ctx, plan).await?;
            info!("orchestrator: estimator");
            plan = EstimatorAgent.run(ctx, plan).await?;
            info!("orchestrator: risk_reviewer");
            plan = RiskReviewerAgent.run(ctx, plan).await?;
            info!("orchestrator: validator");

            match ValidatorAgent.run(ctx, plan.clone()).await {
                Ok(p) => {
                    plan = p;
                    if plan.confidence.passes() {
                        validator_passed = true;
                        break;
                    }
                    warn!(
                        "validator: weak sections: {:?}",
                        plan.confidence
                            .weak_sections()
                            .iter()
                            .map(|s| &s.section)
                            .collect::<Vec<_>>()
                    );
                }
                Err(e) => {
                    warn!("validator rejected plan: {e}");
                }
            }

            if retries >= self.cfg.max_validator_retries {
                break;
            }
            retries += 1;
        }

        Ok(OrchestratorOutcome { plan, retries, validator_passed })
    }
}
