use std::sync::Arc;

use adhd_ranch_domain::Animal;
use adhd_ranch_storage::AnimalStore;

use crate::SettingsProvider;

pub struct Animals {
    store: Arc<dyn AnimalStore>,
    settings: SettingsProvider,
}

impl Animals {
    pub fn new(store: Arc<dyn AnimalStore>, settings: SettingsProvider) -> Self {
        Self { store, settings }
    }

    pub fn list(&self) -> Vec<Animal> {
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

    struct FixedStore(Vec<Animal>);

    impl AnimalStore for FixedStore {
        fn list(&self) -> Vec<Animal> {
            self.0.clone()
        }
    }

    fn animals_with(enabled: bool) -> Animals {
        let store = FixedStore(vec![Animal {
            id: "session-1".into(),
            name: "adhd-ranch".into(),
        }]);
        let settings = Settings {
            agents: AgentsConfig { enabled },
            ..Settings::default()
        };
        Animals::new(Arc::new(store), Arc::new(move || settings.clone()))
    }

    #[test]
    fn lists_agent_animals_when_agents_are_enabled() {
        assert_eq!(animals_with(true).list().len(), 1);
    }

    #[test]
    fn lists_no_animals_when_agents_are_disabled() {
        assert!(animals_with(false).list().is_empty());
    }
}
