//! What agents have lately told the ranch.
//!
//! The session store answers "what is running now". This answers "what arrived, and
//! did the ranch make anything of it" — the question worth asking when the answer to
//! the first one is wrong.

use std::collections::VecDeque;
use std::sync::{Arc, RwLock};

use adhd_ranch_domain::agents::claude_code::hooks::{
    agent_session_from_payload, session_id_from_payload,
};
use adhd_ranch_domain::agents::hooks::{HookAction, HookFiring};
use adhd_ranch_domain::session::SessionActivity;

use crate::agent_session_store::HookEventSink;

/// Far enough back to cover several turns across a handful of sessions, which is as
/// much as anybody reads when a hook is misbehaving.
const REMEMBERED: usize = 200;

/// Stamps a firing with the time it arrived. Injected so the journal reads no clock
/// of its own and its tests need no real one.
pub type Clock = Arc<dyn Fn() -> String + Send + Sync>;

/// Reads back what the ranch was told.
pub trait HookHistory: Send + Sync {
    /// Oldest first.
    fn recent(&self) -> Vec<HookFiring>;
}

/// Records every firing on its way to the sink that acts on it.
///
/// A decorator rather than a branch inside the store: what an agent said and what
/// the ranch made of it are two different questions, and only something wrapping
/// both can answer the second.
pub struct HookJournal {
    acting: Arc<dyn HookEventSink>,
    now: Clock,
    seen: RwLock<VecDeque<HookFiring>>,
}

impl HookJournal {
    pub fn new(acting: Arc<dyn HookEventSink>, now: Clock) -> Self {
        Self {
            acting,
            now,
            seen: RwLock::new(VecDeque::new()),
        }
    }
}

impl HookEventSink for HookJournal {
    fn apply(&self, action: HookAction, payload: &str) -> bool {
        let changed = self.acting.apply(action, payload);
        let firing = read((self.now)(), action, payload, changed);

        log::info!(
            "agent hook: {} session={} pen={} changed={changed}",
            action.verb(),
            firing
                .session_id
                .as_deref()
                .unwrap_or("<unreadable payload>"),
            firing.pen.as_deref().unwrap_or("-"),
        );

        if let Ok(mut seen) = self.seen.write() {
            if seen.len() == REMEMBERED {
                seen.pop_front();
            }
            seen.push_back(firing);
        }
        changed
    }
}

impl HookHistory for HookJournal {
    fn recent(&self) -> Vec<HookFiring> {
        let Ok(seen) = self.seen.read() else {
            return Vec::new();
        };
        seen.iter().cloned().collect()
    }
}

/// Reuses the parsers the acting sink uses, so the debug window cannot disagree with
/// the ranch about what a payload said.
///
/// The payload itself is deliberately not kept. Claude Code's `UserPromptSubmit`
/// carries the text the user just typed, and a debug window is exactly the wrong
/// place for it to resurface.
fn read(at: String, action: HookAction, payload: &str, changed: bool) -> HookFiring {
    HookFiring {
        at,
        action,
        // Read on its own, because an `End` never becomes a session and would
        // otherwise be listed without saying which session ended.
        session_id: session_id_from_payload(payload),
        pen: agent_session_from_payload(payload, SessionActivity::Idle).map(|s| s.pen.name),
        changed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Accepting(bool);

    impl HookEventSink for Accepting {
        fn apply(&self, _action: HookAction, _payload: &str) -> bool {
            self.0
        }
    }

    fn journal(changed: bool) -> HookJournal {
        HookJournal::new(
            Arc::new(Accepting(changed)),
            Arc::new(|| "2026-01-01T00:00:00Z".to_string()),
        )
    }

    const PAYLOAD: &str = r#"{"session_id":"abc","cwd":"/code/app"}"#;

    #[test]
    fn a_firing_is_remembered_with_what_it_said() {
        let journal = journal(true);

        journal.apply(HookAction::Working, PAYLOAD);

        let seen = journal.recent();
        assert_eq!(seen.len(), 1);
        assert_eq!(seen[0].action, HookAction::Working);
        assert_eq!(seen[0].session_id.as_deref(), Some("abc"));
        assert_eq!(seen[0].pen.as_deref(), Some("app"));
        assert!(seen[0].changed);
    }

    #[test]
    fn the_verdict_of_the_acting_sink_is_passed_back() {
        assert!(!journal(false).apply(HookAction::Working, PAYLOAD));
    }

    #[test]
    fn a_firing_that_changed_nothing_is_still_remembered() {
        let journal = journal(false);

        journal.apply(HookAction::Working, PAYLOAD);

        assert!(!journal.recent()[0].changed);
    }

    #[test]
    fn a_payload_the_ranch_cannot_read_is_still_remembered() {
        let journal = journal(false);

        journal.apply(HookAction::Start, "not json at all");

        let seen = journal.recent();
        assert_eq!(seen.len(), 1);
        assert_eq!(seen[0].session_id, None);
        assert_eq!(seen[0].pen, None);
    }

    #[test]
    fn an_end_says_which_session_ended() {
        let journal = journal(true);

        journal.apply(HookAction::End, r#"{"session_id":"abc"}"#);

        assert_eq!(journal.recent()[0].session_id.as_deref(), Some("abc"));
    }

    #[test]
    fn the_oldest_firing_is_forgotten_once_full() {
        let journal = journal(true);
        for n in 0..REMEMBERED + 10 {
            journal.apply(
                HookAction::Working,
                &format!(r#"{{"session_id":"{n}","cwd":"/c"}}"#),
            );
        }

        let seen = journal.recent();
        assert_eq!(seen.len(), REMEMBERED);
        assert_eq!(seen[0].session_id.as_deref(), Some("10"));
    }
}
