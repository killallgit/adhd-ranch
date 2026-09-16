use std::fs;
use std::path::PathBuf;

use adhd_ranch_domain::claude_hook::agent_session_from_payload;
use adhd_ranch_domain::AgentSession;

pub trait AgentSessionStore: Send + Sync {
    fn list(&self) -> Vec<AgentSession>;
}

pub struct ClaudeSessionStore {
    sessions_dir: PathBuf,
}

impl ClaudeSessionStore {
    pub fn new(sessions_dir: PathBuf) -> Self {
        Self { sessions_dir }
    }
}

impl AgentSessionStore for ClaudeSessionStore {
    fn list(&self) -> Vec<AgentSession> {
        let entries = match fs::read_dir(&self.sessions_dir) {
            Ok(entries) => entries,
            Err(error) => {
                log::warn!(
                    "claude sessions: cannot read {}: {error}",
                    self.sessions_dir.display()
                );
                return Vec::new();
            }
        };

        let mut sessions: Vec<AgentSession> = entries
            .filter_map(Result::ok)
            .map(|entry| entry.path())
            .filter(|path| path.extension().is_some_and(|ext| ext == "json"))
            .filter_map(|path| fs::read_to_string(path).ok())
            .filter_map(|payload| agent_session_from_payload(&payload))
            .collect();
        sessions.sort_by(|a, b| a.id.cmp(&b.id));
        sessions
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn write_session(dir: &TempDir, file: &str, payload: &str) {
        fs::write(dir.path().join(file), payload).unwrap();
    }

    #[test]
    fn lists_one_session_per_file() {
        let dir = TempDir::new().unwrap();
        write_session(&dir, "b.json", r#"{"session_id":"b","cwd":"/code/two"}"#);
        write_session(&dir, "a.json", r#"{"session_id":"a","cwd":"/code/one"}"#);

        let sessions = ClaudeSessionStore::new(dir.path().to_path_buf()).list();

        assert_eq!(
            sessions,
            vec![
                AgentSession {
                    id: "a".into(),
                    name: "one".into()
                },
                AgentSession {
                    id: "b".into(),
                    name: "two".into()
                },
            ]
        );
    }

    #[test]
    fn skips_in_flight_temp_files() {
        let dir = TempDir::new().unwrap();
        write_session(&dir, ".a.json.tmp", r#"{"session_id":"a"}"#);

        let sessions = ClaudeSessionStore::new(dir.path().to_path_buf()).list();

        assert!(sessions.is_empty());
    }

    #[test]
    fn skips_unreadable_session_files() {
        let dir = TempDir::new().unwrap();
        write_session(&dir, "a.json", "{truncated");

        let sessions = ClaudeSessionStore::new(dir.path().to_path_buf()).list();

        assert!(sessions.is_empty());
    }

    #[test]
    fn missing_sessions_dir_lists_no_sessions() {
        let dir = TempDir::new().unwrap();

        let sessions = ClaudeSessionStore::new(dir.path().join("missing")).list();

        assert!(sessions.is_empty());
    }
}
