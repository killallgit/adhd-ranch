use std::sync::Arc;

use adhd_ranch_domain::{cap_state, OverCapMonitor, Settings};
use adhd_ranch_storage::FocusStore;

use crate::error::CommandError;

pub trait CapNotifier: Send + Sync {
    fn focuses_over_cap(&self, max: usize);
    fn focuses_under_cap(&self);
    fn task_over_cap(&self, focus_id: &str, max: usize);
    fn task_under_cap(&self, focus_id: &str);
}

pub struct CapEvaluator {
    store: Arc<dyn FocusStore>,
    monitor: Arc<OverCapMonitor>,
    notifier: Arc<dyn CapNotifier>,
    settings: Settings,
}

impl CapEvaluator {
    pub fn new(
        store: Arc<dyn FocusStore>,
        monitor: Arc<OverCapMonitor>,
        notifier: Arc<dyn CapNotifier>,
        settings: Settings,
    ) -> Self {
        Self {
            store,
            monitor,
            notifier,
            settings,
        }
    }

    pub fn evaluate(&self) -> Result<(), CommandError> {
        let focuses = self.store.list()?;
        let state = cap_state(&focuses, self.settings.caps);
        let transition = self.monitor.evaluate(&state);

        if !self.settings.alerts.system_notifications {
            return Ok(());
        }

        if transition.focuses_to_over {
            self.notifier
                .focuses_over_cap(self.settings.caps.max_focuses);
        }
        if transition.focuses_to_under {
            self.notifier.focuses_under_cap();
        }
        for id in &transition.task_to_over_focus_ids {
            self.notifier
                .task_over_cap(id, self.settings.caps.max_tasks_per_focus);
        }
        for id in &transition.task_to_under_focus_ids {
            self.notifier.task_under_cap(id);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Mutex;

    use adhd_ranch_domain::focus::{Focus, FocusId, Task};
    use adhd_ranch_domain::{Alerts, Caps, NewFocus, Widget};
    use adhd_ranch_storage::FocusStoreError;

    use super::*;

    #[derive(Debug, PartialEq, Eq)]
    enum Call {
        FocusesOver(usize),
        FocusesUnder,
        TaskOver(String, usize),
        TaskUnder(String),
    }

    struct RecordingNotifier {
        calls: Mutex<Vec<Call>>,
    }

    impl RecordingNotifier {
        fn new() -> Self {
            Self {
                calls: Mutex::new(Vec::new()),
            }
        }

        fn calls(&self) -> Vec<Call> {
            self.calls.lock().unwrap().drain(..).collect()
        }
    }

    impl CapNotifier for RecordingNotifier {
        fn focuses_over_cap(&self, max: usize) {
            self.calls.lock().unwrap().push(Call::FocusesOver(max));
        }
        fn focuses_under_cap(&self) {
            self.calls.lock().unwrap().push(Call::FocusesUnder);
        }
        fn task_over_cap(&self, focus_id: &str, max: usize) {
            self.calls
                .lock()
                .unwrap()
                .push(Call::TaskOver(focus_id.to_string(), max));
        }
        fn task_under_cap(&self, focus_id: &str) {
            self.calls
                .lock()
                .unwrap()
                .push(Call::TaskUnder(focus_id.to_string()));
        }
    }

    struct StubStore {
        focuses: Mutex<Vec<Focus>>,
    }

    impl StubStore {
        fn new() -> Self {
            Self {
                focuses: Mutex::new(Vec::new()),
            }
        }

        fn set(&self, focuses: Vec<Focus>) {
            *self.focuses.lock().unwrap() = focuses;
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
        ) -> Result<String, FocusStoreError> {
            unimplemented!()
        }
        fn delete_focus(&self, _focus_id: &str) -> Result<(), FocusStoreError> {
            unimplemented!()
        }
        fn append_task(&self, _focus_id: &str, _text: &str) -> Result<(), FocusStoreError> {
            unimplemented!()
        }
        fn delete_task(&self, _focus_id: &str, _index: usize) -> Result<(), FocusStoreError> {
            unimplemented!()
        }
    }

    fn focus_with_tasks(id: &str, count: usize) -> Focus {
        Focus {
            id: FocusId(id.into()),
            title: id.into(),
            description: String::new(),
            created_at: String::new(),
            tasks: (0..count)
                .map(|i| Task {
                    id: format!("{id}:{i}"),
                    text: format!("t{i}"),
                })
                .collect(),
        }
    }

    fn settings(notifications: bool) -> Settings {
        Settings {
            caps: Caps {
                max_focuses: 5,
                max_tasks_per_focus: 7,
            },
            alerts: Alerts {
                system_notifications: notifications,
            },
            widget: Widget::default(),
        }
    }

    fn build(notifications: bool) -> (Arc<StubStore>, Arc<RecordingNotifier>, CapEvaluator) {
        let store = Arc::new(StubStore::new());
        let notifier = Arc::new(RecordingNotifier::new());
        let evaluator = CapEvaluator::new(
            store.clone(),
            Arc::new(OverCapMonitor::new()),
            notifier.clone(),
            settings(notifications),
        );
        (store, notifier, evaluator)
    }

    #[test]
    fn under_caps_emits_nothing() {
        let (store, notifier, evaluator) = build(true);
        store.set(vec![focus_with_tasks("a", 3)]);
        evaluator.evaluate().unwrap();
        assert!(notifier.calls().is_empty());
    }

    #[test]
    fn focuses_to_over_calls_notifier_once() {
        let (store, notifier, evaluator) = build(true);
        store.set(
            (0..6)
                .map(|i| focus_with_tasks(&format!("f{i}"), 0))
                .collect(),
        );
        evaluator.evaluate().unwrap();
        assert_eq!(notifier.calls(), vec![Call::FocusesOver(5)]);

        evaluator.evaluate().unwrap();
        assert!(notifier.calls().is_empty());
    }

    #[test]
    fn focuses_recovery_calls_under() {
        let (store, notifier, evaluator) = build(true);
        store.set(
            (0..6)
                .map(|i| focus_with_tasks(&format!("f{i}"), 0))
                .collect(),
        );
        evaluator.evaluate().unwrap();
        let _ = notifier.calls();

        store.set(
            (0..3)
                .map(|i| focus_with_tasks(&format!("f{i}"), 0))
                .collect(),
        );
        evaluator.evaluate().unwrap();
        assert_eq!(notifier.calls(), vec![Call::FocusesUnder]);
    }

    #[test]
    fn task_cap_transitions_per_focus() {
        let (store, notifier, evaluator) = build(true);
        store.set(vec![focus_with_tasks("a", 9)]);
        evaluator.evaluate().unwrap();
        assert_eq!(notifier.calls(), vec![Call::TaskOver("a".into(), 7)]);

        store.set(vec![focus_with_tasks("a", 3)]);
        evaluator.evaluate().unwrap();
        assert_eq!(notifier.calls(), vec![Call::TaskUnder("a".into())]);
    }

    #[test]
    fn notifications_disabled_suppresses_calls() {
        let (store, notifier, evaluator) = build(false);
        store.set(
            (0..6)
                .map(|i| focus_with_tasks(&format!("f{i}"), 0))
                .collect(),
        );
        evaluator.evaluate().unwrap();
        assert!(notifier.calls().is_empty());
    }
}
