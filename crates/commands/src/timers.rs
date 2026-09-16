use std::sync::Arc;

use adhd_ranch_domain::{
    tick, FocusTimer, NotificationSource, TaskTimerExpiredSource, TimerExpiredSource, TimerOwner,
    TimerPreset, TimerStatus, TimerTransitionTarget,
};
use adhd_ranch_storage::TimerStore;

use crate::{ClockSecs, CommandError, SettingsProvider};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NotificationRequest {
    pub source_key: String,
    pub title: String,
    pub body: String,
}

pub trait NotificationSink: Send + Sync {
    fn notify(&self, request: NotificationRequest);
}

/// Every Timer rule lives here: starting, clearing, reviving a Focus whose
/// Timer expired, and turning Running into Expired once the countdown is up.
/// Owners are data, so Focus and Task Timers take the same path.
pub struct Timers {
    store: Arc<dyn TimerStore>,
    clock_secs: ClockSecs,
    settings: SettingsProvider,
    notifications: Arc<dyn NotificationSink>,
}

impl Timers {
    pub fn new(
        store: Arc<dyn TimerStore>,
        clock_secs: ClockSecs,
        settings: SettingsProvider,
        notifications: Arc<dyn NotificationSink>,
    ) -> Self {
        Self {
            store,
            clock_secs,
            settings,
            notifications,
        }
    }

    /// A Timer running from now for the preset's duration.
    pub(crate) fn running(&self, preset: &TimerPreset) -> FocusTimer {
        FocusTimer {
            duration_secs: preset.duration_secs(),
            started_at: (self.clock_secs)(),
            status: TimerStatus::Running,
        }
    }

    pub fn start(&self, owner: &TimerOwner, preset: &TimerPreset) -> Result<(), CommandError> {
        let timer = self.running(preset);
        self.store.write_timer(owner, Some(&timer))?;
        Ok(())
    }

    pub fn clear(&self, owner: &TimerOwner) -> Result<(), CommandError> {
        self.store.write_timer(owner, None)?;
        Ok(())
    }

    /// Adding a Task revives its Focus: an expired Focus Timer is cleared so the
    /// Animal goes back to normal. Tasks are never revived.
    pub fn revive_if_expired(&self, focus_id: &str) -> Result<(), CommandError> {
        let focuses = self.store.focuses()?;
        let expired = focuses
            .iter()
            .find(|focus| focus.id.0 == focus_id)
            .and_then(|focus| focus.timer.as_ref())
            .is_some_and(|timer| matches!(timer.status, TimerStatus::Expired));

        if expired {
            self.clear(&TimerOwner::focus(focus_id))?;
        }
        Ok(())
    }

    /// Persists every Timer whose countdown ran out and notifies through the
    /// enabled notification sources. Persisted Expired status is the dedup lock,
    /// so a Timer only fires once.
    pub fn expire_due(&self, now_secs: i64) -> Result<(), CommandError> {
        let focuses = self.store.focuses()?;
        let settings = self.settings.get();

        for transition in tick(now_secs, &focuses) {
            let Some(focus) = focuses.iter().find(|f| f.id.0 == transition.focus_id) else {
                continue;
            };

            let (owner, running_timer, request) = match &transition.target {
                TimerTransitionTarget::Focus => {
                    let Some(timer) = focus.timer.as_ref() else {
                        continue;
                    };
                    (
                        TimerOwner::focus(&transition.focus_id),
                        timer,
                        NotificationRequest {
                            source_key: TimerExpiredSource.key().to_string(),
                            title: "Timer expired".to_string(),
                            body: format!("{} reached its timer.", transition.focus_title),
                        },
                    )
                }
                TimerTransitionTarget::Task { index, text } => {
                    let Some(timer) = focus.tasks.get(*index).and_then(|task| task.timer.as_ref())
                    else {
                        continue;
                    };
                    (
                        TimerOwner::task(&transition.focus_id, *index),
                        timer,
                        NotificationRequest {
                            source_key: TaskTimerExpiredSource.key().to_string(),
                            title: "Task timer expired".to_string(),
                            body: format!("{}: {text}", transition.focus_title),
                        },
                    )
                }
            };

            self.store
                .write_timer(&owner, Some(&expired(running_timer)))?;

            let enabled = match transition.target {
                TimerTransitionTarget::Focus => {
                    settings.notifications.is_enabled(&TimerExpiredSource)
                }
                TimerTransitionTarget::Task { .. } => {
                    settings.notifications.is_enabled(&TaskTimerExpiredSource)
                }
            };
            if enabled {
                self.notifications.notify(request);
            }
        }
        Ok(())
    }
}

fn expired(timer: &FocusTimer) -> FocusTimer {
    FocusTimer {
        status: TimerStatus::Expired,
        ..timer.clone()
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Mutex;

    use adhd_ranch_domain::{
        Focus, FocusId, NotificationSettings, Settings, Task, TimerPreset, TimerStatus,
    };
    use adhd_ranch_storage::InMemoryTimerStore;

    use super::*;

    struct RecordingSink {
        requests: Mutex<Vec<NotificationRequest>>,
    }

    impl RecordingSink {
        fn new() -> Self {
            Self {
                requests: Mutex::new(Vec::new()),
            }
        }

        fn requests(&self) -> Vec<NotificationRequest> {
            self.requests.lock().unwrap().clone()
        }
    }

    impl NotificationSink for RecordingSink {
        fn notify(&self, request: NotificationRequest) {
            self.requests.lock().unwrap().push(request);
        }
    }

    fn notifications_with(source: &dyn NotificationSource, enabled: bool) -> NotificationSettings {
        let mut settings = NotificationSettings::default();
        settings.set(source, enabled);
        settings
    }

    fn timer(status: TimerStatus, duration_secs: u64, started_at: i64) -> FocusTimer {
        FocusTimer {
            duration_secs,
            started_at,
            status,
        }
    }

    fn focus(id: &str, timer: Option<FocusTimer>, tasks: Vec<Task>) -> Focus {
        Focus {
            id: FocusId(id.to_string()),
            title: format!("Focus {id}"),
            description: String::new(),
            created_at: String::new(),
            tasks,
            timer,
        }
    }

    fn task(text: &str, timer: Option<FocusTimer>) -> Task {
        Task {
            id: text.to_string(),
            text: text.to_string(),
            done: false,
            timer,
        }
    }

    struct Fixture {
        timers: Timers,
        store: Arc<InMemoryTimerStore>,
        sink: Arc<RecordingSink>,
    }

    fn fixture_with(focuses: Vec<Focus>, notifications: NotificationSettings, now: i64) -> Fixture {
        let store = Arc::new(InMemoryTimerStore::new(focuses));
        let sink = Arc::new(RecordingSink::new());
        let settings = Settings {
            notifications,
            ..Settings::default()
        };
        let timers = Timers::new(
            store.clone(),
            Arc::new(move || now),
            Arc::new(move || settings.clone()),
            sink.clone(),
        );
        Fixture {
            timers,
            store,
            sink,
        }
    }

    fn fixture(focuses: Vec<Focus>) -> Fixture {
        fixture_with(focuses, NotificationSettings::default(), 1_000)
    }

    #[test]
    fn start_writes_a_running_timer_for_the_preset_duration() {
        let f = fixture(vec![focus("a", None, Vec::new())]);

        f.timers
            .start(&TimerOwner::focus("a"), &TimerPreset::Two)
            .unwrap();

        assert_eq!(
            f.store.snapshot()[0].timer,
            Some(timer(TimerStatus::Running, 120, 1_000))
        );
    }

    #[test]
    fn start_takes_the_same_path_for_a_task() {
        let f = fixture(vec![focus("a", None, vec![task("write tests", None)])]);

        f.timers
            .start(&TimerOwner::task("a", 0), &TimerPreset::Four)
            .unwrap();

        assert_eq!(
            f.store.snapshot()[0].tasks[0].timer,
            Some(timer(TimerStatus::Running, 240, 1_000))
        );
    }

    #[test]
    fn clear_removes_the_timer() {
        let f = fixture(vec![focus(
            "a",
            Some(timer(TimerStatus::Running, 120, 900)),
            Vec::new(),
        )]);

        f.timers.clear(&TimerOwner::focus("a")).unwrap();

        assert_eq!(f.store.snapshot()[0].timer, None);
    }

    #[test]
    fn revive_clears_an_expired_focus_timer() {
        let f = fixture(vec![focus(
            "a",
            Some(timer(TimerStatus::Expired, 120, 500)),
            Vec::new(),
        )]);

        f.timers.revive_if_expired("a").unwrap();

        assert_eq!(f.store.snapshot()[0].timer, None);
    }

    #[test]
    fn revive_leaves_a_running_focus_timer_alone() {
        let running = timer(TimerStatus::Running, 600, 900);
        let f = fixture(vec![focus("a", Some(running.clone()), Vec::new())]);

        f.timers.revive_if_expired("a").unwrap();

        assert_eq!(f.store.snapshot()[0].timer, Some(running));
        assert!(f.store.writes().is_empty());
    }

    #[test]
    fn revive_ignores_an_unknown_focus() {
        let f = fixture(Vec::new());

        f.timers.revive_if_expired("missing").unwrap();

        assert!(f.store.writes().is_empty());
    }

    #[test]
    fn expire_due_persists_expired_status_for_a_focus() {
        let f = fixture(vec![focus(
            "a",
            Some(timer(TimerStatus::Running, 60, 900)),
            Vec::new(),
        )]);

        f.timers.expire_due(1_000).unwrap();

        assert_eq!(
            f.store.snapshot()[0].timer,
            Some(timer(TimerStatus::Expired, 60, 900))
        );
    }

    #[test]
    fn expire_due_persists_expired_status_for_a_task() {
        let f = fixture(vec![focus(
            "a",
            None,
            vec![task(
                "write tests",
                Some(timer(TimerStatus::Running, 60, 900)),
            )],
        )]);

        f.timers.expire_due(1_000).unwrap();

        assert_eq!(
            f.store.snapshot()[0].tasks[0].timer,
            Some(timer(TimerStatus::Expired, 60, 900))
        );
    }

    #[test]
    fn expire_due_notifies_once_per_expiry() {
        let f = fixture(vec![focus(
            "a",
            Some(timer(TimerStatus::Running, 60, 900)),
            Vec::new(),
        )]);

        f.timers.expire_due(1_000).unwrap();
        f.timers.expire_due(1_001).unwrap();

        assert_eq!(f.sink.requests().len(), 1);
    }

    #[test]
    fn expire_due_skips_notification_when_the_source_is_disabled() {
        let f = fixture_with(
            vec![focus(
                "a",
                Some(timer(TimerStatus::Running, 60, 900)),
                Vec::new(),
            )],
            notifications_with(&TimerExpiredSource, false),
            1_000,
        );

        f.timers.expire_due(1_000).unwrap();

        assert!(f.sink.requests().is_empty());
        assert_eq!(
            f.store.snapshot()[0].timer,
            Some(timer(TimerStatus::Expired, 60, 900))
        );
    }

    #[test]
    fn expire_due_uses_the_task_source_for_a_task_timer() {
        let f = fixture_with(
            vec![focus(
                "a",
                None,
                vec![task(
                    "write tests",
                    Some(timer(TimerStatus::Running, 60, 900)),
                )],
            )],
            notifications_with(&TaskTimerExpiredSource, false),
            1_000,
        );

        f.timers.expire_due(1_000).unwrap();

        assert!(f.sink.requests().is_empty());
    }

    #[test]
    fn expire_due_leaves_running_timers_alone() {
        let f = fixture(vec![focus(
            "a",
            Some(timer(TimerStatus::Running, 600, 900)),
            Vec::new(),
        )]);

        f.timers.expire_due(1_000).unwrap();

        assert!(f.store.writes().is_empty());
        assert!(f.sink.requests().is_empty());
    }
}
