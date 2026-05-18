use crate::focus::Focus;
use crate::timer::{timer_remaining_secs, TimerStatus};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TimerTransition {
    pub focus_id: String,
    pub focus_title: String,
    pub target: TimerTransitionTarget,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TimerTransitionTarget {
    Focus,
    Task { index: usize, text: String },
}

pub fn tick(now_secs: i64, focuses: &[Focus]) -> Vec<TimerTransition> {
    let mut transitions = Vec::new();
    for f in focuses {
        if let Some(timer) = f.timer.as_ref() {
            // Persisted Expired status is the dedup lock — without this gate, a focus
            // would re-fire a transition every tick after the first expiry.
            if matches!(timer.status, TimerStatus::Running)
                && timer_remaining_secs(timer, now_secs).is_none()
            {
                transitions.push(TimerTransition {
                    focus_id: f.id.0.clone(),
                    focus_title: f.title.clone(),
                    target: TimerTransitionTarget::Focus,
                });
            }
        }

        for (index, task) in f.tasks.iter().enumerate() {
            let Some(timer) = task.timer.as_ref() else {
                continue;
            };
            if !matches!(timer.status, TimerStatus::Running) {
                continue;
            }
            if timer_remaining_secs(timer, now_secs).is_some() {
                continue;
            }
            transitions.push(TimerTransition {
                focus_id: f.id.0.clone(),
                focus_title: f.title.clone(),
                target: TimerTransitionTarget::Task {
                    index,
                    text: task.text.clone(),
                },
            });
        }
    }
    transitions
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::focus::{Focus, FocusId};
    use crate::timer::{FocusTimer, TimerStatus};

    fn focus_with(id: &str, title: &str, timer: Option<FocusTimer>) -> Focus {
        Focus {
            id: FocusId(id.to_string()),
            title: title.to_string(),
            description: String::new(),
            created_at: String::new(),
            tasks: Vec::new(),
            timer,
        }
    }

    fn running(duration_secs: u64, started_at: i64) -> FocusTimer {
        FocusTimer {
            duration_secs,
            started_at,
            status: TimerStatus::Running,
        }
    }

    #[test]
    fn multiple_focuses_return_transitions_in_input_order() {
        let expired_a = focus_with("a", "Alpha", Some(running(60, 1_000)));
        let still_running = focus_with("b", "Beta", Some(running(600, 1_000)));
        let no_timer = focus_with("c", "Gamma", None);
        let expired_d = focus_with("d", "Delta", Some(running(30, 1_000)));
        let focuses = vec![expired_a, still_running, no_timer, expired_d];
        assert_eq!(
            tick(1_100, &focuses),
            vec![
                TimerTransition {
                    focus_id: "a".to_string(),
                    focus_title: "Alpha".to_string(),
                    target: TimerTransitionTarget::Focus,
                },
                TimerTransition {
                    focus_id: "d".to_string(),
                    focus_title: "Delta".to_string(),
                    target: TimerTransitionTarget::Focus,
                },
            ]
        );
    }

    #[test]
    fn exactly_at_expiry_boundary_transitions() {
        let f = focus_with("focus-1", "Ship feature", Some(running(60, 1_000)));
        // started_at + duration_secs == now_secs → elapsed == duration → expired
        assert_eq!(
            tick(1_060, std::slice::from_ref(&f)),
            vec![TimerTransition {
                focus_id: "focus-1".to_string(),
                focus_title: "Ship feature".to_string(),
                target: TimerTransitionTarget::Focus,
            }]
        );
    }

    #[test]
    fn focus_without_timer_is_skipped() {
        let f = focus_with("focus-1", "Ship feature", None);
        assert_eq!(tick(1_100, std::slice::from_ref(&f)), vec![]);
    }

    #[test]
    fn already_expired_status_is_skipped() {
        let f = focus_with(
            "focus-1",
            "Ship feature",
            Some(FocusTimer {
                duration_secs: 60,
                started_at: 1_000,
                status: TimerStatus::Expired,
            }),
        );
        assert_eq!(tick(1_100, std::slice::from_ref(&f)), vec![]);
    }

    #[test]
    fn running_not_yet_expired_returns_empty() {
        let f = focus_with("focus-1", "Ship feature", Some(running(60, 1_000)));
        assert_eq!(tick(1_030, std::slice::from_ref(&f)), vec![]);
    }

    #[test]
    fn running_past_expiry_returns_transition_with_id_and_title() {
        let f = focus_with("focus-1", "Ship feature", Some(running(60, 1_000)));
        let transitions = tick(1_100, std::slice::from_ref(&f));
        assert_eq!(
            transitions,
            vec![TimerTransition {
                focus_id: "focus-1".to_string(),
                focus_title: "Ship feature".to_string(),
                target: TimerTransitionTarget::Focus,
            }]
        );
    }

    #[test]
    fn running_task_timer_past_expiry_returns_task_transition() {
        let mut f = focus_with("focus-1", "Ship feature", None);
        f.tasks.push(crate::focus::Task {
            id: "task-1".to_string(),
            text: "Write tests".to_string(),
            done: false,
            timer: Some(running(60, 1_000)),
        });

        assert_eq!(
            tick(1_100, std::slice::from_ref(&f)),
            vec![TimerTransition {
                focus_id: "focus-1".to_string(),
                focus_title: "Ship feature".to_string(),
                target: TimerTransitionTarget::Task {
                    index: 0,
                    text: "Write tests".to_string(),
                },
            }]
        );
    }

    #[test]
    fn task_without_timer_is_skipped() {
        let mut f = focus_with("focus-1", "Ship feature", None);
        f.tasks.push(crate::focus::Task {
            id: "task-1".to_string(),
            text: "Write tests".to_string(),
            done: false,
            timer: None,
        });

        assert_eq!(tick(1_100, std::slice::from_ref(&f)), vec![]);
    }

    #[test]
    fn already_expired_task_timer_is_skipped() {
        let mut f = focus_with("focus-1", "Ship feature", None);
        f.tasks.push(crate::focus::Task {
            id: "task-1".to_string(),
            text: "Write tests".to_string(),
            done: false,
            timer: Some(FocusTimer {
                duration_secs: 60,
                started_at: 1_000,
                status: TimerStatus::Expired,
            }),
        });

        assert_eq!(tick(1_100, std::slice::from_ref(&f)), vec![]);
    }

    #[test]
    fn running_task_timer_before_expiry_is_skipped() {
        let mut f = focus_with("focus-1", "Ship feature", None);
        f.tasks.push(crate::focus::Task {
            id: "task-1".to_string(),
            text: "Write tests".to_string(),
            done: false,
            timer: Some(running(60, 1_000)),
        });

        assert_eq!(tick(1_030, std::slice::from_ref(&f)), vec![]);
    }
}
