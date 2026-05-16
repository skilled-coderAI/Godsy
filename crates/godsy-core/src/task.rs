use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Complexity {
    S,
    M,
    L,
}

impl Complexity {
    pub fn token_budget(self) -> u32 {
        match self {
            Complexity::S => 4_000,
            Complexity::M => 12_000,
            Complexity::L => 32_000,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: String,
    pub order: u32,
    pub goal: String,
    pub component_refs: Vec<String>,
    pub inputs: Vec<String>,
    pub outputs: Vec<String>,
    pub files_touched: Vec<String>,
    pub acceptance_criteria: Vec<String>,
    pub complexity: Complexity,
    pub depends_on: Vec<String>,
}

impl Task {
    pub fn fits_single_agent_turn(&self) -> bool {
        !matches!(self.complexity, Complexity::L)
    }
}
