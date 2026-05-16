use std::sync::Arc;

use adhd_ranch_domain::{
    tick, FocusTimer, NotificationSource, Settings, TaskTimerExpiredSource, TimerExpiredSource,
    TimerStatus, TimerTransitionTarget,
};
use adhd_ranch_storage::FocusStore;

use crate::CommandError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NotificationRequest {
    pub source_key: String,
    pub title: String,
    pub body: String,
}

pub trait NotificationSink: Send + Sync {
    fn notify(&self, request: NotificationRequest);
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TimerExpiryEvent {
    FocusTimerExpired {
        focus_id: String,
        focus_title: String,
    },
    TaskTimerExpired {
        focus_id: String,
        focus_title: String,
        task_index: usize,
        task_text: String,
    },
}

pub struct TimerExpiryWorkflow {
    store: Arc<dyn FocusStore>,
    settings: Settings,
    notifications: Arc<dyn NotificationSink>,
}

impl TimerExpiryWorkflow {
    pub fn new(
        store: Arc<dyn FocusStore>,
        settings: Settings,
        notifications: Arc<dyn NotificationSink>,
    ) -> Self {
        Self {
            store,
            settings,
            notifications,
        }
    }

    pub fn run_once(&self, now_secs: i64) -> Result<Vec<TimerExpiryEvent>, CommandError> {
        let focuses = self.store.list()?;
        let mut events = Vec::new();
        for transition in tick(now_secs, &focuses) {
            let Some(focus) = focuses
                .iter()
                .find(|focus| focus.id.0 == transition.focus_id)
            else {
                continue;
            };

            match transition.target {
                TimerTransitionTarget::Focus => {
                    let Some(running_timer) = focus.timer.as_ref() else {
                        continue;
                    };
                    let expired = expired_timer(running_timer);
                    self.store.update_timer(&transition.focus_id, &expired)?;
                    if self.settings.notifications.is_enabled(&TimerExpiredSource) {
                        self.notifications.notify(NotificationRequest {
                            source_key: TimerExpiredSource.key().to_string(),
                            title: "Timer expired".to_string(),
                            body: format!("{} reached its timer.", transition.focus_title),
                        });
                    }
                    events.push(TimerExpiryEvent::FocusTimerExpired {
                        focus_id: transition.focus_id,
                        focus_title: transition.focus_title,
                    });
                }
                TimerTransitionTarget::Task { index, text } => {
                    let Some(task) = focus.tasks.get(index) else {
                        continue;
                    };
                    let Some(running_timer) = task.timer.as_ref() else {
                        continue;
                    };
                    let expired = expired_timer(running_timer);
                    self.store
                        .update_task_timer(&transition.focus_id, index, &expired)?;
                    if self
                        .settings
                        .notifications
                        .is_enabled(&TaskTimerExpiredSource)
                    {
                        self.notifications.notify(NotificationRequest {
                            source_key: TaskTimerExpiredSource.key().to_string(),
                            title: "Task timer expired".to_string(),
                            body: format!("{}: {text}", transition.focus_title),
                        });
                    }
                    events.push(TimerExpiryEvent::TaskTimerExpired {
                        focus_id: transition.focus_id,
                        focus_title: transition.focus_title,
                        task_index: index,
                        task_text: text,
                    });
                }
            }
        }
        Ok(events)
    }
}

fn expired_timer(timer: &FocusTimer) -> FocusTimer {
    FocusTimer {
        duration_secs: timer.duration_secs,
        started_at: timer.started_at,
        status: TimerStatus::Expired,
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use adhd_ranch_domain::{
        Focus, FocusId, FocusTimer, NewFocus, NotificationSettings, NotificationSource, Settings,
        Task, TaskTimerExpiredSource, TimerExpiredSource, TimerStatus,
    };
    use adhd_ranch_storage::{FocusStore, FocusStoreError};

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

    struct StubStore {
        focuses: Mutex<Vec<Focus>>,
    }

    impl StubStore {
        fn new(focuses: Vec<Focus>) -> Self {
            Self {
                focuses: Mutex::new(focuses),
            }
        }
    }

    impl FocusStore for StubStore {
        fn list(&self) -> Result<Vec<Focus>, FocusStoreError> {
            Ok(self.focuses.lock().unwrap().clone())
        }

        fn create_focus(
            &self,
            _new_focus: &NewFocus,
            _id: &str,
            _created_at: &str,
            _timer: Option<FocusTimer>,
        ) -> Result<String, FocusStoreError> {
            unimplemented!()
        }

        fn delete_focus(&self, _focus_id: &str) -> Result<(), FocusStoreError> {
            unimplemented!()
        }

        fn rename_focus(&self, _focus_id: &str, _title: &str) -> Result<(), FocusStoreError> {
            unimplemented!()
        }

        fn append_task(&self, _focus_id: &str, _text: &str) -> Result<(), FocusStoreError> {
            unimplemented!()
        }

        fn delete_task(&self, _focus_id: &str, _index: usize) -> Result<(), FocusStoreError> {
            unimplemented!()
        }

        fn update_task(
            &self,
            _focus_id: &str,
            _index: usize,
            _text: &str,
        ) -> Result<(), FocusStoreError> {
            unimplemented!()
        }

        fn toggle_task(
            &self,
            _focus_id: &str,
            _index: usize,
            _done: bool,
        ) -> Result<(), FocusStoreError> {
            unimplemented!()
        }

        fn update_timer(&self, focus_id: &str, timer: &FocusTimer) -> Result<(), FocusStoreError> {
            let mut focuses = self.focuses.lock().unwrap();
            let focus = focuses
                .iter_mut()
                .find(|focus| focus.id.0 == focus_id)
                .ok_or_else(|| FocusStoreError::NotFound(focus_id.to_string()))?;
            focus.timer = Some(timer.clone());
            Ok(())
        }

        fn clear_timer(&self, _focus_id: &str) -> Result<(), FocusStoreError> {
            unimplemented!()
        }

        fn update_task_timer(
            &self,
            focus_id: &str,
            index: usize,
            timer: &FocusTimer,
        ) -> Result<(), FocusStoreError> {
            let mut focuses = self.focuses.lock().unwrap();
            let focus = focuses
                .iter_mut()
                .find(|focus| focus.id.0 == focus_id)
                .ok_or_else(|| FocusStoreError::NotFound(focus_id.to_string()))?;
            let task =
                focus
                    .tasks
                    .get_mut(index)
                    .ok_or_else(|| FocusStoreError::TaskIndexOutOfRange {
                        focus_id: focus_id.to_string(),
                        index,
                    })?;
            task.timer = Some(timer.clone());
            Ok(())
        }

        fn clear_task_timer(&self, _focus_id: &str, _index: usize) -> Result<(), FocusStoreError> {
            unimplemented!()
        }
    }

    fn running(duration_secs: u64, started_at: i64) -> FocusTimer {
        FocusTimer {
            duration_secs,
            started_at,
            status: TimerStatus::Running,
        }
    }

    fn settings_with(notifications: NotificationSettings) -> Settings {
        Settings {
            notifications,
            ..Settings::default()
        }
    }

    #[test]
    fn expires_task_timer_and_requests_task_notification() {
        let store = Arc::new(StubStore::new(vec![Focus {
            id: FocusId("focus-1".to_string()),
            title: "Ship feature".to_string(),
            description: String::new(),
            created_at: String::new(),
            tasks: vec![Task {
                id: "task-1".to_string(),
                text: "Write tests".to_string(),
                done: false,
                timer: Some(running(60, 1_000)),
            }],
            timer: None,
        }]));
        let notifications = Arc::new(RecordingSink::new());
        let workflow =
            TimerExpiryWorkflow::new(store.clone(), Settings::default(), notifications.clone());

        let events = workflow.run_once(1_100).unwrap();

        let focuses = store.list().unwrap();
        assert_eq!(
            focuses[0].tasks[0]
                .timer
                .as_ref()
                .map(|timer| &timer.status),
            Some(&TimerStatus::Expired)
        );
        assert_eq!(
            events,
            vec![TimerExpiryEvent::TaskTimerExpired {
                focus_id: "focus-1".to_string(),
                focus_title: "Ship feature".to_string(),
                task_index: 0,
                task_text: "Write tests".to_string(),
            }]
        );
        assert_eq!(
            notifications.requests(),
            vec![NotificationRequest {
                source_key: TaskTimerExpiredSource.key().to_string(),
                title: "Task timer expired".to_string(),
                body: "Ship feature: Write tests".to_string(),
            }]
        );
    }

    #[test]
    fn disabled_task_timer_notification_source_still_persists_expiry() {
        let store = Arc::new(StubStore::new(vec![Focus {
            id: FocusId("focus-1".to_string()),
            title: "Ship feature".to_string(),
            description: String::new(),
            created_at: String::new(),
            tasks: vec![Task {
                id: "task-1".to_string(),
                text: "Write tests".to_string(),
                done: false,
                timer: Some(running(60, 1_000)),
            }],
            timer: None,
        }]));
        let notifications = Arc::new(RecordingSink::new());
        let mut settings = NotificationSettings::default();
        settings.set(&TaskTimerExpiredSource, false);
        let workflow = TimerExpiryWorkflow::new(
            store.clone(),
            settings_with(settings),
            notifications.clone(),
        );

        workflow.run_once(1_100).unwrap();

        let focuses = store.list().unwrap();
        assert_eq!(
            focuses[0].tasks[0]
                .timer
                .as_ref()
                .map(|timer| &timer.status),
            Some(&TimerStatus::Expired)
        );
        assert_eq!(notifications.requests(), vec![]);
    }

    #[test]
    fn focus_timer_expiry_uses_focus_timer_notification_source() {
        let store = Arc::new(StubStore::new(vec![Focus {
            id: FocusId("focus-1".to_string()),
            title: "Ship feature".to_string(),
            description: String::new(),
            created_at: String::new(),
            tasks: Vec::new(),
            timer: Some(running(60, 1_000)),
        }]));
        let notifications = Arc::new(RecordingSink::new());
        let workflow =
            TimerExpiryWorkflow::new(store.clone(), Settings::default(), notifications.clone());

        let events = workflow.run_once(1_100).unwrap();

        let focuses = store.list().unwrap();
        assert_eq!(
            focuses[0].timer.as_ref().map(|timer| &timer.status),
            Some(&TimerStatus::Expired)
        );
        assert_eq!(
            events,
            vec![TimerExpiryEvent::FocusTimerExpired {
                focus_id: "focus-1".to_string(),
                focus_title: "Ship feature".to_string(),
            }]
        );
        assert_eq!(
            notifications.requests(),
            vec![NotificationRequest {
                source_key: TimerExpiredSource.key().to_string(),
                title: "Timer expired".to_string(),
                body: "Ship feature reached its timer.".to_string(),
            }]
        );
    }
}
