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
    TaskIndexOutOfRange { focus_id: String, index: usize },
}

impl std::fmt::Display for WriterError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(e) => write!(f, "writer io: {e}"),
            Self::FocusNotFound(slug) => write!(f, "focus not found: {slug}"),
            Self::FocusAlreadyExists(slug) => write!(f, "focus already exists: {slug}"),
            Self::TaskIndexOutOfRange { focus_id, index } => {
                write!(f, "task index {index} out of range for focus {focus_id}")
            }
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
    fn delete_task(&self, focus_id: &str, index: usize) -> Result<(), WriterError>;
    fn create_focus(
        &self,
        new_focus: &NewFocus,
        id: &str,
        created_at: &str,
    ) -> Result<String, WriterError>;
    fn delete_focus(&self, focus_id: &str) -> Result<(), WriterError>;
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

    fn focus_md(&self, focus_id: &str) -> PathBuf {
        self.root.join(focus_id).join("focus.md")
    }

    fn read_focus(&self, focus_id: &str) -> Result<String, WriterError> {
        match fs::read_to_string(self.focus_md(focus_id)) {
            Ok(s) => Ok(s),
            Err(e) if e.kind() == io::ErrorKind::NotFound => {
                Err(WriterError::FocusNotFound(focus_id.to_string()))
            }
            Err(e) => Err(e.into()),
        }
    }
}

impl FocusWriter for MarkdownFocusWriter {
    fn append_task(&self, focus_id: &str, text: &str) -> Result<(), WriterError> {
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

    fn delete_task(&self, focus_id: &str, index: usize) -> Result<(), WriterError> {
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
                .ok_or_else(|| WriterError::TaskIndexOutOfRange {
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

    fn delete_focus(&self, focus_id: &str) -> Result<(), WriterError> {
        let dir = self.root.join(focus_id);
        if !dir.exists() {
            return Err(WriterError::FocusNotFound(focus_id.to_string()));
        }
        fs::remove_dir_all(&dir)?;
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

    fn focus_md(id: &str, tasks: &[&str]) -> String {
        let mut s = format!(
            "---\nid: {id}\ntitle: A\ndescription:\ncreated_at: 2026-04-30T12:00:00Z\n---\n"
        );
        for t in tasks {
            s.push_str(&format!("- [ ] {t}\n"));
        }
        s
    }

    #[test]
    fn append_task_appends_a_checkbox_bullet() {
        let dir = TempDir::new().unwrap();
        write_focus(dir.path(), "a", &focus_md("a", &["existing"]));
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
        assert!(matches!(err, WriterError::FocusNotFound(slug) if slug == "missing"));
    }

    #[test]
    fn delete_task_removes_only_the_indexed_bullet() {
        let dir = TempDir::new().unwrap();
        write_focus(dir.path(), "a", &focus_md("a", &["one", "two", "three"]));
        let writer = MarkdownFocusWriter::new(dir.path());
        writer.delete_task("a", 1).unwrap();
        let content = fs::read_to_string(dir.path().join("a/focus.md")).unwrap();
        assert!(content.contains("- [ ] one"));
        assert!(!content.contains("- [ ] two"));
        assert!(content.contains("- [ ] three"));
    }

    #[test]
    fn delete_task_errors_when_index_out_of_range() {
        let dir = TempDir::new().unwrap();
        write_focus(dir.path(), "a", &focus_md("a", &["one"]));
        let writer = MarkdownFocusWriter::new(dir.path());
        let err = writer.delete_task("a", 5).unwrap_err();
        assert!(matches!(
            err,
            WriterError::TaskIndexOutOfRange { index: 5, .. }
        ));
    }

    #[test]
    fn delete_task_errors_on_missing_focus() {
        let dir = TempDir::new().unwrap();
        let writer = MarkdownFocusWriter::new(dir.path());
        let err = writer.delete_task("missing", 0).unwrap_err();
        assert!(matches!(err, WriterError::FocusNotFound(_)));
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
        assert!(matches!(err, WriterError::FocusAlreadyExists(slug) if slug == "customer-x-bug"));
    }

    #[test]
    fn delete_focus_removes_dir() {
        let dir = TempDir::new().unwrap();
        write_focus(dir.path(), "a", &focus_md("a", &[]));
        let writer = MarkdownFocusWriter::new(dir.path());
        writer.delete_focus("a").unwrap();
        assert!(!dir.path().join("a").exists());
    }

    #[test]
    fn delete_focus_errors_when_missing() {
        let dir = TempDir::new().unwrap();
        let writer = MarkdownFocusWriter::new(dir.path());
        let err = writer.delete_focus("missing").unwrap_err();
        assert!(matches!(err, WriterError::FocusNotFound(_)));
    }
}
