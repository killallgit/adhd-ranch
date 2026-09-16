use std::sync::Arc;

use adhd_ranch_domain::AgentSession;
use adhd_ranch_storage::AgentSessionStore;

use crate::SettingsProvider;

pub struct AgentSessions {
    store: Arc<dyn AgentSessionStore>,
    settings: SettingsProvider,
}

impl AgentSessions {
    pub fn new(store: Arc<dyn AgentSessionStore>, settings: SettingsProvider) -> Self {
        Self { store, settings }
    }

    pub fn list(&self) -> Vec<AgentSession> {
        if !self.settings.get().agents.enabled {
            return Vec::new();
        }
        self.store.list()
    }
}

#[cfg(test)]
mod tests {
    use adhd_ranch_domain::{AgentsConfig, Settings};

    use super::*;

    struct FixedStore(Vec<AgentSession>);

    impl AgentSessionStore for FixedStore {
        fn list(&self) -> Vec<AgentSession> {
            self.0.clone()
        }
    }

    fn sessions_with(enabled: bool) -> AgentSessions {
        let store = FixedStore(vec![AgentSession {
            id: "session-1".into(),
            name: "adhd-ranch".into(),
        }]);
        let settings = Settings {
            agents: AgentsConfig { enabled },
            ..Settings::default()
        };
        AgentSessions::new(Arc::new(store), Arc::new(move || settings.clone()))
    }

    #[test]
    fn lists_sessions_when_agents_are_enabled() {
        assert_eq!(sessions_with(true).list().len(), 1);
    }

    #[test]
    fn lists_no_sessions_when_agents_are_disabled() {
        assert!(sessions_with(false).list().is_empty());
    }
}
