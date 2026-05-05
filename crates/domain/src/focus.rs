use serde::{Deserialize, Serialize};

use crate::error::DomainError;
use crate::timer::FocusTimer;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaskText(String);

impl TaskText {
    pub fn new(text: impl Into<String>) -> Result<Self, DomainError> {
        let text = text.into();
        if text.trim().is_empty() {
            return Err(DomainError::EmptyTaskText);
        }
        Ok(Self(text))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct FocusId(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Task {
    pub id: String,
    pub text: String,
    #[serde(default)]
    pub done: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Focus {
    pub id: FocusId,
    pub title: String,
    pub description: String,
    pub created_at: String,
    pub tasks: Vec<Task>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub timer: Option<FocusTimer>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn task_text_new_preserves_input() {
        let t = TaskText::new("ship it").unwrap();
        assert_eq!(t.as_str(), "ship it");
    }

    #[test]
    fn task_text_new_rejects_empty() {
        assert_eq!(TaskText::new("").unwrap_err(), DomainError::EmptyTaskText);
    }

    #[test]
    fn task_text_new_rejects_whitespace() {
        assert_eq!(
            TaskText::new("   ").unwrap_err(),
            DomainError::EmptyTaskText
        );
    }

    #[test]
    fn focus_round_trips_via_serde() {
        let f = Focus {
            id: FocusId("abc".into()),
            title: "Customer X bug".into(),
            description: "ship the fix".into(),
            created_at: "2026-04-30T12:00:00Z".into(),
            tasks: vec![Task {
                id: "abc:0".into(),
                text: "step one".into(),
                done: false,
            }],
            timer: None,
        };
        let json = serde_json::to_string(&f).expect("serialize");
        let back: Focus = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(f, back);
    }
}
