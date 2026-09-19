use std::path::Path;

use serde_json::Value;

use crate::session::{pen_for_cwd, AgentSession, SessionActivity};

/// Read the JSON a hook firing delivered on stdin into the animal it describes.
///
/// Other tools speak this same hook protocol, so a payload that does not carry what
/// the ranch needs is simply not ours to draw.
pub fn agent_session_from_payload(
    payload: &str,
    activity: SessionActivity,
) -> Option<AgentSession> {
    let value: Value = serde_json::from_str(payload).ok()?;
    let id = session_id(&value)?;
    // A session with no working directory cannot be placed in a pen, and a pen is
    // the only place the ranch draws an animal.
    let cwd = Path::new(value.get("cwd").and_then(Value::as_str)?);
    Some(AgentSession {
        id,
        name: cwd.file_name()?.to_str()?.to_string(),
        pen: pen_for_cwd(cwd)?,
        activity,
    })
}

/// The session a payload is about, for firings that only need to name one.
pub fn session_id_from_payload(payload: &str) -> Option<String> {
    session_id(&serde_json::from_str(payload).ok()?)
}

fn session_id(payload: &Value) -> Option<String> {
    let id = payload.get("session_id")?.as_str()?.trim();
    (!id.is_empty()).then(|| id.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    const PAYLOAD: &str = r#"{"session_id":"4af8005a-7a52","cwd":"/code/app"}"#;

    #[test]
    fn a_payload_names_its_session() {
        assert_eq!(session_id_from_payload(PAYLOAD).unwrap(), "4af8005a-7a52");
    }

    #[test]
    fn a_payload_without_a_working_directory_is_not_an_animal() {
        let no_cwd = r#"{"session_id":"4af8005a-7a52"}"#;

        assert!(agent_session_from_payload(no_cwd, SessionActivity::Idle).is_none());
    }

    #[test]
    fn a_session_is_named_for_the_directory_it_works_in() {
        let session = agent_session_from_payload(PAYLOAD, SessionActivity::Working).unwrap();

        assert_eq!(session.name, "app");
        assert_eq!(session.activity, SessionActivity::Working);
    }
}
