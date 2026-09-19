use serde::{Deserialize, Serialize};

/// Whether a session is mid-turn. A session is working from the moment a prompt is
/// submitted until the turn stops — there is no clock in this, and no expiry.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "export-ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "export-ts", ts(export))]
pub enum SessionActivity {
    Working,
    Idle,
}
