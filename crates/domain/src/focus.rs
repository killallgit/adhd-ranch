use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct FocusId(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Task {
    pub id: String,
    pub text: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Focus {
    pub id: FocusId,
    pub title: String,
    pub description: String,
    pub created_at: String,
    pub tasks: Vec<Task>,
}

#[cfg(test)]
mod tests {
    use super::*;

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
            }],
        };
        let json = serde_json::to_string(&f).expect("serialize");
        let back: Focus = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(f, back);
    }
}
