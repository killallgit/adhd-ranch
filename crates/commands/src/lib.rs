use std::sync::Arc;

use adhd_ranch_domain::Settings;
use adhd_ranch_storage::{DecisionLog, FocusStore, ProposalQueue};

pub mod applier;
pub mod error;
pub mod focus;
pub mod proposal;

pub use applier::{
    AddTaskApplier, AppliedOutcome, DiscardApplier, NewFocusApplier, ProposalApplier,
    ProposalDispatcher,
};
pub use error::CommandError;
pub use focus::{CreateFocusInput, CreatedFocus};
pub use proposal::{CreateProposalInput, CreatedProposal, DecisionOutcome, ProposalEdit};

pub type Clock = Arc<dyn Fn() -> String + Send + Sync>;
pub type IdGen = Arc<dyn Fn() -> String + Send + Sync>;

pub struct Commands {
    pub(crate) store: Arc<dyn FocusStore>,
    pub(crate) queue: Arc<dyn ProposalQueue>,
    pub(crate) decisions: Arc<dyn DecisionLog>,
    pub(crate) dispatcher: Arc<ProposalDispatcher>,
    pub(crate) clock: Clock,
    pub(crate) id_gen: IdGen,
    pub(crate) settings: Settings,
}

impl Commands {
    pub fn new(
        store: Arc<dyn FocusStore>,
        queue: Arc<dyn ProposalQueue>,
        decisions: Arc<dyn DecisionLog>,
        dispatcher: Arc<ProposalDispatcher>,
        clock: Clock,
        id_gen: IdGen,
        settings: Settings,
    ) -> Self {
        Self {
            store,
            queue,
            decisions,
            dispatcher,
            clock,
            id_gen,
            settings,
        }
    }

    pub fn settings(&self) -> Settings {
        self.settings
    }
}
