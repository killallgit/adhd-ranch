use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DecisionKind {
    Accept,
    Reject,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Decision {
    pub ts: String,
    pub proposal_id: String,
    pub decision: DecisionKind,
    pub reasoning: String,
    pub target: Option<String>,
    #[serde(default)]
    pub edited: bool,
}
