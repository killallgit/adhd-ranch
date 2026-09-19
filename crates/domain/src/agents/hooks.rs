//! What the ranch asks a coding agent to tell it about itself.
//!
//! Every agent has some way to run a command when something happens; the event
//! names differ and the config shapes differ, but what the ranch wants out of them
//! does not. [`HookAction`] is that shared want, and each agent's module maps its own
//! events onto it.

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// What one hook firing tells the ranch.
///
/// Only the session lifecycle is wired up, because the ranch only draws a session as
/// running or resting. Richer events — which tool, which subagent, which model — get
/// their own variants here when something actually consumes them; an agent's event
/// table then gains a row rather than a rewrite.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "export-ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "export-ts", ts(export))]
pub enum HookAction {
    /// The session exists and has not started working yet.
    Start,
    /// The session began a turn.
    Working,
    /// The turn ended; the session is waiting on its human.
    Idle,
    /// The session is over.
    End,
}

impl HookAction {
    /// Read back a word the client sent. Unknown words are not ours to guess at.
    pub fn from_verb(verb: &str) -> Option<Self> {
        [Self::Start, Self::Working, Self::Idle, Self::End]
            .into_iter()
            .find(|action| action.verb() == verb)
    }

    /// The word the installed client is invoked with.
    pub const fn verb(self) -> &'static str {
        match self {
            Self::Start => "start",
            Self::Working => "working",
            Self::Idle => "idle",
            Self::End => "end",
        }
    }
}

/// What changing an agent's hook configuration would produce.
#[derive(Debug, Clone, PartialEq)]
pub enum HookEdit {
    /// The file already says what we want, so there is nothing to write.
    Unchanged,
    Updated(Value),
    /// Not a shape we recognise. The file belongs to the agent and to whatever else
    /// the user has pointed at it, so an edit we cannot reason about is not made.
    Unsupported,
}

/// One firing as the debug window shows it: what an agent said, and what the ranch
/// made of it.
///
/// Deliberately flat strings rather than a parsed session. The point of showing a
/// firing is to see what actually arrived, including the payloads the ranch could
/// make nothing of, and those have no session to speak of.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "export-ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "export-ts", ts(export))]
pub struct HookFiring {
    pub at: String,
    pub action: HookAction,
    /// `None` when the payload was not one the ranch could read.
    pub session_id: Option<String>,
    pub pen: Option<String>,
    /// Whether the ranch looks any different for having heard this.
    pub changed: bool,
}

/// Where the ranch and an agent are supposed to meet.
///
/// Every one of these is settled once, at startup, and a mismatch in any of them is
/// silent by design — the client says nothing when it cannot deliver. Showing them
/// turns "nothing is happening" into a question with an answer.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "export-ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "export-ts", ts(export))]
pub struct HookWiring {
    pub agent: String,
    pub settings_file: String,
    pub client_bin: String,
    pub socket_path: String,
    /// Whether the agent's settings currently name this exact client and socket.
    pub installed: bool,
    /// Whether the ranch is drawing agents at all.
    pub enabled: bool,
}
