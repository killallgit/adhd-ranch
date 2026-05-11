use crate::focus::Focus;
use crate::timer::{timer_remaining_secs, TimerStatus};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TimerTransition {
    pub focus_id: String,
    pub focus_title: String,
}

pub fn tick(now_secs: i64, focuses: &[Focus]) -> Vec<TimerTransition> {
    focuses
        .iter()
        .filter_map(|f| {
            let timer = f.timer.as_ref()?;
            // Persisted Expired status is the dedup lock — without this gate, a focus
            // would re-fire a transition every tick after the first expiry.
            if !matches!(timer.status, TimerStatus::Running) {
                return None;
            }
            if timer_remaining_secs(timer, now_secs).is_some() {
                return None;
            }
            Some(TimerTransition {
                focus_id: f.id.0.clone(),
                focus_title: f.title.clone(),
            })
        })
        .collect()
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
                },
                TimerTransition {
                    focus_id: "d".to_string(),
                    focus_title: "Delta".to_string(),
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
            }]
        );
    }
}
