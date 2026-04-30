use std::path::{Path, PathBuf};

use adhd_ranch_domain::Decision;

use crate::jsonl::{JsonlError, JsonlLog};

pub type DecisionLogError = JsonlError;

pub trait DecisionLog: Send + Sync {
    fn append(&self, decision: &Decision) -> Result<(), DecisionLogError>;
}

pub struct JsonlDecisionLog {
    log: JsonlLog<Decision>,
}

impl JsonlDecisionLog {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self {
            log: JsonlLog::new(path),
        }
    }

    pub fn path(&self) -> &Path {
        self.log.path()
    }
}

impl DecisionLog for JsonlDecisionLog {
    fn append(&self, decision: &Decision) -> Result<(), DecisionLogError> {
        self.log.append(decision)
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
            edited: false,
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
