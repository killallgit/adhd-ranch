use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use adhd_ranch_domain::{
    parse_focus_md, slugify, Focus, FocusTimer, NewFocus, ParseError, TimerStatus,
};

use crate::atomic::atomic_write;
use crate::focus_document::{FocusDocument, FocusDocumentError};

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
    fn rename_focus(&self, focus_id: &str, title: &str) -> Result<(), FocusStoreError>;
    fn append_task(&self, focus_id: &str, text: &str) -> Result<(), FocusStoreError>;
    fn delete_task(&self, focus_id: &str, index: usize) -> Result<(), FocusStoreError>;
    fn update_task(&self, focus_id: &str, index: usize, text: &str) -> Result<(), FocusStoreError>;
    fn toggle_task(&self, focus_id: &str, index: usize, done: bool) -> Result<(), FocusStoreError>;
    fn update_timer(&self, focus_id: &str, timer: &FocusTimer) -> Result<(), FocusStoreError>;
    fn clear_timer(&self, focus_id: &str) -> Result<(), FocusStoreError>;
    fn update_task_timer(
        &self,
        focus_id: &str,
        index: usize,
        timer: &FocusTimer,
    ) -> Result<(), FocusStoreError>;
    fn clear_task_timer(&self, focus_id: &str, index: usize) -> Result<(), FocusStoreError>;
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

    fn timer_json(&self, focus_id: &str) -> PathBuf {
        self.root.join(focus_id).join("timer.json")
    }

    fn task_timers_json(&self, focus_id: &str) -> PathBuf {
        self.root.join(focus_id).join("task-timers.json")
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
                // A corrupted timer.json must not take down list() — the focus
                // itself is still readable and useful. Surface as `timer: None`
                // so the UI degrades gracefully; user can fix the sidecar by
                // recreating the focus.
                focus.timer = serde_json::from_str(&raw).ok();
            }
            let task_timers_path = entry.path().join("task-timers.json");
            if task_timers_path.is_file() {
                let raw = fs::read_to_string(&task_timers_path)?;
                if let Ok(timers) = serde_json::from_str::<Vec<Option<FocusTimer>>>(&raw) {
                    for (task, timer) in focus.tasks.iter_mut().zip(timers) {
                        task.timer = timer;
                    }
                }
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
        let slug = slugify(new_focus.title());
        let dir = self.root.join(&slug);
        if dir.exists() {
            return Err(FocusStoreError::AlreadyExists(slug));
        }
        fs::create_dir_all(&dir)?;

        let body = format!(
            "---\nid: {id}\ntitle: {title}\ndescription: {description}\ncreated_at: {created_at}\n---\n",
            title = new_focus.title(),
            description = new_focus.description(),
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

    fn rename_focus(&self, focus_id: &str, title: &str) -> Result<(), FocusStoreError> {
        let current = self.read_focus(focus_id)?;
        let next = FocusDocument::from_raw(current)
            .rename_focus(title)
            .into_raw();
        atomic_write(&self.focus_md(focus_id), next.as_bytes())?;
        Ok(())
    }

    fn append_task(&self, focus_id: &str, text: &str) -> Result<(), FocusStoreError> {
        let current = self.read_focus(focus_id)?;
        let next = FocusDocument::from_raw(current)
            .append_task(text)
            .into_raw();
        atomic_write(&self.focus_md(focus_id), next.as_bytes())?;
        if let Err(err) = self.clear_expired_timer(focus_id) {
            log::warn!("failed to clear expired timer after appending task to {focus_id}: {err}");
        }
        Ok(())
    }

    fn delete_task(&self, focus_id: &str, index: usize) -> Result<(), FocusStoreError> {
        let current = self.read_focus(focus_id)?;
        let next = FocusDocument::from_raw(current)
            .delete_task(index)
            .map_err(|e| map_document_error(focus_id, e))?
            .into_raw();
        atomic_write(&self.focus_md(focus_id), next.as_bytes())?;
        if let Err(err) = self.remove_task_timer_index(focus_id, index) {
            log::warn!(
                "failed to remove task timer {index} after deleting task from {focus_id}: {err}"
            );
        }
        Ok(())
    }

    fn update_task(&self, focus_id: &str, index: usize, text: &str) -> Result<(), FocusStoreError> {
        let current = self.read_focus(focus_id)?;
        let next = FocusDocument::from_raw(current)
            .update_task(index, text)
            .map_err(|e| map_document_error(focus_id, e))?
            .into_raw();
        atomic_write(&self.focus_md(focus_id), next.as_bytes())?;
        Ok(())
    }

    fn toggle_task(&self, focus_id: &str, index: usize, done: bool) -> Result<(), FocusStoreError> {
        let current = self.read_focus(focus_id)?;
        let next = FocusDocument::from_raw(current)
            .toggle_task(index, done)
            .map_err(|e| map_document_error(focus_id, e))?
            .into_raw();
        atomic_write(&self.focus_md(focus_id), next.as_bytes())?;
        Ok(())
    }

    fn update_timer(&self, focus_id: &str, timer: &FocusTimer) -> Result<(), FocusStoreError> {
        let dir = self.root.join(focus_id);
        if !dir.is_dir() {
            return Err(FocusStoreError::NotFound(focus_id.to_string()));
        }
        let bytes = serde_json::to_vec(timer).map_err(io::Error::other)?;
        match atomic_write(&dir.join("timer.json"), &bytes) {
            Ok(()) => Ok(()),
            Err(e) if e.kind() == io::ErrorKind::NotFound => {
                Err(FocusStoreError::NotFound(focus_id.to_string()))
            }
            Err(e) => Err(e.into()),
        }
    }

    fn clear_timer(&self, focus_id: &str) -> Result<(), FocusStoreError> {
        let dir = self.root.join(focus_id);
        if !dir.is_dir() {
            return Err(FocusStoreError::NotFound(focus_id.to_string()));
        }
        match fs::remove_file(self.timer_json(focus_id)) {
            Ok(()) => Ok(()),
            Err(err) if err.kind() == io::ErrorKind::NotFound => Ok(()),
            Err(err) => Err(err.into()),
        }
    }

    fn update_task_timer(
        &self,
        focus_id: &str,
        index: usize,
        timer: &FocusTimer,
    ) -> Result<(), FocusStoreError> {
        let task_count = self.task_count(focus_id)?;
        if index >= task_count {
            return Err(FocusStoreError::TaskIndexOutOfRange {
                focus_id: focus_id.to_string(),
                index,
            });
        }
        let mut timers = self.read_task_timers(focus_id)?;
        timers.resize(task_count, None);
        timers[index] = Some(timer.clone());
        self.write_task_timers(focus_id, &timers)
    }

    fn clear_task_timer(&self, focus_id: &str, index: usize) -> Result<(), FocusStoreError> {
        let task_count = self.task_count(focus_id)?;
        if index >= task_count {
            return Err(FocusStoreError::TaskIndexOutOfRange {
                focus_id: focus_id.to_string(),
                index,
            });
        }
        let mut timers = self.read_task_timers(focus_id)?;
        timers.resize(task_count, None);
        timers[index] = None;
        self.write_task_timers(focus_id, &timers)
    }
}

impl MarkdownFocusStore {
    fn task_count(&self, focus_id: &str) -> Result<usize, FocusStoreError> {
        let raw = self.read_focus(focus_id)?;
        let focus = parse_focus_md(&raw).map_err(|error| FocusStoreError::Parse {
            path: self.focus_md(focus_id),
            error,
        })?;
        Ok(focus.tasks.len())
    }

    fn read_task_timers(&self, focus_id: &str) -> Result<Vec<Option<FocusTimer>>, FocusStoreError> {
        match fs::read_to_string(self.task_timers_json(focus_id)) {
            Ok(raw) => Ok(serde_json::from_str(&raw).unwrap_or_default()),
            Err(err) if err.kind() == io::ErrorKind::NotFound => Ok(Vec::new()),
            Err(err) => Err(err.into()),
        }
    }

    fn write_task_timers(
        &self,
        focus_id: &str,
        timers: &[Option<FocusTimer>],
    ) -> Result<(), FocusStoreError> {
        let path = self.task_timers_json(focus_id);
        if timers.iter().all(Option::is_none) {
            return match fs::remove_file(path) {
                Ok(()) => Ok(()),
                Err(err) if err.kind() == io::ErrorKind::NotFound => Ok(()),
                Err(err) => Err(err.into()),
            };
        }
        let bytes = serde_json::to_vec(timers).map_err(io::Error::other)?;
        atomic_write(&path, &bytes)?;
        Ok(())
    }

    fn remove_task_timer_index(&self, focus_id: &str, index: usize) -> Result<(), FocusStoreError> {
        let mut timers = self.read_task_timers(focus_id)?;
        if index < timers.len() {
            timers.remove(index);
            self.write_task_timers(focus_id, &timers)?;
        }
        Ok(())
    }

    fn clear_expired_timer(&self, focus_id: &str) -> Result<(), FocusStoreError> {
        let path = self.timer_json(focus_id);
        let raw = match fs::read_to_string(&path) {
            Ok(raw) => raw,
            Err(err) if err.kind() == io::ErrorKind::NotFound => return Ok(()),
            Err(err) => return Err(err.into()),
        };
        let Ok(timer) = serde_json::from_str::<FocusTimer>(&raw) else {
            return Ok(());
        };
        if timer.status != TimerStatus::Expired {
            return Ok(());
        }
        match fs::remove_file(&path) {
            Ok(()) => Ok(()),
            Err(err) if err.kind() == io::ErrorKind::NotFound => Ok(()),
            Err(err) => Err(err.into()),
        }
    }
}

fn map_document_error(focus_id: &str, error: FocusDocumentError) -> FocusStoreError {
    match error {
        FocusDocumentError::TaskIndexOutOfRange { index } => FocusStoreError::TaskIndexOutOfRange {
            focus_id: focus_id.to_string(),
            index,
        },
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
    fn append_task_clears_expired_timer_sidecar() {
        let dir = TempDir::new().unwrap();
        write_focus(dir.path(), "a", &focus_md("a", &["existing"]));
        let timer_path = dir.path().join("a/timer.json");
        fs::write(
            &timer_path,
            serde_json::to_vec(&FocusTimer {
                duration_secs: 60,
                started_at: 1_000,
                status: TimerStatus::Expired,
            })
            .unwrap(),
        )
        .unwrap();
        let store = MarkdownFocusStore::new(dir.path());

        store.append_task("a", "revive me").unwrap();

        assert!(!timer_path.exists());
        let focuses = store.list().unwrap();
        assert!(focuses[0].timer.is_none());
        assert_eq!(focuses[0].tasks.len(), 2);
    }

    #[test]
    fn append_task_preserves_running_timer_sidecar() {
        let dir = TempDir::new().unwrap();
        write_focus(dir.path(), "a", &focus_md("a", &["existing"]));
        let timer_path = dir.path().join("a/timer.json");
        fs::write(
            &timer_path,
            serde_json::to_vec(&FocusTimer {
                duration_secs: 60,
                started_at: 1_000,
                status: TimerStatus::Running,
            })
            .unwrap(),
        )
        .unwrap();
        let store = MarkdownFocusStore::new(dir.path());

        store.append_task("a", "keep timer").unwrap();

        assert!(timer_path.exists());
        let focuses = store.list().unwrap();
        assert_eq!(
            focuses[0].timer.as_ref().map(|timer| &timer.status),
            Some(&TimerStatus::Running)
        );
    }

    #[test]
    fn append_task_keeps_success_when_timer_cleanup_fails() {
        let dir = TempDir::new().unwrap();
        write_focus(dir.path(), "a", &focus_md("a", &["existing"]));
        fs::create_dir(dir.path().join("a/timer.json")).unwrap();
        let store = MarkdownFocusStore::new(dir.path());

        store.append_task("a", "still appended").unwrap();

        let content = fs::read_to_string(dir.path().join("a/focus.md")).unwrap();
        assert!(content.contains("- [ ] still appended"));
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
    fn delete_task_keeps_success_when_task_timer_cleanup_fails() {
        let dir = TempDir::new().unwrap();
        write_focus(dir.path(), "a", &focus_md("a", &["one", "two"]));
        fs::create_dir(dir.path().join("a/task-timers.json")).unwrap();
        let store = MarkdownFocusStore::new(dir.path());

        store.delete_task("a", 1).unwrap();

        let content = fs::read_to_string(dir.path().join("a/focus.md")).unwrap();
        assert!(content.contains("- [ ] one"));
        assert!(!content.contains("- [ ] two"));
    }

    #[test]
    fn create_focus_writes_frontmatter_and_returns_slug() {
        let dir = TempDir::new().unwrap();
        let store = MarkdownFocusStore::new(dir.path());
        let slug = store
            .create_focus(
                &NewFocus::new("Customer X bug", "ship it").unwrap(),
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
                &NewFocus::new("Customer X bug", "").unwrap(),
                "id-2",
                "2026-04-30T12:00:00Z",
                None,
            )
            .unwrap_err();
        assert!(matches!(err, FocusStoreError::AlreadyExists(slug) if slug == "customer-x-bug"));
    }

    #[test]
    fn toggle_task_marks_done() {
        let dir = TempDir::new().unwrap();
        write_focus(dir.path(), "a", &focus_md("a", &["one", "two"]));
        let store = MarkdownFocusStore::new(dir.path());

        store.toggle_task("a", 0, true).unwrap();

        let content = fs::read_to_string(dir.path().join("a/focus.md")).unwrap();
        assert!(content.contains("- [x] one"));
        assert!(content.contains("- [ ] two"));
    }

    #[test]
    fn toggle_task_marks_undone() {
        let dir = TempDir::new().unwrap();
        let body = "---\nid: a\ntitle: A\ndescription:\ncreated_at: 2026-04-30T12:00:00Z\n---\n- [x] done\n";
        write_focus(dir.path(), "a", body);
        let store = MarkdownFocusStore::new(dir.path());

        store.toggle_task("a", 0, false).unwrap();

        let content = fs::read_to_string(dir.path().join("a/focus.md")).unwrap();
        assert!(content.contains("- [ ] done"));
        assert!(!content.contains("- [x] done"));
    }

    #[test]
    fn toggle_task_errors_on_index_out_of_range() {
        let dir = TempDir::new().unwrap();
        write_focus(dir.path(), "a", &focus_md("a", &["only"]));
        let store = MarkdownFocusStore::new(dir.path());
        let err = store.toggle_task("a", 9, true).unwrap_err();
        assert!(matches!(
            err,
            FocusStoreError::TaskIndexOutOfRange { index: 9, .. }
        ));
    }

    #[test]
    fn update_task_replaces_text_preserving_state() {
        let dir = TempDir::new().unwrap();
        let body = "---\nid: a\ntitle: A\ndescription:\ncreated_at: 2026-04-30T12:00:00Z\n---\n- [ ] one\n- [x] two\n- [ ] three\n";
        write_focus(dir.path(), "a", body);
        let store = MarkdownFocusStore::new(dir.path());

        store.update_task("a", 1, "TWO RENAMED").unwrap();

        let content = fs::read_to_string(dir.path().join("a/focus.md")).unwrap();
        assert!(content.contains("- [ ] one"));
        assert!(content.contains("- [x] TWO RENAMED"));
        assert!(content.contains("- [ ] three"));
        assert!(!content.contains("- [x] two"));
    }

    #[test]
    fn update_task_errors_on_index_out_of_range() {
        let dir = TempDir::new().unwrap();
        write_focus(dir.path(), "a", &focus_md("a", &["one"]));
        let store = MarkdownFocusStore::new(dir.path());
        let err = store.update_task("a", 5, "x").unwrap_err();
        assert!(matches!(
            err,
            FocusStoreError::TaskIndexOutOfRange { index: 5, .. }
        ));
    }

    #[test]
    fn rename_focus_updates_only_title() {
        let dir = TempDir::new().unwrap();
        write_focus(
            dir.path(),
            "a",
            &fixture("a", "Old Title", "2026-04-30T12:00:00Z", &["one", "two"]),
        );
        let store = MarkdownFocusStore::new(dir.path());

        store.rename_focus("a", "New Title").unwrap();

        let focuses = store.list().unwrap();
        assert_eq!(focuses.len(), 1);
        assert_eq!(focuses[0].title, "New Title");
        assert_eq!(focuses[0].id, adhd_ranch_domain::FocusId("a".into()));
        assert_eq!(focuses[0].tasks.len(), 2);
        assert!(dir.path().join("a/focus.md").is_file());
    }

    #[test]
    fn rename_focus_errors_on_missing() {
        let dir = TempDir::new().unwrap();
        let store = MarkdownFocusStore::new(dir.path());
        let err = store.rename_focus("ghost", "x").unwrap_err();
        assert!(matches!(err, FocusStoreError::NotFound(slug) if slug == "ghost"));
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

    // Issue 035: direct unit-test coverage for the create/list/delete/task
    // mutation cycle and timer sidecar edge cases. Names mirror the spec's
    // acceptance table; each test exercises only public store API.

    #[test]
    fn create_then_list_roundtrip() {
        let dir = TempDir::new().unwrap();
        let store = MarkdownFocusStore::new(dir.path());

        let slug = store
            .create_focus(
                &NewFocus::new("Customer X bug", "ship it").unwrap(),
                "id-1",
                "2026-04-30T12:00:00Z",
                None,
            )
            .unwrap();

        let focuses = store.list().unwrap();

        assert_eq!(focuses.len(), 1);
        let f = &focuses[0];
        assert_eq!(f.id, adhd_ranch_domain::FocusId(slug));
        assert_eq!(f.title, "Customer X bug");
        assert_eq!(f.description, "ship it");
        assert_eq!(f.created_at, "2026-04-30T12:00:00Z");
        assert!(f.timer.is_none());
    }

    #[test]
    fn list_with_timer_sidecar() {
        let dir = TempDir::new().unwrap();
        let store = MarkdownFocusStore::new(dir.path());
        let timer = FocusTimer {
            duration_secs: 240,
            started_at: 1_700_000_000,
            status: adhd_ranch_domain::TimerStatus::Running,
        };

        store
            .create_focus(
                &NewFocus::new("With timer", "").unwrap(),
                "id-1",
                "2026-04-30T12:00:00Z",
                Some(timer.clone()),
            )
            .unwrap();

        let focuses = store.list().unwrap();
        assert_eq!(focuses.len(), 1);
        assert_eq!(focuses[0].timer, Some(timer));
    }

    #[test]
    fn update_timer_persists_new_status() {
        let dir = TempDir::new().unwrap();
        let store = MarkdownFocusStore::new(dir.path());
        let timer = FocusTimer {
            duration_secs: 60,
            started_at: 1_000,
            status: adhd_ranch_domain::TimerStatus::Running,
        };
        let slug = store
            .create_focus(
                &NewFocus::new("With timer", "").unwrap(),
                "id-1",
                "2026-04-30T12:00:00Z",
                Some(timer.clone()),
            )
            .unwrap();

        let expired = FocusTimer {
            status: adhd_ranch_domain::TimerStatus::Expired,
            ..timer
        };
        store.update_timer(&slug, &expired).unwrap();

        let focuses = store.list().unwrap();
        assert_eq!(
            focuses[0].timer.as_ref().unwrap().status,
            adhd_ranch_domain::TimerStatus::Expired
        );
    }

    #[test]
    fn update_timer_missing_focus_returns_not_found() {
        let dir = TempDir::new().unwrap();
        let store = MarkdownFocusStore::new(dir.path());
        let timer = FocusTimer {
            duration_secs: 60,
            started_at: 0,
            status: adhd_ranch_domain::TimerStatus::Running,
        };
        let err = store.update_timer("does-not-exist", &timer).unwrap_err();
        assert!(matches!(err, FocusStoreError::NotFound(_)));
    }

    #[test]
    fn clear_timer_removes_timer_sidecar() {
        let dir = TempDir::new().unwrap();
        let store = MarkdownFocusStore::new(dir.path());
        let timer = FocusTimer {
            duration_secs: 60,
            started_at: 1_000,
            status: adhd_ranch_domain::TimerStatus::Running,
        };
        let slug = store
            .create_focus(
                &NewFocus::new("With timer", "").unwrap(),
                "id-1",
                "2026-04-30T12:00:00Z",
                Some(timer),
            )
            .unwrap();

        store.clear_timer(&slug).unwrap();

        let focuses = store.list().unwrap();
        assert!(focuses[0].timer.is_none());
    }

    #[test]
    fn task_timer_sidecar_maps_to_task_by_index() {
        let dir = TempDir::new().unwrap();
        write_focus(dir.path(), "a", &focus_md("a", &["one", "two"]));
        let store = MarkdownFocusStore::new(dir.path());
        let timer = FocusTimer {
            duration_secs: 240,
            started_at: 1_700_000_000,
            status: adhd_ranch_domain::TimerStatus::Running,
        };

        store.update_task_timer("a", 1, &timer).unwrap();

        let focuses = store.list().unwrap();
        assert!(focuses[0].tasks[0].timer.is_none());
        assert_eq!(focuses[0].tasks[1].timer, Some(timer));
    }

    #[test]
    fn update_task_timer_persists_expired_status_by_focus_and_task_index() {
        let dir = TempDir::new().unwrap();
        write_focus(dir.path(), "a", &focus_md("a", &["one", "two"]));
        let store = MarkdownFocusStore::new(dir.path());
        let running = FocusTimer {
            duration_secs: 240,
            started_at: 1_700_000_000,
            status: adhd_ranch_domain::TimerStatus::Running,
        };
        store.update_task_timer("a", 1, &running).unwrap();

        let expired = FocusTimer {
            status: adhd_ranch_domain::TimerStatus::Expired,
            ..running
        };
        store.update_task_timer("a", 1, &expired).unwrap();

        let focuses = store.list().unwrap();
        assert!(focuses[0].tasks[0].timer.is_none());
        assert_eq!(focuses[0].tasks[1].timer, Some(expired));
    }

    #[test]
    fn update_task_timer_errors_when_index_out_of_range() {
        let dir = TempDir::new().unwrap();
        write_focus(dir.path(), "a", &focus_md("a", &["one"]));
        let store = MarkdownFocusStore::new(dir.path());
        let timer = FocusTimer {
            duration_secs: 240,
            started_at: 1_700_000_000,
            status: adhd_ranch_domain::TimerStatus::Expired,
        };

        let err = store.update_task_timer("a", 1, &timer).unwrap_err();

        assert!(matches!(
            err,
            FocusStoreError::TaskIndexOutOfRange {
                focus_id,
                index: 1
            } if focus_id == "a"
        ));
    }

    #[test]
    fn delete_task_removes_matching_task_timer_index() {
        let dir = TempDir::new().unwrap();
        write_focus(dir.path(), "a", &focus_md("a", &["one", "two"]));
        let store = MarkdownFocusStore::new(dir.path());
        let timer = FocusTimer {
            duration_secs: 240,
            started_at: 1_700_000_000,
            status: adhd_ranch_domain::TimerStatus::Running,
        };
        store.update_task_timer("a", 0, &timer).unwrap();

        store.delete_task("a", 0).unwrap();

        let focuses = store.list().unwrap();
        assert_eq!(focuses[0].tasks.len(), 1);
        assert!(focuses[0].tasks[0].timer.is_none());
    }

    #[test]
    fn list_without_timer_sidecar() {
        let dir = TempDir::new().unwrap();
        let store = MarkdownFocusStore::new(dir.path());

        store
            .create_focus(
                &NewFocus::new("No timer", "").unwrap(),
                "id-1",
                "2026-04-30T12:00:00Z",
                None,
            )
            .unwrap();

        let focuses = store.list().unwrap();
        assert_eq!(focuses.len(), 1);
        assert!(focuses[0].timer.is_none());
    }

    #[test]
    fn delete_removes_directory() {
        let dir = TempDir::new().unwrap();
        let store = MarkdownFocusStore::new(dir.path());
        let slug = store
            .create_focus(
                &NewFocus::new("Bye", "").unwrap(),
                "id-1",
                "2026-04-30T12:00:00Z",
                None,
            )
            .unwrap();
        assert!(dir.path().join(&slug).is_dir());

        store.delete_focus(&slug).unwrap();

        assert!(!dir.path().join(&slug).exists());
    }

    #[test]
    fn delete_nonexistent_returns_err() {
        let dir = TempDir::new().unwrap();
        let store = MarkdownFocusStore::new(dir.path());
        let err = store.delete_focus("ghost").unwrap_err();
        assert!(matches!(err, FocusStoreError::NotFound(slug) if slug == "ghost"));
    }

    #[test]
    fn corrupted_timer_json_degrades_gracefully() {
        let dir = TempDir::new().unwrap();
        let store = MarkdownFocusStore::new(dir.path());
        let slug = store
            .create_focus(
                &NewFocus::new("Broken timer", "").unwrap(),
                "id-1",
                "2026-04-30T12:00:00Z",
                None,
            )
            .unwrap();
        let timer_path = dir.path().join(&slug).join("timer.json");
        fs::write(&timer_path, b"{ this is not valid json").unwrap();

        let focuses = store.list().unwrap();

        assert_eq!(focuses.len(), 1);
        assert_eq!(focuses[0].title, "Broken timer");
        assert!(focuses[0].timer.is_none());
    }

    #[test]
    fn append_task_persists() {
        let dir = TempDir::new().unwrap();
        let store = MarkdownFocusStore::new(dir.path());
        let slug = store
            .create_focus(
                &NewFocus::new("Has tasks", "").unwrap(),
                "id-1",
                "2026-04-30T12:00:00Z",
                None,
            )
            .unwrap();

        store.append_task(&slug, "first thing").unwrap();

        let focuses = store.list().unwrap();
        assert_eq!(focuses.len(), 1);
        assert_eq!(focuses[0].tasks.len(), 1);
        assert_eq!(focuses[0].tasks[0].text, "first thing");
    }

    #[test]
    fn delete_task_persists() {
        let dir = TempDir::new().unwrap();
        let store = MarkdownFocusStore::new(dir.path());
        let slug = store
            .create_focus(
                &NewFocus::new("Two tasks", "").unwrap(),
                "id-1",
                "2026-04-30T12:00:00Z",
                None,
            )
            .unwrap();
        store.append_task(&slug, "keep me").unwrap();
        store.append_task(&slug, "remove me").unwrap();

        store.delete_task(&slug, 1).unwrap();

        let focuses = store.list().unwrap();
        assert_eq!(focuses[0].tasks.len(), 1);
        assert_eq!(focuses[0].tasks[0].text, "keep me");
    }
}
