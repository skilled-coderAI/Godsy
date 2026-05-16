use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

use crate::api_spec::ApiSpec;
use crate::citation::Citation;
use crate::confidence::ConfidenceReport;
use crate::task::Task;
use crate::ui_spec::{BusinessLogicAnalysis, UiSpec};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProblemStatement {
    pub summary: String,
    pub success_criteria: Vec<String>,
    pub assumptions: Vec<String>,
    pub clarifications: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Architecture {
    pub overview: String,
    pub components: Vec<Component>,
    pub mermaid_diagram: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Component {
    pub id: String,
    pub name: String,
    pub responsibility: String,
    pub depends_on: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataModel {
    pub entities: Vec<Entity>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entity {
    pub id: String,
    pub name: String,
    pub fields: Vec<Field>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Field {
    pub name: String,
    pub ty: String,
    pub required: bool,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StackDecision {
    pub language: String,
    pub frameworks: Vec<String>,
    pub storage: String,
    pub runtime_target: String,
    pub rationale: String,
    pub citation_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskItem {
    pub id: String,
    pub description: String,
    pub severity: Severity,
    pub mitigation: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Low,
    Medium,
    High,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Plan {
    pub id: String,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
    pub user_request: String,
    pub problem: ProblemStatement,
    pub architecture: Architecture,
    pub data_model: DataModel,
    pub stack: StackDecision,
    pub api: ApiSpec,
    pub ui: UiSpec,
    pub business_logic: BusinessLogicAnalysis,
    pub tasks: Vec<Task>,
    pub risks: Vec<RiskItem>,
    pub citations: Vec<Citation>,
    pub confidence: ConfidenceReport,
}

impl Plan {
    pub fn new(user_request: impl Into<String>) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            created_at: OffsetDateTime::now_utc(),
            user_request: user_request.into(),
            problem: ProblemStatement {
                summary: String::new(),
                success_criteria: vec![],
                assumptions: vec![],
                clarifications: vec![],
            },
            architecture: Architecture {
                overview: String::new(),
                components: vec![],
                mermaid_diagram: String::new(),
            },
            data_model: DataModel { entities: vec![] },
            stack: StackDecision {
                language: String::new(),
                frameworks: vec![],
                storage: String::new(),
                runtime_target: String::new(),
                rationale: String::new(),
                citation_ids: vec![],
            },
            api: ApiSpec::default(),
            ui: UiSpec::default(),
            business_logic: BusinessLogicAnalysis::default(),
            tasks: vec![],
            risks: vec![],
            citations: vec![],
            confidence: ConfidenceReport::default(),
        }
    }
}
