use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FocusId(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Focus {
    pub id: FocusId,
    pub title: String,
    pub description: String,
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
        };
        let json = serde_json::to_string(&f).expect("serialize");
        let back: Focus = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(f, back);
    }
}
