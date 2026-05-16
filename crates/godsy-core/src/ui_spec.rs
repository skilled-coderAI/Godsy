use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiScreen {
    pub id: String,
    pub name: String,
    pub route: String,
    pub purpose: String,
    #[serde(default)]
    pub component_refs: Vec<String>,
    pub user_flow: Vec<String>,
    #[serde(default)]
    pub api_endpoint_refs: Vec<String>,
    #[serde(default)]
    pub entity_refs: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiComponent {
    pub id: String,
    pub name: String,
    pub kind: String,
    pub responsibility: String,
    #[serde(default)]
    pub props: Vec<UiProp>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiProp {
    pub name: String,
    pub ty: String,
    pub required: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UiSpec {
    pub framework: String,
    pub state_management: String,
    pub design_notes: String,
    pub screens: Vec<UiScreen>,
    pub shared_components: Vec<UiComponent>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum RuleSeverity {
    Info,
    Warn,
    Block,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BusinessRule {
    pub id: String,
    pub name: String,
    pub description: String,
    pub severity: RuleSeverity,
    #[serde(default)]
    pub entity_refs: Vec<String>,
    #[serde(default)]
    pub triggered_by: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowStep {
    pub order: u32,
    pub actor: String,
    pub action: String,
    pub system_response: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workflow {
    pub id: String,
    pub name: String,
    pub trigger: String,
    pub steps: Vec<WorkflowStep>,
    #[serde(default)]
    pub rule_refs: Vec<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BusinessLogicAnalysis {
    pub overview: String,
    pub rules: Vec<BusinessRule>,
    pub workflows: Vec<Workflow>,
}
