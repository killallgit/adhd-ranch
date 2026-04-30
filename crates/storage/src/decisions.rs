use std::fs::OpenOptions;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use adhd_ranch_domain::Decision;

#[derive(Debug)]
pub enum DecisionLogError {
    Io(io::Error),
    Serde(serde_json::Error),
}

impl std::fmt::Display for DecisionLogError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(e) => write!(f, "decision log io: {e}"),
            Self::Serde(e) => write!(f, "decision log serde: {e}"),
        }
    }
}

impl std::error::Error for DecisionLogError {}

impl From<io::Error> for DecisionLogError {
    fn from(e: io::Error) -> Self {
        Self::Io(e)
    }
}

impl From<serde_json::Error> for DecisionLogError {
    fn from(e: serde_json::Error) -> Self {
        Self::Serde(e)
    }
}

pub trait DecisionLog: Send + Sync {
    fn append(&self, decision: &Decision) -> Result<(), DecisionLogError>;
}

pub struct JsonlDecisionLog {
    path: PathBuf,
    lock: Mutex<()>,
}

impl JsonlDecisionLog {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self {
            path: path.into(),
            lock: Mutex::new(()),
        }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }
}

impl DecisionLog for JsonlDecisionLog {
    fn append(&self, decision: &Decision) -> Result<(), DecisionLogError> {
        let _guard = self.lock.lock().unwrap_or_else(|e| e.into_inner());
        if let Some(parent) = self.path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let mut line = serde_json::to_string(decision)?;
        line.push('\n');
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)?;
        file.write_all(line.as_bytes())?;
        file.sync_data()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use adhd_ranch_domain::DecisionKind;
    use std::fs;
    use tempfile::TempDir;

    fn decision(id: &str, kind: DecisionKind) -> Decision {
        Decision {
            ts: "2026-04-30T12:00:00Z".into(),
            proposal_id: id.into(),
            decision: kind,
            reasoning: "fits".into(),
            target: Some("focus-x".into()),
        }
    }

    #[test]
    fn append_writes_one_line_per_decision_and_round_trips() {
        let dir = TempDir::new().unwrap();
        let log = JsonlDecisionLog::new(dir.path().join("decisions.jsonl"));
        log.append(&decision("a", DecisionKind::Accept)).unwrap();
        log.append(&decision("b", DecisionKind::Reject)).unwrap();

        let contents = fs::read_to_string(log.path()).unwrap();
        let lines: Vec<&str> = contents.lines().collect();
        assert_eq!(lines.len(), 2);
        let first: Decision = serde_json::from_str(lines[0]).unwrap();
        assert_eq!(first.proposal_id, "a");
        assert_eq!(first.decision, DecisionKind::Accept);
    }
}
