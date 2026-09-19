use std::collections::BTreeMap;
use std::sync::RwLock;

use adhd_ranch_domain::agents::claude_code::hooks::{
    agent_session_from_payload, session_id_from_payload,
};
use adhd_ranch_domain::agents::hooks::HookAction;
use adhd_ranch_domain::{AgentSession, SessionActivity};

pub trait AgentSessionStore: Send + Sync {
    fn list(&self) -> Vec<AgentSession>;
}

/// The sessions agents have told the ranch about, as they last described them.
///
/// Held in memory on purpose. A session exists only while the process running it
/// does, so there is nothing here worth surviving a restart — and anything written
/// down would have to be reconciled against reality on the way back up, which is the
/// staleness this whole design exists to avoid.
#[derive(Default)]
pub struct LiveSessions {
    sessions: RwLock<BTreeMap<String, AgentSession>>,
}

impl LiveSessions {
    pub fn new() -> Self {
        Self::default()
    }

    /// Apply one hook firing, reporting whether the ranch now looks any different.
    ///
    /// A session the ranch has never heard of is taken at its word rather than
    /// dropped: hooks installed mid-flight miss the `SessionStart` of everything
    /// already running, and the first thing such a session says is enough to draw it.
    pub fn apply(&self, action: HookAction, payload: &str) -> bool {
        match action {
            HookAction::End => self.forget(payload),
            HookAction::Working => self.record(payload, SessionActivity::Working),
            HookAction::Start | HookAction::Idle => self.record(payload, SessionActivity::Idle),
        }
    }

    fn record(&self, payload: &str, activity: SessionActivity) -> bool {
        let Some(session) = agent_session_from_payload(payload, activity) else {
            return false;
        };
        let Ok(mut held) = self.sessions.write() else {
            return false;
        };
        if held.get(&session.id) == Some(&session) {
            return false;
        }
        held.insert(session.id.clone(), session);
        true
    }

    fn forget(&self, payload: &str) -> bool {
        let Some(id) = session_id_from_payload(payload) else {
            return false;
        };
        let Ok(mut held) = self.sessions.write() else {
            return false;
        };
        held.remove(&id).is_some()
    }
}

impl AgentSessionStore for LiveSessions {
    fn list(&self) -> Vec<AgentSession> {
        let Ok(held) = self.sessions.read() else {
            return Vec::new();
        };
        let mut sessions: Vec<AgentSession> = held.values().cloned().collect();
        // Grouped by pen so the overlay's animals keep a stable order as sessions
        // come and go.
        sessions.sort_by(|a, b| (&a.pen.id, &a.id).cmp(&(&b.pen.id, &b.id)));
        sessions
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const ID: &str = "4af8005a-7a52";

    fn payload(cwd: &str) -> String {
        format!(r#"{{"session_id":"{ID}","cwd":"{cwd}"}}"#)
    }

    #[test]
    fn a_started_session_is_resting() {
        let live = LiveSessions::new();

        assert!(live.apply(HookAction::Start, &payload("/code/app")));

        assert_eq!(live.list()[0].activity, SessionActivity::Idle);
    }

    #[test]
    fn a_working_session_is_working() {
        let live = LiveSessions::new();
        live.apply(HookAction::Start, &payload("/code/app"));

        assert!(live.apply(HookAction::Working, &payload("/code/app")));

        assert_eq!(live.list()[0].activity, SessionActivity::Working);
    }

    #[test]
    fn a_session_the_ranch_never_saw_start_is_still_drawn() {
        let live = LiveSessions::new();

        assert!(live.apply(HookAction::Working, &payload("/code/app")));

        assert_eq!(live.list().len(), 1);
    }

    #[test]
    fn an_ended_session_is_gone() {
        let live = LiveSessions::new();
        live.apply(HookAction::Start, &payload("/code/app"));

        assert!(live.apply(HookAction::End, &payload("/code/app")));

        assert!(live.list().is_empty());
    }

    #[test]
    fn repeating_what_the_ranch_already_knows_changes_nothing() {
        let live = LiveSessions::new();
        live.apply(HookAction::Working, &payload("/code/app"));

        assert!(!live.apply(HookAction::Working, &payload("/code/app")));
    }

    #[test]
    fn ending_a_session_the_ranch_never_knew_changes_nothing() {
        let live = LiveSessions::new();

        assert!(!live.apply(HookAction::End, &payload("/code/app")));
    }

    #[test]
    fn a_payload_that_is_not_ours_is_ignored() {
        let live = LiveSessions::new();

        assert!(!live.apply(HookAction::Start, "not json at all"));
        assert!(live.list().is_empty());
    }

    #[test]
    fn sessions_are_listed_grouped_by_pen() {
        let live = LiveSessions::new();
        live.apply(
            HookAction::Start,
            r#"{"session_id":"z","cwd":"/code/alpha"}"#,
        );
        live.apply(
            HookAction::Start,
            r#"{"session_id":"a","cwd":"/code/zulu"}"#,
        );

        let pens: Vec<_> = live.list().into_iter().map(|s| s.pen.name).collect();
        assert_eq!(pens, vec!["alpha", "zulu"]);
    }
}
