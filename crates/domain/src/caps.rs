use crate::focus::Focus;
use crate::settings::Caps;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CapState {
    pub focuses_over: bool,
    pub focus_count: usize,
    pub over_task_focus_ids: Vec<String>,
}

impl CapState {
    pub fn any_over(&self) -> bool {
        self.focuses_over || !self.over_task_focus_ids.is_empty()
    }
}

pub fn cap_state(focuses: &[Focus], caps: Caps) -> CapState {
    let focuses_over = focuses.len() > caps.max_focuses;
    let mut over_task_focus_ids = Vec::new();
    for focus in focuses {
        if focus.tasks.len() > caps.max_tasks_per_focus {
            over_task_focus_ids.push(focus.id.0.clone());
        }
    }
    CapState {
        focuses_over,
        focus_count: focuses.len(),
        over_task_focus_ids,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::focus::{FocusId, Task};

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
                    done: false,
                })
                .collect(),
            timer: None,
        }
    }

    fn caps(max_focuses: usize, max_tasks_per_focus: usize) -> Caps {
        Caps {
            max_focuses,
            max_tasks_per_focus,
        }
    }

    #[test]
    fn under_returns_no_over_flags() {
        let focuses = vec![focus_with_tasks("a", 3)];
        let state = cap_state(&focuses, caps(5, 7));
        assert!(!state.focuses_over);
        assert!(state.over_task_focus_ids.is_empty());
        assert!(!state.any_over());
    }

    #[test]
    fn at_cap_is_not_over() {
        let focuses: Vec<Focus> = (0..5)
            .map(|i| focus_with_tasks(&format!("f{i}"), 7))
            .collect();
        let state = cap_state(&focuses, caps(5, 7));
        assert!(!state.focuses_over);
        assert!(state.over_task_focus_ids.is_empty());
    }

    #[test]
    fn over_focuses_cap_is_flagged() {
        let focuses: Vec<Focus> = (0..6)
            .map(|i| focus_with_tasks(&format!("f{i}"), 0))
            .collect();
        let state = cap_state(&focuses, caps(5, 7));
        assert!(state.focuses_over);
        assert_eq!(state.focus_count, 6);
    }

    #[test]
    fn over_tasks_cap_lists_focus_ids() {
        let focuses = vec![
            focus_with_tasks("good", 3),
            focus_with_tasks("bad", 9),
            focus_with_tasks("worse", 12),
        ];
        let state = cap_state(&focuses, caps(5, 7));
        assert!(!state.focuses_over);
        assert_eq!(state.over_task_focus_ids, vec!["bad", "worse"]);
        assert!(state.any_over());
    }
}
