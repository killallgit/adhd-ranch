use std::path::Path;
use std::time::Duration;

use notify::RecursiveMode;
use notify_debouncer_mini::{new_debouncer, DebounceEventResult, Debouncer};

#[derive(Debug)]
pub enum WatcherError {
    Notify(notify::Error),
}

impl std::fmt::Display for WatcherError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Notify(error) => write!(f, "watcher error: {error}"),
        }
    }
}

impl std::error::Error for WatcherError {}

impl From<notify::Error> for WatcherError {
    fn from(error: notify::Error) -> Self {
        Self::Notify(error)
    }
}

pub struct FocusWatcher {
    _debouncer: Debouncer<notify::RecommendedWatcher>,
}

pub fn watch_path<P, F>(
    root: P,
    debounce: Duration,
    mut on_change: F,
) -> Result<FocusWatcher, WatcherError>
where
    P: AsRef<Path>,
    F: FnMut() + Send + 'static,
{
    let mut debouncer = new_debouncer(debounce, move |events: DebounceEventResult| {
        if let Ok(events) = events {
            if !events.is_empty() {
                on_change();
            }
        }
    })?;

    debouncer
        .watcher()
        .watch(root.as_ref(), RecursiveMode::Recursive)?;

    Ok(FocusWatcher {
        _debouncer: debouncer,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;
    use std::thread::sleep;
    use tempfile::TempDir;

    #[test]
    fn fires_callback_when_a_file_changes_under_root() {
        let dir = TempDir::new().unwrap();
        let counter = Arc::new(AtomicUsize::new(0));
        let counter_in_cb = counter.clone();

        let _watcher = watch_path(dir.path(), Duration::from_millis(50), move || {
            counter_in_cb.fetch_add(1, Ordering::SeqCst);
        })
        .unwrap();

        let path = dir.path().join("focus.md");
        fs::write(&path, "first").unwrap();

        let deadline = std::time::Instant::now() + Duration::from_secs(2);
        while counter.load(Ordering::SeqCst) == 0 && std::time::Instant::now() < deadline {
            sleep(Duration::from_millis(25));
        }

        assert!(
            counter.load(Ordering::SeqCst) >= 1,
            "expected at least one change event"
        );
    }
}
