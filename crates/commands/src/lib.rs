use std::sync::Arc;

use adhd_ranch_domain::Settings;
use adhd_ranch_storage::{DecisionLog, FocusStore, ProposalQueue};

pub mod caps;
pub mod error;
pub mod focus;
pub mod lifecycle;
pub mod proposal;

pub use caps::{CapEvaluator, CapNotifier};
pub use error::CommandError;
pub use focus::{CreateFocusInput, CreatedFocus};
pub use lifecycle::ProposalLifecycle;
pub use proposal::{CreateProposalInput, CreatedProposal, DecisionOutcome, ProposalEdit};

pub type Clock = Arc<dyn Fn() -> String + Send + Sync>;
pub type ClockSecs = Arc<dyn Fn() -> i64 + Send + Sync>;
pub type IdGen = Arc<dyn Fn() -> String + Send + Sync>;

pub struct Commands {
    pub(crate) store: Arc<dyn FocusStore>,
    pub(crate) queue: Arc<dyn ProposalQueue>,
    pub(crate) lifecycle: Arc<ProposalLifecycle>,
    pub(crate) clock: Clock,
    pub(crate) clock_secs: ClockSecs,
    pub(crate) id_gen: IdGen,
    pub(crate) settings: Settings,
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
        let lifecycle = Arc::new(ProposalLifecycle::new(
            store.clone(),
            queue.clone(),
            decisions,
            clock.clone(),
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
        self.settings.clone()
    }
}
