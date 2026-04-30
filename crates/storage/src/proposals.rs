use std::fs::{File, OpenOptions};
use std::io::{self, BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use adhd_ranch_domain::Proposal;

#[derive(Debug)]
pub enum QueueError {
    Io(io::Error),
    Serde(serde_json::Error),
}

impl std::fmt::Display for QueueError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(e) => write!(f, "queue io: {e}"),
            Self::Serde(e) => write!(f, "queue serde: {e}"),
        }
    }
}

impl std::error::Error for QueueError {}

impl From<io::Error> for QueueError {
    fn from(e: io::Error) -> Self {
        Self::Io(e)
    }
}

impl From<serde_json::Error> for QueueError {
    fn from(e: serde_json::Error) -> Self {
        Self::Serde(e)
    }
}

pub trait ProposalQueue: Send + Sync {
    fn append(&self, proposal: &Proposal) -> Result<(), QueueError>;
    fn list(&self) -> Result<Vec<Proposal>, QueueError>;
}

pub struct JsonlProposalQueue {
    path: PathBuf,
    write_lock: Mutex<()>,
}

impl JsonlProposalQueue {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self {
            path: path.into(),
            write_lock: Mutex::new(()),
        }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }
}

impl ProposalQueue for JsonlProposalQueue {
    fn append(&self, proposal: &Proposal) -> Result<(), QueueError> {
        let _guard = self.write_lock.lock().unwrap_or_else(|e| e.into_inner());
        if let Some(parent) = self.path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let mut line = serde_json::to_string(proposal)?;
        line.push('\n');
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)?;
        file.write_all(line.as_bytes())?;
        file.sync_data()?;
        Ok(())
    }

    fn list(&self) -> Result<Vec<Proposal>, QueueError> {
        let file = match File::open(&self.path) {
            Ok(f) => f,
            Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(Vec::new()),
            Err(e) => return Err(e.into()),
        };
        let reader = BufReader::new(file);
        let mut out = Vec::new();
        for line in reader.lines() {
            let line = line?;
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }
            let proposal: Proposal = serde_json::from_str(trimmed)?;
            out.push(proposal);
        }
        Ok(out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use adhd_ranch_domain::{NewFocus, ProposalId, ProposalKind};
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
                    new_focus: NewFocus {
                        title: format!("T{i}"),
                        description: String::new(),
                    },
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
}
