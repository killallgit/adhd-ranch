use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use adhd_ranch_domain::{slugify, NewFocus};

use crate::atomic::atomic_write;

#[derive(Debug)]
pub enum WriterError {
    Io(io::Error),
    FocusNotFound(String),
    FocusAlreadyExists(String),
}

impl std::fmt::Display for WriterError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(e) => write!(f, "writer io: {e}"),
            Self::FocusNotFound(slug) => write!(f, "focus not found: {slug}"),
            Self::FocusAlreadyExists(slug) => write!(f, "focus already exists: {slug}"),
        }
    }
}

impl std::error::Error for WriterError {}

impl From<io::Error> for WriterError {
    fn from(e: io::Error) -> Self {
        Self::Io(e)
    }
}

pub trait FocusWriter: Send + Sync {
    fn append_task(&self, focus_id: &str, text: &str) -> Result<(), WriterError>;
    fn create_focus(
        &self,
        new_focus: &NewFocus,
        id: &str,
        created_at: &str,
    ) -> Result<String, WriterError>;
}

pub struct MarkdownFocusWriter {
    root: PathBuf,
}

impl MarkdownFocusWriter {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    pub fn root(&self) -> &Path {
        &self.root
    }
}

impl FocusWriter for MarkdownFocusWriter {
    fn append_task(&self, focus_id: &str, text: &str) -> Result<(), WriterError> {
        let path = self.root.join(focus_id).join("focus.md");
        let current = match fs::read_to_string(&path) {
            Ok(s) => s,
            Err(e) if e.kind() == io::ErrorKind::NotFound => {
                return Err(WriterError::FocusNotFound(focus_id.to_string()));
            }
            Err(e) => return Err(e.into()),
        };

        let mut next = current;
        if !next.ends_with('\n') {
            next.push('\n');
        }
        next.push_str("- [ ] ");
        next.push_str(text);
        next.push('\n');

        atomic_write(&path, next.as_bytes())?;
        Ok(())
    }

    fn create_focus(
        &self,
        new_focus: &NewFocus,
        id: &str,
        created_at: &str,
    ) -> Result<String, WriterError> {
        let slug = slugify(&new_focus.title);
        let dir = self.root.join(&slug);
        if dir.exists() {
            return Err(WriterError::FocusAlreadyExists(slug));
        }
        fs::create_dir_all(&dir)?;

        let body = format!(
            "---\nid: {id}\ntitle: {title}\ndescription: {description}\ncreated_at: {created_at}\n---\n",
            title = new_focus.title,
            description = new_focus.description,
        );
        atomic_write(&dir.join("focus.md"), body.as_bytes())?;
        Ok(slug)
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

    fn focus_md(id: &str) -> String {
        format!("---\nid: {id}\ntitle: A\ndescription:\ncreated_at: 2026-04-30T12:00:00Z\n---\n- [ ] existing\n")
    }

    #[test]
    fn append_task_appends_a_checkbox_bullet() {
        let dir = TempDir::new().unwrap();
        write_focus(dir.path(), "a", &focus_md("a"));
        let writer = MarkdownFocusWriter::new(dir.path());
        writer.append_task("a", "new task").unwrap();
        let content = fs::read_to_string(dir.path().join("a/focus.md")).unwrap();
        assert!(content.contains("- [ ] existing"));
        assert!(content.trim_end().ends_with("- [ ] new task"));
    }

    #[test]
    fn append_task_errors_on_missing_focus() {
        let dir = TempDir::new().unwrap();
        let writer = MarkdownFocusWriter::new(dir.path());
        let err = writer.append_task("missing", "x").unwrap_err();
        match err {
            WriterError::FocusNotFound(slug) => assert_eq!(slug, "missing"),
            other => panic!("expected FocusNotFound, got {other}"),
        }
    }

    #[test]
    fn create_focus_writes_frontmatter_and_returns_slug() {
        let dir = TempDir::new().unwrap();
        let writer = MarkdownFocusWriter::new(dir.path());
        let slug = writer
            .create_focus(
                &NewFocus {
                    title: "Customer X bug".into(),
                    description: "ship it".into(),
                },
                "id-1",
                "2026-04-30T12:00:00Z",
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
        let writer = MarkdownFocusWriter::new(dir.path());
        write_focus(dir.path(), "customer-x-bug", "existing");
        let err = writer
            .create_focus(
                &NewFocus {
                    title: "Customer X bug".into(),
                    description: "".into(),
                },
                "id-2",
                "2026-04-30T12:00:00Z",
            )
            .unwrap_err();
        match err {
            WriterError::FocusAlreadyExists(slug) => assert_eq!(slug, "customer-x-bug"),
            other => panic!("expected FocusAlreadyExists, got {other}"),
        }
    }
}
