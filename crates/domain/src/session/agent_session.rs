use serde::{Deserialize, Serialize};

use super::activity::SessionActivity;
use super::pen::Pen;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "export-ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "export-ts", ts(export))]
pub struct AgentSession {
    pub id: String,
    pub name: String,
    pub pen: Pen,
    pub activity: SessionActivity,
}
