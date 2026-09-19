//! What the ranch asks a coding agent to tell it about itself.
//!
//! Every agent has some way to run a command when something happens; the event
//! names differ and the config shapes differ, but what the ranch wants out of them
//! does not. [`HookAction`] is that shared want, and each agent's module maps its own
//! events onto it.

use serde_json::Value;

/// What one hook firing tells the ranch.
///
/// Only the session lifecycle is wired up, because the ranch only draws a session as
/// running or resting. Richer events — which tool, which subagent, which model — get
/// their own variants here when something actually consumes them; an agent's event
/// table then gains a row rather than a rewrite.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
