use std::sync::Arc;

use adhd_ranch_domain::Settings;
use adhd_ranch_storage::{DecisionLog, FocusStore, ProposalQueue};

pub mod caps;
pub mod error;
pub mod focus;
pub mod lifecycle;
pub mod proposal;
pub mod timer_expiry;

pub use caps::{CapEvaluator, CapNotifier};
pub use error::CommandError;
pub use focus::{CreateFocusInput, CreatedFocus};
pub use lifecycle::ProposalLifecycle;
pub use proposal::{CreateProposalInput, CreatedProposal, DecisionOutcome, ProposalEdit};
pub use timer_expiry::{
    NotificationRequest, NotificationSink, TimerExpiryEvent, TimerExpiryWorkflow,
};

pub type Clock = Arc<dyn Fn() -> String + Send + Sync>;
pub type ClockSecs = Arc<dyn Fn() -> i64 + Send + Sync>;
pub type IdGen = Arc<dyn Fn() -> String + Send + Sync>;

pub trait SettingsReader: Send + Sync {
    fn get(&self) -> Settings;
}

impl<F> SettingsReader for F
where
    F: Fn() -> Settings + Send + Sync,
{
    fn get(&self) -> Settings {
        self()
    }
}

pub type SettingsProvider = Arc<dyn SettingsReader>;

pub struct Commands {
    pub(crate) store: Arc<dyn FocusStore>,
    pub(crate) queue: Arc<dyn ProposalQueue>,
    pub(crate) lifecycle: Arc<ProposalLifecycle>,
    pub(crate) clock: Clock,
    pub(crate) clock_secs: ClockSecs,
    pub(crate) id_gen: IdGen,
    pub(crate) settings: SettingsProvider,
}

impl Commands {
    pub fn new(
        store: Arc<dyn FocusStore>,
        queue: Arc<dyn ProposalQueue>,
        decisions: Arc<dyn DecisionLog>,
        clock: Clock,
        clock_secs: ClockSecs,
        id_gen: IdGen,
        settings: Settings,
    ) -> Self {
        Self::new_with_settings_provider(
            store,
            queue,
            decisions,
            clock,
            clock_secs,
            id_gen,
            Arc::new(move || settings.clone()),
        )
    }

    pub fn new_with_settings_provider(
        store: Arc<dyn FocusStore>,
        queue: Arc<dyn ProposalQueue>,
        decisions: Arc<dyn DecisionLog>,
        clock: Clock,
        clock_secs: ClockSecs,
        id_gen: IdGen,
        settings: SettingsProvider,
    ) -> Self {
        let lifecycle = Arc::new(ProposalLifecycle::new(
            store.clone(),
            queue.clone(),
            decisions,
            clock.clone(),
            clock_secs.clone(),
            id_gen.clone(),
        ));
        Self {
            store,
            queue,
            lifecycle,
            clock,
            clock_secs,
            id_gen,
            settings,
        }
    }

    pub fn settings(&self) -> Settings {
        self.settings.get()
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use adhd_ranch_domain::{Caps, Settings};
    use adhd_ranch_storage::{JsonlDecisionLog, JsonlProposalQueue, MarkdownFocusStore};
    use tempfile::TempDir;

    use super::*;

    #[test]
    fn caps_reads_latest_settings_provider_value() {
        let dir = TempDir::new().unwrap();
        let settings = Arc::new(Mutex::new(Settings::default()));
        let commands = Commands::new_with_settings_provider(
            Arc::new(MarkdownFocusStore::new(dir.path().join("focuses"))),
            Arc::new(JsonlProposalQueue::new(dir.path().join("proposals.jsonl"))),
            Arc::new(JsonlDecisionLog::new(dir.path().join("decisions.jsonl"))),
            Arc::new(|| "2026-01-01T00:00:00Z".to_string()),
            Arc::new(|| 1_700_000_000),
            Arc::new(|| "id-fixed".to_string()),
            {
                let settings = settings.clone();
                Arc::new(move || settings.lock().unwrap().clone())
            },
        );

        assert_eq!(commands.caps().max_focuses, 5);

        settings.lock().unwrap().caps = Caps {
            max_focuses: 9,
            max_tasks_per_focus: 11,
        };

        assert_eq!(commands.caps().max_focuses, 9);
        assert_eq!(commands.caps().max_tasks_per_focus, 11);
    }
}
