use std::sync::Mutex;

use adhd_ranch_domain::{Focus, FocusTimer, TimerOwner};

use crate::focus_store::FocusStoreError;

/// The Timer half of the Focus directory: read every Focus, write one Timer.
/// Callers name what the Timer belongs to; where it lands on disk is ours.
pub trait TimerStore: Send + Sync {
    /// Every Focus, with its Timers, so expiry can see what is due.
    fn focuses(&self) -> Result<Vec<Focus>, FocusStoreError>;
    fn write_timer(
        &self,
        owner: &TimerOwner,
        timer: Option<&FocusTimer>,
    ) -> Result<(), FocusStoreError>;
}

/// Second adapter at the TimerStore seam: keeps Focuses in memory so timer
/// use cases can be tested without touching the file system.
pub struct InMemoryTimerStore {
    focuses: Mutex<Vec<Focus>>,
    writes: Mutex<Vec<(TimerOwner, Option<FocusTimer>)>>,
}

impl InMemoryTimerStore {
    pub fn new(focuses: Vec<Focus>) -> Self {
        Self {
            focuses: Mutex::new(focuses),
            writes: Mutex::new(Vec::new()),
        }
    }

    /// Every write the store has taken, oldest first.
    pub fn writes(&self) -> Vec<(TimerOwner, Option<FocusTimer>)> {
        self.writes.lock().unwrap().clone()
    }

    /// The Focuses as they stand after the writes so far.
    pub fn snapshot(&self) -> Vec<Focus> {
        self.focuses.lock().unwrap().clone()
    }
}

impl TimerStore for InMemoryTimerStore {
    fn focuses(&self) -> Result<Vec<Focus>, FocusStoreError> {
        Ok(self.focuses.lock().unwrap().clone())
    }

    fn write_timer(
        &self,
        owner: &TimerOwner,
        timer: Option<&FocusTimer>,
    ) -> Result<(), FocusStoreError> {
        let mut focuses = self.focuses.lock().unwrap();
        let focus = focuses
            .iter_mut()
            .find(|focus| focus.id.0 == owner.focus_id())
            .ok_or_else(|| FocusStoreError::NotFound(owner.focus_id().to_string()))?;

        match owner {
            TimerOwner::Focus { .. } => focus.timer = timer.cloned(),
            TimerOwner::Task { focus_id, index } => {
                let task = focus.tasks.get_mut(*index).ok_or_else(|| {
                    FocusStoreError::TaskIndexOutOfRange {
                        focus_id: focus_id.clone(),
                        index: *index,
                    }
                })?;
                task.timer = timer.cloned();
            }
        }

        self.writes
            .lock()
            .unwrap()
            .push((owner.clone(), timer.cloned()));
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use adhd_ranch_domain::{FocusId, Task, TimerStatus};

    use super::*;

    fn timer() -> FocusTimer {
        FocusTimer {
            duration_secs: 120,
            started_at: 1_000,
            status: TimerStatus::Running,
        }
    }

    fn focus_with_tasks(id: &str, tasks: usize) -> Focus {
        Focus {
            id: FocusId(id.to_string()),
            title: id.to_string(),
            description: String::new(),
            created_at: String::new(),
            tasks: (0..tasks)
                .map(|i| Task {
                    id: format!("task-{i}"),
                    text: format!("task {i}"),
                    done: false,
                    timer: None,
                })
                .collect(),
            timer: None,
        }
    }

    #[test]
    fn writing_a_focus_timer_updates_the_focus() {
        let store = InMemoryTimerStore::new(vec![focus_with_tasks("a", 0)]);

        store
            .write_timer(&TimerOwner::focus("a"), Some(&timer()))
            .unwrap();

        assert_eq!(store.snapshot()[0].timer, Some(timer()));
    }

    #[test]
    fn writing_none_clears_the_timer() {
        let store = InMemoryTimerStore::new(vec![focus_with_tasks("a", 0)]);
        store
            .write_timer(&TimerOwner::focus("a"), Some(&timer()))
            .unwrap();

        store.write_timer(&TimerOwner::focus("a"), None).unwrap();

        assert_eq!(store.snapshot()[0].timer, None);
    }

    #[test]
    fn writing_a_task_timer_updates_that_task_only() {
        let store = InMemoryTimerStore::new(vec![focus_with_tasks("a", 2)]);

        store
            .write_timer(&TimerOwner::task("a", 1), Some(&timer()))
            .unwrap();

        let focus = &store.snapshot()[0];
        assert_eq!(focus.tasks[0].timer, None);
        assert_eq!(focus.tasks[1].timer, Some(timer()));
    }

    #[test]
    fn writing_a_timer_for_an_unknown_focus_is_not_found() {
        let store = InMemoryTimerStore::new(Vec::new());

        let error = store
            .write_timer(&TimerOwner::focus("missing"), Some(&timer()))
            .unwrap_err();

        assert!(matches!(error, FocusStoreError::NotFound(_)));
    }

    #[test]
    fn writing_a_timer_past_the_last_task_is_out_of_range() {
        let store = InMemoryTimerStore::new(vec![focus_with_tasks("a", 1)]);

        let error = store
            .write_timer(&TimerOwner::task("a", 5), Some(&timer()))
            .unwrap_err();

        assert!(matches!(
            error,
            FocusStoreError::TaskIndexOutOfRange { index: 5, .. }
        ));
    }
}
