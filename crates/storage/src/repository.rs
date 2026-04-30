use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use adhd_ranch_domain::{parse_focus_md, Focus, ParseError};

#[derive(Debug)]
pub enum RepositoryError {
    Io(io::Error),
    Parse { path: PathBuf, error: ParseError },
}

impl std::fmt::Display for RepositoryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(error) => write!(f, "io error: {error}"),
            Self::Parse { path, error } => {
                write!(f, "parse error in {}: {error}", path.display())
            }
        }
    }
}

impl std::error::Error for RepositoryError {}

impl From<io::Error> for RepositoryError {
    fn from(error: io::Error) -> Self {
        Self::Io(error)
    }
}

pub trait FocusRepository: Send + Sync {
    fn list(&self) -> Result<Vec<Focus>, RepositoryError>;
}

pub struct MarkdownFocusRepository {
    root: PathBuf,
}

impl MarkdownFocusRepository {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    pub fn root(&self) -> &Path {
        &self.root
    }
}

impl FocusRepository for MarkdownFocusRepository {
    fn list(&self) -> Result<Vec<Focus>, RepositoryError> {
        let entries = match fs::read_dir(&self.root) {
            Ok(entries) => entries,
            Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(Vec::new()),
            Err(error) => return Err(error.into()),
        };

        let mut out: Vec<Focus> = Vec::new();
        for entry in entries {
            let entry = entry?;
            if !entry.file_type()?.is_dir() {
                continue;
            }
            let focus_md = entry.path().join("focus.md");
            if !focus_md.is_file() {
                continue;
            }
            let raw = fs::read_to_string(&focus_md)?;
            let focus = parse_focus_md(&raw).map_err(|error| RepositoryError::Parse {
                path: focus_md,
                error,
            })?;
            out.push(focus);
        }

        out.sort_by(|a, b| {
            a.created_at
                .cmp(&b.created_at)
                .then_with(|| a.id.0.cmp(&b.id.0))
        });
        Ok(out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn write_focus(root: &Path, slug: &str, body: &str) {
        let dir = root.join(slug);
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("focus.md"), body).unwrap();
    }

    fn fixture(id: &str, title: &str, created_at: &str, tasks: &[&str]) -> String {
        let mut s =
            format!("---\nid: {id}\ntitle: {title}\ndescription:\ncreated_at: {created_at}\n---\n");
        for t in tasks {
            s.push_str(&format!("- [ ] {t}\n"));
        }
        s
    }

    #[test]
    fn missing_root_returns_empty_list_not_error() {
        let dir = TempDir::new().unwrap();
        let repo = MarkdownFocusRepository::new(dir.path().join("does-not-exist"));
        assert!(repo.list().unwrap().is_empty());
    }

    #[test]
    fn empty_root_returns_empty_list() {
        let dir = TempDir::new().unwrap();
        let repo = MarkdownFocusRepository::new(dir.path());
        assert!(repo.list().unwrap().is_empty());
    }

    #[test]
    fn lists_focuses_with_their_tasks() {
        let dir = TempDir::new().unwrap();
        write_focus(
            dir.path(),
            "a",
            &fixture("a", "Alpha", "2026-04-30T12:00:00Z", &["one"]),
        );
        write_focus(
            dir.path(),
            "b",
            &fixture("b", "Beta", "2026-04-30T12:01:00Z", &["two", "three"]),
        );

        let repo = MarkdownFocusRepository::new(dir.path());
        let focuses = repo.list().unwrap();
        assert_eq!(focuses.len(), 2);
        assert_eq!(focuses[0].title, "Alpha");
        assert_eq!(focuses[0].tasks.len(), 1);
        assert_eq!(focuses[1].title, "Beta");
        assert_eq!(focuses[1].tasks.len(), 2);
    }

    #[test]
    fn ignores_directories_without_focus_md() {
        let dir = TempDir::new().unwrap();
        fs::create_dir_all(dir.path().join("not-a-focus")).unwrap();
        write_focus(
            dir.path(),
            "a",
            &fixture("a", "A", "2026-04-30T12:00:00Z", &[]),
        );
        let repo = MarkdownFocusRepository::new(dir.path());
        let focuses = repo.list().unwrap();
        assert_eq!(focuses.len(), 1);
    }

    #[test]
    fn parse_error_is_surfaced_with_path() {
        let dir = TempDir::new().unwrap();
        write_focus(dir.path(), "broken", "not yaml at all");
        let repo = MarkdownFocusRepository::new(dir.path());
        let err = repo.list().unwrap_err();
        match err {
            RepositoryError::Parse { path, .. } => {
                assert!(path.ends_with("broken/focus.md"), "{path:?}");
            }
            other => panic!("expected Parse, got {other:?}"),
        }
    }
}
