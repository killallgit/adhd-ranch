use std::sync::Arc;

use adhd_ranch_domain::{
    Decision, DecisionKind, FocusTimer, Proposal, ProposalId, ProposalKind, TimerStatus,
};
use adhd_ranch_storage::{DecisionLog, FocusStore, ProposalQueue};

use crate::error::CommandError;
use crate::focus::create_focus_in_store;
use crate::proposal::{DecisionOutcome, ProposalEdit};
use crate::{Clock, ClockSecs, IdGen};

pub struct ProposalLifecycle {
    store: Arc<dyn FocusStore>,
    queue: Arc<dyn ProposalQueue>,
    decisions: Arc<dyn DecisionLog>,
    clock: Clock,
    clock_secs: ClockSecs,
    id_gen: IdGen,
}

impl ProposalLifecycle {
    pub fn new(
        store: Arc<dyn FocusStore>,
        queue: Arc<dyn ProposalQueue>,
        decisions: Arc<dyn DecisionLog>,
        clock: Clock,
        clock_secs: ClockSecs,
        id_gen: IdGen,
    ) -> Self {
        Self {
            store,
            queue,
            decisions,
            clock,
            clock_secs,
            id_gen,
        }
    }

    pub fn accept(&self, id: &str, edit: ProposalEdit) -> Result<DecisionOutcome, CommandError> {
        let original = self.load(id)?;
        let (proposal, edited) = apply_edit(original, &edit);
        proposal.validate()?;
        let target = self.apply(&proposal)?;
        self.record_decision(&proposal, DecisionKind::Accept, target.clone(), edited)?;
        self.queue.remove(&proposal.id)?;
        Ok(DecisionOutcome {
            id: id.to_string(),
            target,
        })
    }

    pub fn reject(&self, id: &str) -> Result<DecisionOutcome, CommandError> {
        let proposal = self.load(id)?;
        self.record_decision(&proposal, DecisionKind::Reject, None, false)?;
        self.queue.remove(&proposal.id)?;
        Ok(DecisionOutcome {
            id: id.to_string(),
            target: None,
        })
    }

    fn apply(&self, proposal: &Proposal) -> Result<Option<String>, CommandError> {
        match &proposal.kind {
            ProposalKind::AddTask {
                target_focus_id,
                task_text,
            } => {
                self.store.append_task(target_focus_id, task_text)?;
                Ok(Some(target_focus_id.clone()))
            }
            ProposalKind::NewFocus { new_focus } => {
                let timer = new_focus.timer_preset().map(|preset| FocusTimer {
                    duration_secs: preset.duration_secs(),
                    started_at: (self.clock_secs)(),
                    status: TimerStatus::Running,
                });
                let slug = create_focus_in_store(
                    &self.store,
                    &self.clock,
                    &self.id_gen,
                    new_focus,
                    timer,
                )?;
                Ok(Some(slug))
            }
            ProposalKind::Discard => Ok(None),
        }
    }

    fn load(&self, id: &str) -> Result<Proposal, CommandError> {
        self.queue
            .find(&ProposalId(id.to_string()))?
            .ok_or_else(|| CommandError::NotFound(format!("proposal not found: {id}")))
    }

    fn record_decision(
        &self,
        proposal: &Proposal,
        kind: DecisionKind,
        target: Option<String>,
        edited: bool,
    ) -> Result<(), CommandError> {
        let decision = Decision {
            ts: (self.clock)(),
            proposal_id: proposal.id.0.clone(),
            decision: kind,
            reasoning: proposal.reasoning.clone(),
            target,
            edited,
        };
        self.decisions.append(&decision)?;
        Ok(())
    }
}

fn apply_edit(mut proposal: Proposal, edit: &ProposalEdit) -> (Proposal, bool) {
    let mut edited = false;
    match &mut proposal.kind {
        ProposalKind::AddTask {
            target_focus_id,
            task_text,
        } => {
            if let Some(new_id) = edit.target_focus_id.as_ref() {
                if new_id != target_focus_id {
                    *target_focus_id = new_id.clone();
                    edited = true;
                }
            }
            if let Some(new_text) = edit.task_text.as_ref() {
                if new_text != task_text {
                    *task_text = new_text.clone();
                    edited = true;
                }
            }
        }
        ProposalKind::NewFocus { new_focus } => {
            if let Some(replacement) = edit.new_focus.as_ref() {
                if replacement != new_focus {
                    *new_focus = replacement.clone();
                    edited = true;
                }
            }
        }
        ProposalKind::Discard => {}
    }
    (proposal, edited)
}

#[cfg(test)]
mod tests {
    use super::*;
    use adhd_ranch_domain::{NewFocus, ProposalId, ProposalKind};
    use adhd_ranch_storage::{JsonlDecisionLog, JsonlProposalQueue, MarkdownFocusStore};
    use std::fs;
    use tempfile::TempDir;

    struct Harness {
        _dir: TempDir,
        focuses_root: std::path::PathBuf,
        lifecycle: ProposalLifecycle,
        queue: Arc<dyn ProposalQueue>,
        decisions_path: std::path::PathBuf,
    }

    fn build_lifecycle() -> Harness {
        let dir = TempDir::new().unwrap();
        let focuses_root = dir.path().join("focuses");
        fs::create_dir_all(&focuses_root).unwrap();
        let proposals_path = dir.path().join("proposals.jsonl");
        let decisions_path = dir.path().join("decisions.jsonl");

        let store: Arc<dyn FocusStore> = Arc::new(MarkdownFocusStore::new(focuses_root.clone()));
        let queue: Arc<dyn ProposalQueue> = Arc::new(JsonlProposalQueue::new(proposals_path));
        let decisions: Arc<dyn DecisionLog> =
            Arc::new(JsonlDecisionLog::new(decisions_path.clone()));
        let clock: Clock = Arc::new(|| "2026-04-30T12:00:00Z".to_string());
        let clock_secs: crate::ClockSecs = Arc::new(|| 1_700_000_000);
        let id_gen: IdGen = Arc::new(|| "id-fixed".to_string());

        let lifecycle =
            ProposalLifecycle::new(store, queue.clone(), decisions, clock, clock_secs, id_gen);
        Harness {
            _dir: dir,
            focuses_root,
            lifecycle,
            queue,
            decisions_path,
        }
    }

    fn write_focus(root: &std::path::Path, slug: &str, body: &str) {
        let dir = root.join(slug);
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("focus.md"), body).unwrap();
    }

    fn focus_md(id: &str) -> String {
        format!("---\nid: {id}\ntitle: A\ndescription:\ncreated_at: 2026-04-30T12:00:00Z\n---\n")
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
    fn accept_add_task_appends_bullet_records_decision_and_clears_queue() {
        let h = build_lifecycle();
        write_focus(&h.focuses_root, "f1", &focus_md("f1"));
        h.queue
            .append(&proposal(
                "p1",
                ProposalKind::AddTask {
                    target_focus_id: "f1".into(),
                    task_text: "ship it".into(),
                },
            ))
            .unwrap();

        let out = h.lifecycle.accept("p1", ProposalEdit::default()).unwrap();
        assert_eq!(out.target.as_deref(), Some("f1"));

        let body = fs::read_to_string(h.focuses_root.join("f1/focus.md")).unwrap();
        assert!(body.contains("- [ ] ship it"));
        assert!(h.queue.list().unwrap().is_empty());

        let log = fs::read_to_string(&h.decisions_path).unwrap();
        assert!(log.contains("\"decision\":\"accept\""));
        assert!(log.contains("\"proposal_id\":\"p1\""));
    }

    #[test]
    fn accept_new_focus_creates_dir_and_returns_slug() {
        let h = build_lifecycle();
        h.queue
            .append(&proposal(
                "p1",
                ProposalKind::NewFocus {
                    new_focus: NewFocus::new("Customer X bug", "ship").unwrap(),
                },
            ))
            .unwrap();

        let out = h.lifecycle.accept("p1", ProposalEdit::default()).unwrap();
        assert_eq!(out.target.as_deref(), Some("customer-x-bug"));
        assert!(h.focuses_root.join("customer-x-bug/focus.md").exists());
    }

    #[test]
    fn accept_discard_records_decision_with_no_target() {
        let h = build_lifecycle();
        h.queue
            .append(&proposal("p1", ProposalKind::Discard))
            .unwrap();
        let out = h.lifecycle.accept("p1", ProposalEdit::default()).unwrap();
        assert_eq!(out.target, None);
        assert!(h.queue.list().unwrap().is_empty());
    }

    #[test]
    fn accept_with_edit_uses_overrides_and_marks_edited() {
        let h = build_lifecycle();
        write_focus(&h.focuses_root, "f1", &focus_md("f1"));
        write_focus(&h.focuses_root, "f2", &focus_md("f2"));
        h.queue
            .append(&proposal(
                "p1",
                ProposalKind::AddTask {
                    target_focus_id: "f1".into(),
                    task_text: "old".into(),
                },
            ))
            .unwrap();

        let edit = ProposalEdit {
            target_focus_id: Some("f2".into()),
            task_text: Some("new".into()),
            new_focus: None,
        };
        let out = h.lifecycle.accept("p1", edit).unwrap();
        assert_eq!(out.target.as_deref(), Some("f2"));

        let body = fs::read_to_string(h.focuses_root.join("f2/focus.md")).unwrap();
        assert!(body.contains("- [ ] new"));

        let log = fs::read_to_string(&h.decisions_path).unwrap();
        assert!(log.contains("\"edited\":true"));
    }

    #[test]
    fn accept_unknown_id_returns_not_found() {
        let h = build_lifecycle();
        let err = h
            .lifecycle
            .accept("missing", ProposalEdit::default())
            .unwrap_err();
        assert!(matches!(err, CommandError::NotFound(_)));
    }

    #[test]
    fn reject_records_decision_and_clears_queue_without_mutation() {
        let h = build_lifecycle();
        h.queue
            .append(&proposal("p1", ProposalKind::Discard))
            .unwrap();
        let out = h.lifecycle.reject("p1").unwrap();
        assert_eq!(out.target, None);
        assert!(h.queue.list().unwrap().is_empty());

        let log = fs::read_to_string(&h.decisions_path).unwrap();
        assert!(log.contains("\"decision\":\"reject\""));
    }
}
