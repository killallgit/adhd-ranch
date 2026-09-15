use std::fs;
use std::path::PathBuf;

use adhd_ranch_domain::claude_hook::animal_from_session_start;
use adhd_ranch_domain::Animal;

pub trait AnimalStore: Send + Sync {
    fn list(&self) -> Vec<Animal>;
}

pub struct ClaudeSessionStore {
    sessions_dir: PathBuf,
}

impl ClaudeSessionStore {
    pub fn new(sessions_dir: PathBuf) -> Self {
        Self { sessions_dir }
    }
}

impl AnimalStore for ClaudeSessionStore {
    fn list(&self) -> Vec<Animal> {
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

        let mut animals: Vec<Animal> = entries
            .filter_map(Result::ok)
            .map(|entry| entry.path())
            .filter(|path| path.extension().is_some_and(|ext| ext == "json"))
            .filter_map(|path| fs::read_to_string(path).ok())
            .filter_map(|payload| animal_from_session_start(&payload))
            .collect();
        animals.sort_by(|a, b| a.id.cmp(&b.id));
        animals
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
    fn lists_one_animal_per_session_file() {
        let dir = TempDir::new().unwrap();
        write_session(&dir, "b.json", r#"{"session_id":"b","cwd":"/code/two"}"#);
        write_session(&dir, "a.json", r#"{"session_id":"a","cwd":"/code/one"}"#);

        let animals = ClaudeSessionStore::new(dir.path().to_path_buf()).list();

        assert_eq!(
            animals,
            vec![
                Animal {
                    id: "a".into(),
                    name: "one".into()
                },
                Animal {
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

        let animals = ClaudeSessionStore::new(dir.path().to_path_buf()).list();

        assert!(animals.is_empty());
    }

    #[test]
    fn skips_unreadable_session_files() {
        let dir = TempDir::new().unwrap();
        write_session(&dir, "a.json", "{truncated");

        let animals = ClaudeSessionStore::new(dir.path().to_path_buf()).list();

        assert!(animals.is_empty());
    }

    #[test]
    fn missing_sessions_dir_lists_no_animals() {
        let dir = TempDir::new().unwrap();

        let animals = ClaudeSessionStore::new(dir.path().join("missing")).list();

        assert!(animals.is_empty());
    }
}
