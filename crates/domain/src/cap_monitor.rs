use std::collections::HashSet;
use std::sync::Mutex;

use crate::caps::CapState;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct CapTransition {
    pub focuses_to_over: bool,
    pub focuses_to_under: bool,
    pub task_to_over_focus_ids: Vec<String>,
    pub task_to_under_focus_ids: Vec<String>,
}

impl CapTransition {
    pub fn fired(&self) -> bool {
        self.focuses_to_over
            || self.focuses_to_under
            || !self.task_to_over_focus_ids.is_empty()
            || !self.task_to_under_focus_ids.is_empty()
    }
}

#[derive(Default)]
struct State {
    focuses_over: bool,
    over_task_focus_ids: HashSet<String>,
}

pub struct OverCapMonitor {
    state: Mutex<State>,
}

impl OverCapMonitor {
    pub fn new() -> Self {
        Self {
            state: Mutex::new(State::default()),
        }
    }

    pub fn evaluate(&self, current: &CapState) -> CapTransition {
        let mut state = self.state.lock().unwrap_or_else(|e| e.into_inner());

        let mut transition = CapTransition::default();

        if current.focuses_over && !state.focuses_over {
            transition.focuses_to_over = true;
        } else if !current.focuses_over && state.focuses_over {
            transition.focuses_to_under = true;
        }
        state.focuses_over = current.focuses_over;

        let now: HashSet<String> = current.over_task_focus_ids.iter().cloned().collect();
        for id in &now {
            if !state.over_task_focus_ids.contains(id) {
                transition.task_to_over_focus_ids.push(id.clone());
            }
        }
        for id in state.over_task_focus_ids.iter() {
            if !now.contains(id) {
                transition.task_to_under_focus_ids.push(id.clone());
            }
        }
        state.over_task_focus_ids = now;

        transition
    }
}

impl Default for OverCapMonitor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cap(focuses_over: bool, task_over_ids: Vec<&str>) -> CapState {
        CapState {
            focuses_over,
            focus_count: if focuses_over { 6 } else { 4 },
            over_task_focus_ids: task_over_ids.into_iter().map(String::from).collect(),
        }
    }

    #[test]
    fn first_under_evaluation_fires_nothing() {
        let monitor = OverCapMonitor::new();
        let t = monitor.evaluate(&cap(false, vec![]));
        assert!(!t.fired());
    }

    #[test]
    fn under_to_over_fires_to_over_once() {
        let monitor = OverCapMonitor::new();
        let _ = monitor.evaluate(&cap(false, vec![]));
        let t = monitor.evaluate(&cap(true, vec![]));
        assert!(t.focuses_to_over);
        let again = monitor.evaluate(&cap(true, vec![]));
        assert!(!again.focuses_to_over);
    }

    #[test]
    fn over_to_under_fires_to_under() {
        let monitor = OverCapMonitor::new();
        let _ = monitor.evaluate(&cap(true, vec![]));
        let t = monitor.evaluate(&cap(false, vec![]));
        assert!(t.focuses_to_under);
        assert!(!t.focuses_to_over);
    }

    #[test]
    fn task_cap_transitions_track_per_focus() {
        let monitor = OverCapMonitor::new();
        let _ = monitor.evaluate(&cap(false, vec![]));
        let t = monitor.evaluate(&cap(false, vec!["f1"]));
        assert_eq!(t.task_to_over_focus_ids, vec!["f1"]);
        let again = monitor.evaluate(&cap(false, vec!["f1"]));
        assert!(again.task_to_over_focus_ids.is_empty());
        let recover = monitor.evaluate(&cap(false, vec![]));
        assert_eq!(recover.task_to_under_focus_ids, vec!["f1"]);
    }
}
