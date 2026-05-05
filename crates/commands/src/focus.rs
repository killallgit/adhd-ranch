use std::sync::Arc;

use adhd_ranch_domain::{Caps, Focus, FocusTimer, NewFocus, TaskText, TimerPreset, TimerStatus};
use adhd_ranch_storage::FocusStore;
use serde::{Deserialize, Serialize};

use crate::error::CommandError;
use crate::{Clock, Commands, IdGen};

#[derive(Debug, Clone, Deserialize)]
pub struct CreateFocusInput {
    pub title: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub timer_preset: Option<TimerPreset>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CreatedFocus {
    pub id: String,
}

pub(crate) fn create_focus_in_store(
    store: &Arc<dyn FocusStore>,
    clock: &Clock,
    id_gen: &IdGen,
    new_focus: &NewFocus,
    timer: Option<FocusTimer>,
) -> Result<String, CommandError> {
    let id = id_gen();
    let created_at = clock();
    Ok(store.create_focus(new_focus, &id, &created_at, timer)?)
}

impl Commands {
    pub fn list_focuses(&self) -> Result<Vec<Focus>, CommandError> {
        Ok(self.store.list()?)
    }

    pub fn create_focus(&self, input: CreateFocusInput) -> Result<CreatedFocus, CommandError> {
        let timer = input.timer_preset.as_ref().map(|preset| FocusTimer {
            duration_secs: preset.duration_secs(),
            started_at: (self.clock_secs)(),
            status: TimerStatus::Running,
        });
        let new_focus =
            NewFocus::new(input.title, input.description)?.with_timer_preset(input.timer_preset);
        let slug =
            create_focus_in_store(&self.store, &self.clock, &self.id_gen, &new_focus, timer)?;
        Ok(CreatedFocus { id: slug })
    }

    pub fn delete_focus(&self, focus_id: &str) -> Result<(), CommandError> {
        self.store.delete_focus(focus_id)?;
        Ok(())
    }

    pub fn append_task(&self, focus_id: &str, text: &str) -> Result<(), CommandError> {
        let text = TaskText::new(text)?;
        self.store.append_task(focus_id, text.as_str())?;
        Ok(())
    }

    pub fn delete_task(&self, focus_id: &str, index: usize) -> Result<(), CommandError> {
        self.store.delete_task(focus_id, index)?;
        Ok(())
    }

    pub fn caps(&self) -> Caps {
        self.settings.caps
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use adhd_ranch_domain::{Settings, TimerStatus};
    use adhd_ranch_storage::{JsonlDecisionLog, JsonlProposalQueue, MarkdownFocusStore};
    use std::sync::Arc;
    use tempfile::TempDir;

    fn build_commands(clock_secs_val: i64) -> (Commands, TempDir) {
        let dir = TempDir::new().unwrap();
        let focuses_root = dir.path().join("focuses");
        std::fs::create_dir_all(&focuses_root).unwrap();
        let store: Arc<dyn adhd_ranch_storage::FocusStore> =
            Arc::new(MarkdownFocusStore::new(focuses_root));
        let queue: Arc<dyn adhd_ranch_storage::ProposalQueue> =
            Arc::new(JsonlProposalQueue::new(dir.path().join("proposals.jsonl")));
        let decisions: Arc<dyn adhd_ranch_storage::DecisionLog> =
            Arc::new(JsonlDecisionLog::new(dir.path().join("decisions.jsonl")));
        let commands = Commands::new(
            store,
            queue,
            decisions,
            Arc::new(|| "2026-01-01T00:00:00Z".to_string()),
            Arc::new(move || clock_secs_val),
            Arc::new(|| "test-id".to_string()),
            Settings::default(),
        );
        (commands, dir)
    }

    #[test]
    fn create_focus_without_preset_stores_no_timer() {
        let (commands, _dir) = build_commands(1_000_000);
        commands
            .create_focus(CreateFocusInput {
                title: "No timer focus".into(),
                description: String::new(),
                timer_preset: None,
            })
            .unwrap();
        let focuses = commands.list_focuses().unwrap();
        assert_eq!(focuses.len(), 1);
        assert!(focuses[0].timer.is_none());
    }

    #[test]
    fn create_focus_blank_title_returns_bad_request() {
        let (commands, _dir) = build_commands(0);
        let err = commands
            .create_focus(CreateFocusInput {
                title: "  ".into(),
                description: String::new(),
                timer_preset: None,
            })
            .unwrap_err();
        assert!(matches!(err, CommandError::BadRequest(_)));
    }

    #[test]
    fn append_task_blank_text_returns_bad_request() {
        let (commands, _dir) = build_commands(0);
        let created = commands
            .create_focus(CreateFocusInput {
                title: "Real focus".into(),
                description: String::new(),
                timer_preset: None,
            })
            .unwrap();
        let err = commands.append_task(&created.id, "   ").unwrap_err();
        assert!(matches!(err, CommandError::BadRequest(_)));
    }

    #[test]
    fn create_focus_with_preset_stores_timer_with_correct_duration() {
        let started_at = 1_700_000_000_i64;
        let (commands, _dir) = build_commands(started_at);
        commands
            .create_focus(CreateFocusInput {
                title: "Timer focus".into(),
                description: String::new(),
                timer_preset: Some(TimerPreset::Eight),
            })
            .unwrap();
        let focuses = commands.list_focuses().unwrap();
        assert_eq!(focuses.len(), 1);
        let timer = focuses[0].timer.as_ref().expect("timer should be Some");
        assert_eq!(timer.duration_secs, 480);
        assert_eq!(timer.started_at, started_at);
        assert_eq!(timer.status, TimerStatus::Running);
    }
}
