use std::sync::Arc;

use adhd_ranch_domain::{Proposal, ProposalKind};
use adhd_ranch_storage::{FocusStore, FocusStoreError};

#[derive(Debug)]
pub enum ApplyError {
    Store(FocusStoreError),
}

impl std::fmt::Display for ApplyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Store(e) => write!(f, "apply: {e}"),
        }
    }
}

impl std::error::Error for ApplyError {}

impl From<FocusStoreError> for ApplyError {
    fn from(e: FocusStoreError) -> Self {
        Self::Store(e)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppliedOutcome {
    pub target: Option<String>,
}

pub trait ProposalApplier: Send + Sync {
    fn apply(&self, proposal: &Proposal) -> Result<AppliedOutcome, ApplyError>;
}

pub struct AddTaskApplier {
    store: Arc<dyn FocusStore>,
}

impl AddTaskApplier {
    pub fn new(store: Arc<dyn FocusStore>) -> Self {
        Self { store }
    }
}

impl ProposalApplier for AddTaskApplier {
    fn apply(&self, proposal: &Proposal) -> Result<AppliedOutcome, ApplyError> {
        match &proposal.kind {
            ProposalKind::AddTask {
                target_focus_id,
                task_text,
            } => {
                self.store.append_task(target_focus_id, task_text)?;
                Ok(AppliedOutcome {
                    target: Some(target_focus_id.clone()),
                })
            }
            _ => unreachable!("AddTaskApplier called with non-add_task kind"),
        }
    }
}

pub struct NewFocusApplier {
    store: Arc<dyn FocusStore>,
    clock: Arc<dyn Fn() -> String + Send + Sync>,
    id_gen: Arc<dyn Fn() -> String + Send + Sync>,
}

impl NewFocusApplier {
    pub fn new(
        store: Arc<dyn FocusStore>,
        clock: Arc<dyn Fn() -> String + Send + Sync>,
        id_gen: Arc<dyn Fn() -> String + Send + Sync>,
    ) -> Self {
        Self {
            store,
            clock,
            id_gen,
        }
    }
}

impl ProposalApplier for NewFocusApplier {
    fn apply(&self, proposal: &Proposal) -> Result<AppliedOutcome, ApplyError> {
        match &proposal.kind {
            ProposalKind::NewFocus { new_focus } => {
                let id = (self.id_gen)();
                let created_at = (self.clock)();
                let slug = self.store.create_focus(new_focus, &id, &created_at)?;
                Ok(AppliedOutcome { target: Some(slug) })
            }
            _ => unreachable!("NewFocusApplier called with non-new_focus kind"),
        }
    }
}

pub struct DiscardApplier;

impl ProposalApplier for DiscardApplier {
    fn apply(&self, _proposal: &Proposal) -> Result<AppliedOutcome, ApplyError> {
        Ok(AppliedOutcome { target: None })
    }
}

pub struct ProposalDispatcher {
    add_task: Arc<dyn ProposalApplier>,
    new_focus: Arc<dyn ProposalApplier>,
    discard: Arc<dyn ProposalApplier>,
}

impl ProposalDispatcher {
    pub fn new(
        add_task: Arc<dyn ProposalApplier>,
        new_focus: Arc<dyn ProposalApplier>,
        discard: Arc<dyn ProposalApplier>,
    ) -> Self {
        Self {
            add_task,
            new_focus,
            discard,
        }
    }

    pub fn from_store(
        store: Arc<dyn FocusStore>,
        clock: Arc<dyn Fn() -> String + Send + Sync>,
        id_gen: Arc<dyn Fn() -> String + Send + Sync>,
    ) -> Self {
        Self::new(
            Arc::new(AddTaskApplier::new(store.clone())),
            Arc::new(NewFocusApplier::new(store, clock, id_gen)),
            Arc::new(DiscardApplier),
        )
    }

    pub fn apply(&self, proposal: &Proposal) -> Result<AppliedOutcome, ApplyError> {
        let strategy = self.strategy_for(&proposal.kind);
        strategy.apply(proposal)
    }

    fn strategy_for(&self, kind: &ProposalKind) -> &Arc<dyn ProposalApplier> {
        match kind {
            ProposalKind::AddTask { .. } => &self.add_task,
            ProposalKind::NewFocus { .. } => &self.new_focus,
            ProposalKind::Discard => &self.discard,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use adhd_ranch_domain::{NewFocus, ProposalId, ProposalKind};
    use adhd_ranch_storage::MarkdownFocusStore;
    use std::fs;
    use tempfile::TempDir;

    fn write_focus(root: &std::path::Path, slug: &str, body: &str) {
        let dir = root.join(slug);
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("focus.md"), body).unwrap();
    }

    fn focus_md(id: &str) -> String {
        format!("---\nid: {id}\ntitle: A\ndescription:\ncreated_at: 2026-04-30T12:00:00Z\n---\n")
    }

    fn dispatcher(focuses_root: &std::path::Path) -> ProposalDispatcher {
        let store: Arc<dyn FocusStore> = Arc::new(MarkdownFocusStore::new(focuses_root));
        ProposalDispatcher::from_store(
            store,
            Arc::new(|| "2026-04-30T12:00:00Z".to_string()),
            Arc::new(|| "id-fixed".to_string()),
        )
    }

    fn proposal(id: &str, kind: ProposalKind) -> Proposal {
        Proposal {
            id: ProposalId(id.into()),
            kind,
            summary: "s".into(),
            reasoning: "r".into(),
            created_at: "2026-04-30T12:00:00Z".into(),
        }
    }

    #[test]
    fn dispatch_add_task_appends_bullet_and_returns_focus_id() {
        let dir = TempDir::new().unwrap();
        write_focus(dir.path(), "f1", &focus_md("f1"));
        let d = dispatcher(dir.path());
        let outcome = d
            .apply(&proposal(
                "p1",
                ProposalKind::AddTask {
                    target_focus_id: "f1".into(),
                    task_text: "ship it".into(),
                },
            ))
            .unwrap();
        assert_eq!(outcome.target.as_deref(), Some("f1"));
        let content = fs::read_to_string(dir.path().join("f1/focus.md")).unwrap();
        assert!(content.contains("- [ ] ship it"));
    }

    #[test]
    fn dispatch_new_focus_creates_dir_and_returns_slug() {
        let dir = TempDir::new().unwrap();
        let d = dispatcher(dir.path());
        let outcome = d
            .apply(&proposal(
                "p1",
                ProposalKind::NewFocus {
                    new_focus: NewFocus {
                        title: "Customer X bug".into(),
                        description: "ship".into(),
                    },
                },
            ))
            .unwrap();
        assert_eq!(outcome.target.as_deref(), Some("customer-x-bug"));
        assert!(dir.path().join("customer-x-bug/focus.md").exists());
    }

    #[test]
    fn dispatch_discard_is_noop_with_no_target() {
        let dir = TempDir::new().unwrap();
        let d = dispatcher(dir.path());
        let outcome = d.apply(&proposal("p1", ProposalKind::Discard)).unwrap();
        assert_eq!(outcome.target, None);
    }
}
