use std::collections::HashMap;

use serde::{Deserialize, Serialize};

pub trait NotificationSource {
    fn key(&self) -> &'static str;
    fn label(&self) -> &'static str;
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "export-ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "export-ts", ts(export))]
pub struct NotificationSettings {
    pub sources: HashMap<String, bool>,
}

impl NotificationSettings {
    pub fn is_enabled(&self, source: &dyn NotificationSource) -> bool {
        *self.sources.get(source.key()).unwrap_or(&true)
    }

    pub fn set(&mut self, source: &dyn NotificationSource, enabled: bool) {
        self.sources.insert(source.key().to_string(), enabled);
    }
}

pub struct TimerExpiredSource;
impl NotificationSource for TimerExpiredSource {
    fn key(&self) -> &'static str {
        "timer_expired"
    }
    fn label(&self) -> &'static str {
        "Timer expired"
    }
}

pub struct TaskTimerExpiredSource;
impl NotificationSource for TaskTimerExpiredSource {
    fn key(&self) -> &'static str {
        "task_timer_expired"
    }
    fn label(&self) -> &'static str {
        "Task timer expired"
    }
}

pub struct FocusesOverCapSource;
impl NotificationSource for FocusesOverCapSource {
    fn key(&self) -> &'static str {
        "focuses_over_cap"
    }
    fn label(&self) -> &'static str {
        "Too many focuses"
    }
}

pub struct TasksOverCapSource;
impl NotificationSource for TasksOverCapSource {
    fn key(&self) -> &'static str {
        "tasks_over_cap"
    }
    fn label(&self) -> &'static str {
        "Too many tasks in a focus"
    }
}

pub fn all_sources() -> Vec<Box<dyn NotificationSource>> {
    vec![
        Box::new(TimerExpiredSource),
        Box::new(TaskTimerExpiredSource),
        Box::new(FocusesOverCapSource),
        Box::new(TasksOverCapSource),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unknown_source_defaults_to_enabled() {
        let s = NotificationSettings::default();
        assert!(s.is_enabled(&TimerExpiredSource));
        assert!(s.is_enabled(&TaskTimerExpiredSource));
        assert!(s.is_enabled(&FocusesOverCapSource));
        assert!(s.is_enabled(&TasksOverCapSource));
    }

    #[test]
    fn explicit_disable_overrides_default() {
        let mut s = NotificationSettings::default();
        s.set(&TimerExpiredSource, false);
        assert!(!s.is_enabled(&TimerExpiredSource));
        assert!(s.is_enabled(&FocusesOverCapSource));
    }

    #[test]
    fn explicit_enable_is_explicit() {
        let mut s = NotificationSettings::default();
        s.set(&FocusesOverCapSource, true);
        assert!(s.is_enabled(&FocusesOverCapSource));
    }

    #[test]
    fn all_sources_lists_known_keys() {
        let keys: Vec<&'static str> = all_sources().iter().map(|s| s.key()).collect();
        assert_eq!(
            keys,
            vec![
                "timer_expired",
                "task_timer_expired",
                "focuses_over_cap",
                "tasks_over_cap"
            ]
        );
    }

    #[test]
    fn source_labels_are_human_readable() {
        assert_eq!(TimerExpiredSource.label(), "Timer expired");
        assert_eq!(TaskTimerExpiredSource.label(), "Task timer expired");
        assert_eq!(FocusesOverCapSource.label(), "Too many focuses");
        assert_eq!(TasksOverCapSource.label(), "Too many tasks in a focus");
    }
}
