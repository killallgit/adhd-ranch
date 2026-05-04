use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use adhd_ranch_domain::{parse_focus_md, slugify, Focus, FocusTimer, NewFocus, ParseError};

use crate::atomic::atomic_write;

#[derive(Debug)]
pub enum FocusStoreError {
    Io(io::Error),
    Parse { path: PathBuf, error: ParseError },
    NotFound(String),
    AlreadyExists(String),
    TaskIndexOutOfRange { focus_id: String, index: usize },
}

impl std::fmt::Display for FocusStoreError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(e) => write!(f, "focus store io: {e}"),
            Self::Parse { path, error } => {
                write!(f, "parse error in {}: {error}", path.display())
            }
            Self::NotFound(slug) => write!(f, "focus not found: {slug}"),
            Self::AlreadyExists(slug) => write!(f, "focus already exists: {slug}"),
            Self::TaskIndexOutOfRange { focus_id, index } => {
                write!(f, "task index {index} out of range for focus {focus_id}")
            }
        }
    }
}

impl std::error::Error for FocusStoreError {}

impl From<io::Error> for FocusStoreError {
    fn from(e: io::Error) -> Self {
        Self::Io(e)
    }
}

pub trait FocusStore: Send + Sync {
    fn list(&self) -> Result<Vec<Focus>, FocusStoreError>;
    fn create_focus(
        &self,
        new_focus: &NewFocus,
        id: &str,
        created_at: &str,
        timer: Option<FocusTimer>,
    ) -> Result<String, FocusStoreError>;
    fn delete_focus(&self, focus_id: &str) -> Result<(), FocusStoreError>;
    fn append_task(&self, focus_id: &str, text: &str) -> Result<(), FocusStoreError>;
    fn delete_task(&self, focus_id: &str, index: usize) -> Result<(), FocusStoreError>;
}

pub struct MarkdownFocusStore {
    root: PathBuf,
}

impl MarkdownFocusStore {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    fn focus_md(&self, focus_id: &str) -> PathBuf {
        self.root.join(focus_id).join("focus.md")
    }

    fn read_focus(&self, focus_id: &str) -> Result<String, FocusStoreError> {
        match fs::read_to_string(self.focus_md(focus_id)) {
            Ok(s) => Ok(s),
            Err(e) if e.kind() == io::ErrorKind::NotFound => {
                Err(FocusStoreError::NotFound(focus_id.to_string()))
            }
            Err(e) => Err(e.into()),
        }
    }
}

impl FocusStore for MarkdownFocusStore {
    fn list(&self) -> Result<Vec<Focus>, FocusStoreError> {
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
            let mut focus = parse_focus_md(&raw).map_err(|error| FocusStoreError::Parse {
                path: focus_md,
                error,
            })?;
            // The authoritative ID is the directory name (slug), not the
            // frontmatter uuid field. All store ops (delete, append_task, etc.)
            // join focus_id to the root dir, so the ID must be the slug.
            focus.id = adhd_ranch_domain::FocusId(entry.file_name().to_string_lossy().into_owned());
            let timer_path = entry.path().join("timer.json");
            if timer_path.is_file() {
                let raw = fs::read_to_string(&timer_path)?;
                focus.timer = Some(
                    serde_json::from_str(&raw)
                        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?,
                );
            }
            out.push(focus);
        }

        out.sort_by(|a, b| {
            a.created_at
                .cmp(&b.created_at)
                .then_with(|| a.id.0.cmp(&b.id.0))
        });
        Ok(out)
    }

    fn create_focus(
        &self,
        new_focus: &NewFocus,
        id: &str,
        created_at: &str,
        timer: Option<FocusTimer>,
    ) -> Result<String, FocusStoreError> {
        let slug = slugify(&new_focus.title);
        let dir = self.root.join(&slug);
        if dir.exists() {
            return Err(FocusStoreError::AlreadyExists(slug));
        }
        fs::create_dir_all(&dir)?;

        let body = format!(
            "---\nid: {id}\ntitle: {title}\ndescription: {description}\ncreated_at: {created_at}\n---\n",
            title = new_focus.title,
            description = new_focus.description,
        );
        atomic_write(&dir.join("focus.md"), body.as_bytes())?;

        if let Some(t) = timer {
            let json = serde_json::to_vec(&t).map_err(io::Error::other)?;
            if let Err(err) = atomic_write(&dir.join("timer.json"), &json) {
                let _ = fs::remove_dir_all(&dir);
                return Err(err.into());
            }
        }

        Ok(slug)
    }

    fn delete_focus(&self, focus_id: &str) -> Result<(), FocusStoreError> {
        let dir = self.root.join(focus_id);
        if !dir.exists() {
            return Err(FocusStoreError::NotFound(focus_id.to_string()));
        }
        fs::remove_dir_all(&dir)?;
        Ok(())
    }

    fn append_task(&self, focus_id: &str, text: &str) -> Result<(), FocusStoreError> {
        let mut next = self.read_focus(focus_id)?;
        if !next.ends_with('\n') {
            next.push('\n');
        }
        next.push_str("- [ ] ");
        next.push_str(text);
        next.push('\n');
        atomic_write(&self.focus_md(focus_id), next.as_bytes())?;
        Ok(())
    }

    fn delete_task(&self, focus_id: &str, index: usize) -> Result<(), FocusStoreError> {
        let current = self.read_focus(focus_id)?;
        let mut bullet_indices: Vec<usize> = Vec::new();
        for (line_idx, line) in current.lines().enumerate() {
            let trimmed = line.trim_start();
            if trimmed.starts_with("- [ ]") || trimmed.starts_with("- [x]") {
                bullet_indices.push(line_idx);
            }
        }
        let target =
            *bullet_indices
                .get(index)
                .ok_or_else(|| FocusStoreError::TaskIndexOutOfRange {
                    focus_id: focus_id.to_string(),
                    index,
                })?;

        let mut out = String::with_capacity(current.len());
        let trailing_newline = current.ends_with('\n');
        for (line_idx, line) in current.lines().enumerate() {
            if line_idx == target {
                continue;
            }
            out.push_str(line);
            out.push('\n');
        }
        if !trailing_newline {
            out.pop();
        }
        atomic_write(&self.focus_md(focus_id), out.as_bytes())?;
        Ok(())
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

    fn focus_md(id: &str, tasks: &[&str]) -> String {
        fixture(id, "A", "2026-04-30T12:00:00Z", tasks)
    }

    #[test]
    fn missing_root_returns_empty_list_not_error() {
        let dir = TempDir::new().unwrap();
        let store = MarkdownFocusStore::new(dir.path().join("does-not-exist"));
        assert!(store.list().unwrap().is_empty());
    }

    #[test]
    fn empty_root_returns_empty_list() {
        let dir = TempDir::new().unwrap();
        let store = MarkdownFocusStore::new(dir.path());
        assert!(store.list().unwrap().is_empty());
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

        let store = MarkdownFocusStore::new(dir.path());
        let focuses = store.list().unwrap();
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
        let store = MarkdownFocusStore::new(dir.path());
        let focuses = store.list().unwrap();
        assert_eq!(focuses.len(), 1);
    }

    #[test]
    fn parse_error_is_surfaced_with_path() {
        let dir = TempDir::new().unwrap();
        write_focus(dir.path(), "broken", "not yaml at all");
        let store = MarkdownFocusStore::new(dir.path());
        let err = store.list().unwrap_err();
        match err {
            FocusStoreError::Parse { path, .. } => {
                assert!(path.ends_with("broken/focus.md"), "{path:?}");
            }
            other => panic!("expected Parse, got {other:?}"),
        }
    }

    #[test]
    fn append_task_appends_a_checkbox_bullet() {
        let dir = TempDir::new().unwrap();
        write_focus(dir.path(), "a", &focus_md("a", &["existing"]));
        let store = MarkdownFocusStore::new(dir.path());
        store.append_task("a", "new task").unwrap();
        let content = fs::read_to_string(dir.path().join("a/focus.md")).unwrap();
        assert!(content.contains("- [ ] existing"));
        assert!(content.trim_end().ends_with("- [ ] new task"));
    }

    #[test]
    fn append_task_errors_on_missing_focus() {
        let dir = TempDir::new().unwrap();
        let store = MarkdownFocusStore::new(dir.path());
        let err = store.append_task("missing", "x").unwrap_err();
        assert!(matches!(err, FocusStoreError::NotFound(slug) if slug == "missing"));
    }

    #[test]
    fn delete_task_removes_only_the_indexed_bullet() {
        let dir = TempDir::new().unwrap();
        write_focus(dir.path(), "a", &focus_md("a", &["one", "two", "three"]));
        let store = MarkdownFocusStore::new(dir.path());
        store.delete_task("a", 1).unwrap();
        let content = fs::read_to_string(dir.path().join("a/focus.md")).unwrap();
        assert!(content.contains("- [ ] one"));
        assert!(!content.contains("- [ ] two"));
        assert!(content.contains("- [ ] three"));
    }

    #[test]
    fn delete_task_errors_when_index_out_of_range() {
        let dir = TempDir::new().unwrap();
        write_focus(dir.path(), "a", &focus_md("a", &["one"]));
        let store = MarkdownFocusStore::new(dir.path());
        let err = store.delete_task("a", 5).unwrap_err();
        assert!(matches!(
            err,
            FocusStoreError::TaskIndexOutOfRange { index: 5, .. }
        ));
    }

    #[test]
    fn delete_task_errors_on_missing_focus() {
        let dir = TempDir::new().unwrap();
        let store = MarkdownFocusStore::new(dir.path());
        let err = store.delete_task("missing", 0).unwrap_err();
        assert!(matches!(err, FocusStoreError::NotFound(_)));
    }

    #[test]
    fn create_focus_writes_frontmatter_and_returns_slug() {
        let dir = TempDir::new().unwrap();
        let store = MarkdownFocusStore::new(dir.path());
        let slug = store
            .create_focus(
                &NewFocus {
                    title: "Customer X bug".into(),
                    description: "ship it".into(),
                    timer_preset: None,
                },
                "id-1",
                "2026-04-30T12:00:00Z",
                None,
            )
            .unwrap();
        assert_eq!(slug, "customer-x-bug");
        let content = fs::read_to_string(dir.path().join("customer-x-bug/focus.md")).unwrap();
        assert!(content.starts_with("---\nid: id-1\n"));
        assert!(content.contains("title: Customer X bug"));
        assert!(content.contains("description: ship it"));
    }

    #[test]
    fn create_focus_errors_when_slug_collides() {
        let dir = TempDir::new().unwrap();
        let store = MarkdownFocusStore::new(dir.path());
        write_focus(dir.path(), "customer-x-bug", "existing");
        let err = store
            .create_focus(
                &NewFocus {
                    title: "Customer X bug".into(),
                    description: "".into(),
                    timer_preset: None,
                },
                "id-2",
                "2026-04-30T12:00:00Z",
                None,
            )
            .unwrap_err();
        assert!(matches!(err, FocusStoreError::AlreadyExists(slug) if slug == "customer-x-bug"));
    }

    #[test]
    fn delete_focus_removes_dir() {
        let dir = TempDir::new().unwrap();
        write_focus(dir.path(), "a", &focus_md("a", &[]));
        let store = MarkdownFocusStore::new(dir.path());
        store.delete_focus("a").unwrap();
        assert!(!dir.path().join("a").exists());
    }

    #[test]
    fn delete_focus_errors_when_missing() {
        let dir = TempDir::new().unwrap();
        let store = MarkdownFocusStore::new(dir.path());
        let err = store.delete_focus("missing").unwrap_err();
        assert!(matches!(err, FocusStoreError::NotFound(_)));
    }
}
