use std::path::{Path, PathBuf};

use adhd_ranch_domain::{Proposal, ProposalId};

use crate::jsonl::{JsonlError, JsonlLog};

pub type QueueError = JsonlError;

pub trait ProposalQueue: Send + Sync {
    fn append(&self, proposal: &Proposal) -> Result<(), QueueError>;
    fn list(&self) -> Result<Vec<Proposal>, QueueError>;
    fn find(&self, id: &ProposalId) -> Result<Option<Proposal>, QueueError>;
    fn remove(&self, id: &ProposalId) -> Result<bool, QueueError>;
}

pub struct JsonlProposalQueue {
    log: JsonlLog<Proposal>,
}

impl JsonlProposalQueue {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self {
            log: JsonlLog::new(path),
        }
    }

    pub fn path(&self) -> &Path {
        self.log.path()
    }
}

impl ProposalQueue for JsonlProposalQueue {
    fn append(&self, proposal: &Proposal) -> Result<(), QueueError> {
        self.log.append(proposal)
    }

    fn list(&self) -> Result<Vec<Proposal>, QueueError> {
        self.log.read_all()
    }

    fn find(&self, id: &ProposalId) -> Result<Option<Proposal>, QueueError> {
        Ok(self.log.read_all()?.into_iter().find(|p| &p.id == id))
    }

    fn remove(&self, id: &ProposalId) -> Result<bool, QueueError> {
        self.log.modify(|items| {
            let before = items.len();
            items.retain(|p| &p.id != id);
            items.len() != before
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use adhd_ranch_domain::{NewFocus, ProposalKind};
    use tempfile::TempDir;

    fn proposal(id: &str, kind: ProposalKind) -> Proposal {
        Proposal {
            id: ProposalId(id.into()),
            kind,
            summary: "did a thing".into(),
            reasoning: "fits".into(),
            created_at: "2026-04-30T12:00:00Z".into(),
        }
    }

    #[test]
    fn list_on_missing_file_returns_empty() {
        let dir = TempDir::new().unwrap();
        let queue = JsonlProposalQueue::new(dir.path().join("proposals.jsonl"));
        assert!(queue.list().unwrap().is_empty());
    }

    #[test]
    fn append_creates_file_and_persists() {
        let dir = TempDir::new().unwrap();
        let queue = JsonlProposalQueue::new(dir.path().join("nested/proposals.jsonl"));
        let p = proposal(
            "p1",
            ProposalKind::AddTask {
                target_focus_id: "f1".into(),
                task_text: "x".into(),
            },
        );
        queue.append(&p).unwrap();

        let listed = queue.list().unwrap();
        assert_eq!(listed.len(), 1);
        assert_eq!(listed[0], p);
    }

    #[test]
    fn append_preserves_order_across_writes() {
        let dir = TempDir::new().unwrap();
        let queue = JsonlProposalQueue::new(dir.path().join("proposals.jsonl"));
        for i in 0..5 {
            let p = proposal(
                &format!("p{i}"),
                ProposalKind::NewFocus {
                    new_focus: NewFocus::new(format!("T{i}"), "").unwrap(),
                },
            );
            queue.append(&p).unwrap();
        }
        let listed = queue.list().unwrap();
        assert_eq!(listed.len(), 5);
        for (i, p) in listed.iter().enumerate() {
            assert_eq!(p.id, ProposalId(format!("p{i}")));
        }
    }

    #[test]
    fn ignores_blank_lines() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("proposals.jsonl");
        std::fs::write(
            &path,
            "{\"id\":\"a\",\"kind\":\"discard\",\"summary\":\"s\",\"reasoning\":\"r\",\"created_at\":\"t\"}\n\n\n",
        )
        .unwrap();
        let queue = JsonlProposalQueue::new(&path);
        let listed = queue.list().unwrap();
        assert_eq!(listed.len(), 1);
    }

    #[test]
    fn find_returns_proposal_or_none() {
        let dir = TempDir::new().unwrap();
        let queue = JsonlProposalQueue::new(dir.path().join("proposals.jsonl"));
        queue
            .append(&proposal("p1", ProposalKind::Discard))
            .unwrap();
        assert!(queue.find(&ProposalId("p1".into())).unwrap().is_some());
        assert!(queue.find(&ProposalId("missing".into())).unwrap().is_none());
    }

    #[test]
    fn remove_drops_one_line_and_keeps_others() {
        let dir = TempDir::new().unwrap();
        let queue = JsonlProposalQueue::new(dir.path().join("proposals.jsonl"));
        queue
            .append(&proposal("p1", ProposalKind::Discard))
            .unwrap();
        queue
            .append(&proposal("p2", ProposalKind::Discard))
            .unwrap();
        assert!(queue.remove(&ProposalId("p1".into())).unwrap());
        let listed = queue.list().unwrap();
        assert_eq!(listed.len(), 1);
        assert_eq!(listed[0].id, ProposalId("p2".into()));
    }

    #[test]
    fn remove_returns_false_when_id_unknown() {
        let dir = TempDir::new().unwrap();
        let queue = JsonlProposalQueue::new(dir.path().join("proposals.jsonl"));
        queue
            .append(&proposal("p1", ProposalKind::Discard))
            .unwrap();
        assert!(!queue.remove(&ProposalId("nope".into())).unwrap());
    }

    #[test]
    fn remove_last_proposal_deletes_file() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("proposals.jsonl");
        let queue = JsonlProposalQueue::new(&path);
        queue
            .append(&proposal("p1", ProposalKind::Discard))
            .unwrap();
        queue.remove(&ProposalId("p1".into())).unwrap();
        assert!(!path.exists());
    }
}
