use serde::{Deserialize, Serialize};

use crate::error::DomainError;
use crate::timer::TimerPreset;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProposalId(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NewFocus {
    pub title: String,
    pub description: String,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub timer_preset: Option<TimerPreset>,
}

impl NewFocus {
    pub fn new(
        title: impl Into<String>,
        description: impl Into<String>,
    ) -> Result<Self, DomainError> {
        let title = title.into();
        if title.trim().is_empty() {
            return Err(DomainError::EmptyTitle);
        }
        Ok(Self {
            title,
            description: description.into(),
            timer_preset: None,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ProposalKind {
    AddTask {
        target_focus_id: String,
        task_text: String,
    },
    NewFocus {
        new_focus: NewFocus,
    },
    Discard,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Proposal {
    pub id: ProposalId,
    #[serde(flatten)]
    pub kind: ProposalKind,
    pub summary: String,
    pub reasoning: String,
    pub created_at: String,
}

#[derive(Debug, PartialEq, Eq)]
pub enum ProposalValidationError {
    EmptySummary,
    EmptyReasoning,
    AddTaskMissingFocus,
    AddTaskEmptyText,
    NewFocusEmptyTitle,
}

impl std::fmt::Display for ProposalValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EmptySummary => f.write_str("summary must not be empty"),
            Self::EmptyReasoning => f.write_str("reasoning must not be empty"),
            Self::AddTaskMissingFocus => f.write_str("add_task requires target_focus_id"),
            Self::AddTaskEmptyText => f.write_str("add_task requires task_text"),
            Self::NewFocusEmptyTitle => f.write_str("new_focus.title must not be empty"),
        }
    }
}

impl std::error::Error for ProposalValidationError {}

impl Proposal {
    pub fn validate(&self) -> Result<(), ProposalValidationError> {
        if self.summary.trim().is_empty() {
            return Err(ProposalValidationError::EmptySummary);
        }
        if self.reasoning.trim().is_empty() {
            return Err(ProposalValidationError::EmptyReasoning);
        }
        match &self.kind {
            ProposalKind::AddTask {
                target_focus_id,
                task_text,
            } => {
                if target_focus_id.trim().is_empty() {
                    return Err(ProposalValidationError::AddTaskMissingFocus);
                }
                if task_text.trim().is_empty() {
                    return Err(ProposalValidationError::AddTaskEmptyText);
                }
            }
            ProposalKind::NewFocus { new_focus } => {
                if new_focus.title.trim().is_empty() {
                    return Err(ProposalValidationError::NewFocusEmptyTitle);
                }
            }
            ProposalKind::Discard => {}
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ok_proposal(kind: ProposalKind) -> Proposal {
        Proposal {
            id: ProposalId("p1".into()),
            kind,
            summary: "did a thing".into(),
            reasoning: "fits because".into(),
            created_at: "2026-04-30T12:00:00Z".into(),
        }
    }

    #[test]
    fn validates_add_task() {
        let p = ok_proposal(ProposalKind::AddTask {
            target_focus_id: "abc".into(),
            task_text: "ship it".into(),
        });
        assert!(p.validate().is_ok());
    }

    #[test]
    fn add_task_requires_focus_id() {
        let p = ok_proposal(ProposalKind::AddTask {
            target_focus_id: "".into(),
            task_text: "x".into(),
        });
        assert_eq!(
            p.validate().unwrap_err(),
            ProposalValidationError::AddTaskMissingFocus
        );
    }

    #[test]
    fn add_task_requires_text() {
        let p = ok_proposal(ProposalKind::AddTask {
            target_focus_id: "abc".into(),
            task_text: "  ".into(),
        });
        assert_eq!(
            p.validate().unwrap_err(),
            ProposalValidationError::AddTaskEmptyText
        );
    }

    #[test]
    fn new_focus_requires_title() {
        let p = ok_proposal(ProposalKind::NewFocus {
            new_focus: NewFocus {
                title: "".into(),
                description: "x".into(),
                timer_preset: None,
            },
        });
        assert_eq!(
            p.validate().unwrap_err(),
            ProposalValidationError::NewFocusEmptyTitle
        );
    }

    #[test]
    fn discard_only_needs_summary_and_reasoning() {
        let p = ok_proposal(ProposalKind::Discard);
        assert!(p.validate().is_ok());
    }

    #[test]
    fn empty_summary_rejected() {
        let mut p = ok_proposal(ProposalKind::Discard);
        p.summary = "  ".into();
        assert_eq!(
            p.validate().unwrap_err(),
            ProposalValidationError::EmptySummary
        );
    }

    #[test]
    fn new_focus_new_accepts_valid_title() {
        let nf = NewFocus::new("Ship it", "the bug").unwrap();
        assert_eq!(nf.title, "Ship it");
        assert_eq!(nf.description, "the bug");
        assert_eq!(nf.timer_preset, None);
    }

    #[test]
    fn new_focus_new_rejects_empty_title() {
        assert_eq!(
            NewFocus::new("", "desc").unwrap_err(),
            DomainError::EmptyTitle
        );
    }

    #[test]
    fn new_focus_new_rejects_whitespace_title() {
        assert_eq!(
            NewFocus::new("   ", "desc").unwrap_err(),
            DomainError::EmptyTitle
        );
    }

    #[test]
    fn add_task_kind_serializes_with_tag() {
        let p = ok_proposal(ProposalKind::AddTask {
            target_focus_id: "abc".into(),
            task_text: "x".into(),
        });
        let json = serde_json::to_value(&p).unwrap();
        assert_eq!(json["kind"], "add_task");
        assert_eq!(json["target_focus_id"], "abc");
        assert_eq!(json["task_text"], "x");
    }

    #[test]
    fn round_trip_preserves_kind() {
        let original = ok_proposal(ProposalKind::NewFocus {
            new_focus: NewFocus {
                title: "T".into(),
                description: "D".into(),
                timer_preset: None,
            },
        });
        let json = serde_json::to_string(&original).unwrap();
        let back: Proposal = serde_json::from_str(&json).unwrap();
        assert_eq!(back, original);
    }
}
