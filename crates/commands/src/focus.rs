use std::sync::Arc;

use adhd_ranch_domain::{
    slugify, Caps, Focus, FocusTimer, NewFocus, TaskText, TimerPreset, TimerStatus,
};
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

    pub fn duplicate_focus(&self, focus_id: &str) -> Result<CreatedFocus, CommandError> {
        let focuses = self.store.list()?;
        let source = focuses
            .iter()
            .find(|focus| focus.id.0 == focus_id)
            .ok_or_else(|| CommandError::NotFound(format!("focus not found: {focus_id}")))?;
        let title = duplicate_title(&source.title, &focuses);
        let new_focus = NewFocus::new(title, source.description.clone())?;
        let slug = create_focus_in_store(&self.store, &self.clock, &self.id_gen, &new_focus, None)?;
        for (index, task) in source.tasks.iter().enumerate() {
            self.store.append_task(&slug, &task.text)?;
            if task.done {
                self.store.toggle_task(&slug, index, true)?;
            }
        }
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

    pub fn rename_focus(&self, focus_id: &str, title: &str) -> Result<(), CommandError> {
        let trimmed = title.trim();
        if trimmed.is_empty() {
            return Err(CommandError::from(
                adhd_ranch_domain::DomainError::EmptyTitle,
            ));
        }
        self.store.rename_focus(focus_id, trimmed)?;
        Ok(())
    }

    pub fn update_task(
        &self,
        focus_id: &str,
        index: usize,
        text: &str,
    ) -> Result<(), CommandError> {
        let text = TaskText::new(text)?;
        self.store.update_task(focus_id, index, text.as_str())?;
        Ok(())
    }

    pub fn toggle_task(
        &self,
        focus_id: &str,
        index: usize,
        done: bool,
    ) -> Result<(), CommandError> {
        self.store.toggle_task(focus_id, index, done)?;
        Ok(())
    }

    pub fn start_timer(&self, focus_id: &str, preset: TimerPreset) -> Result<(), CommandError> {
        let timer = FocusTimer {
            duration_secs: preset.duration_secs(),
            started_at: (self.clock_secs)(),
            status: TimerStatus::Running,
        };
        self.store.update_timer(focus_id, &timer)?;
        Ok(())
    }

    pub fn clear_timer(&self, focus_id: &str) -> Result<(), CommandError> {
        self.store.clear_timer(focus_id)?;
        Ok(())
    }

    pub fn start_task_timer(
        &self,
        focus_id: &str,
        index: usize,
        preset: TimerPreset,
    ) -> Result<(), CommandError> {
        let timer = FocusTimer {
            duration_secs: preset.duration_secs(),
            started_at: (self.clock_secs)(),
            status: TimerStatus::Running,
        };
        self.store.update_task_timer(focus_id, index, &timer)?;
        Ok(())
    }

    pub fn clear_task_timer(&self, focus_id: &str, index: usize) -> Result<(), CommandError> {
        self.store.clear_task_timer(focus_id, index)?;
        Ok(())
    }

    pub fn caps(&self) -> Caps {
        self.settings.caps
    }
}

fn duplicate_title(title: &str, focuses: &[Focus]) -> String {
    let existing_slugs = focuses
        .iter()
        .map(|focus| focus.id.0.as_str())
        .collect::<std::collections::HashSet<_>>();
    let base = format!("{title} copy");
    if !existing_slugs.contains(slugify(&base).as_str()) {
        return base;
    }
    for n in 2.. {
        let candidate = format!("{base} {n}");
        if !existing_slugs.contains(slugify(&candidate).as_str()) {
            return candidate;
        }
    }
    unreachable!("unbounded duplicate title search should always find a title")
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
    fn append_task_revives_expired_focus() {
        let (commands, _dir) = build_commands(1_700_000_000);
        let created = commands
            .create_focus(CreateFocusInput {
                title: "Dead focus".into(),
                description: String::new(),
                timer_preset: None,
            })
            .unwrap();
        commands
            .store
            .update_timer(
                &created.id,
                &FocusTimer {
                    duration_secs: 60,
                    started_at: 1_000,
                    status: TimerStatus::Expired,
                },
            )
            .unwrap();

        commands.append_task(&created.id, "new life").unwrap();

        let focuses = commands.list_focuses().unwrap();
        assert!(focuses[0].timer.is_none());
        assert_eq!(focuses[0].tasks[0].text, "new life");
    }

    #[test]
    fn duplicate_focus_copies_title_description_and_tasks() {
        let (commands, _dir) = build_commands(1_000_000);
        let created = commands
            .create_focus(CreateFocusInput {
                title: "Ship it".into(),
                description: "release plan".into(),
                timer_preset: Some(TimerPreset::Two),
            })
            .unwrap();
        commands.append_task(&created.id, "first").unwrap();
        commands.append_task(&created.id, "second").unwrap();
        commands.toggle_task(&created.id, 1, true).unwrap();

        let duplicated = commands.duplicate_focus(&created.id).unwrap();

        assert_eq!(duplicated.id, "ship-it-copy");
        let focuses = commands.list_focuses().unwrap();
        let copy = focuses
            .iter()
            .find(|focus| focus.id.0 == duplicated.id)
            .unwrap();
        assert_eq!(copy.title, "Ship it copy");
        assert_eq!(copy.description, "release plan");
        assert!(copy.timer.is_none());
        assert_eq!(copy.tasks.len(), 2);
        assert_eq!(copy.tasks[0].text, "first");
        assert!(!copy.tasks[0].done);
        assert_eq!(copy.tasks[1].text, "second");
        assert!(copy.tasks[1].done);
    }

    #[test]
    fn duplicate_focus_uses_numeric_suffix_when_copy_exists() {
        let (commands, _dir) = build_commands(1_000_000);
        let created = commands
            .create_focus(CreateFocusInput {
                title: "Ship it".into(),
                description: String::new(),
                timer_preset: None,
            })
            .unwrap();
        commands.duplicate_focus(&created.id).unwrap();
        let duplicated = commands.duplicate_focus(&created.id).unwrap();

        assert_eq!(duplicated.id, "ship-it-copy-2");
    }

    #[test]
    fn duplicate_focus_unknown_id_returns_not_found() {
        let (commands, _dir) = build_commands(1_000_000);
        let err = commands.duplicate_focus("missing").unwrap_err();
        assert!(matches!(err, CommandError::NotFound(_)));
    }

    #[test]
    fn rename_focus_updates_title() {
        let (commands, _dir) = build_commands(0);
        let created = commands
            .create_focus(CreateFocusInput {
                title: "Old".into(),
                description: String::new(),
                timer_preset: None,
            })
            .unwrap();
        commands.rename_focus(&created.id, "New").unwrap();
        let focuses = commands.list_focuses().unwrap();
        assert_eq!(focuses[0].title, "New");
    }

    #[test]
    fn rename_focus_blank_title_returns_bad_request() {
        let (commands, _dir) = build_commands(0);
        let created = commands
            .create_focus(CreateFocusInput {
                title: "Real".into(),
                description: String::new(),
                timer_preset: None,
            })
            .unwrap();
        let err = commands.rename_focus(&created.id, "   ").unwrap_err();
        assert!(matches!(err, CommandError::BadRequest(_)));
    }

    #[test]
    fn update_task_blank_text_returns_bad_request() {
        let (commands, _dir) = build_commands(0);
        let created = commands
            .create_focus(CreateFocusInput {
                title: "Has tasks".into(),
                description: String::new(),
                timer_preset: None,
            })
            .unwrap();
        commands.append_task(&created.id, "first").unwrap();
        let err = commands.update_task(&created.id, 0, "  ").unwrap_err();
        assert!(matches!(err, CommandError::BadRequest(_)));
    }

    #[test]
    fn update_task_replaces_text() {
        let (commands, _dir) = build_commands(0);
        let created = commands
            .create_focus(CreateFocusInput {
                title: "Has tasks".into(),
                description: String::new(),
                timer_preset: None,
            })
            .unwrap();
        commands.append_task(&created.id, "old").unwrap();
        commands.update_task(&created.id, 0, "new").unwrap();
        let focuses = commands.list_focuses().unwrap();
        assert_eq!(focuses[0].tasks[0].text, "new");
    }

    #[test]
    fn toggle_task_round_trip() {
        let (commands, _dir) = build_commands(0);
        let created = commands
            .create_focus(CreateFocusInput {
                title: "Has tasks".into(),
                description: String::new(),
                timer_preset: None,
            })
            .unwrap();
        commands.append_task(&created.id, "thing").unwrap();
        commands.toggle_task(&created.id, 0, true).unwrap();
        commands.toggle_task(&created.id, 0, false).unwrap();
        let focuses = commands.list_focuses().unwrap();
        assert!(!focuses[0].tasks[0].done);
    }

    #[test]
    fn start_timer_sets_running_timer_with_preset_duration() {
        let started_at = 1_700_000_500_i64;
        let (commands, _dir) = build_commands(started_at);
        let created = commands
            .create_focus(CreateFocusInput {
                title: "No timer yet".into(),
                description: String::new(),
                timer_preset: None,
            })
            .unwrap();

        commands
            .start_timer(&created.id, TimerPreset::Four)
            .unwrap();

        let focuses = commands.list_focuses().unwrap();
        let timer = focuses[0].timer.as_ref().expect("timer should be Some");
        assert_eq!(timer.duration_secs, 240);
        assert_eq!(timer.started_at, started_at);
        assert_eq!(timer.status, TimerStatus::Running);
    }

    #[test]
    fn start_timer_unknown_focus_returns_not_found() {
        let (commands, _dir) = build_commands(0);
        let err = commands
            .start_timer("does-not-exist", TimerPreset::Two)
            .unwrap_err();
        assert!(matches!(err, CommandError::NotFound(_)));
    }

    #[test]
    fn clear_timer_removes_focus_timer() {
        let (commands, _dir) = build_commands(1_700_000_500);
        let created = commands
            .create_focus(CreateFocusInput {
                title: "Timed".into(),
                description: String::new(),
                timer_preset: Some(TimerPreset::Two),
            })
            .unwrap();

        commands.clear_timer(&created.id).unwrap();

        let focuses = commands.list_focuses().unwrap();
        assert!(focuses[0].timer.is_none());
    }

    #[test]
    fn start_task_timer_sets_running_timer_on_task() {
        let started_at = 1_700_000_500_i64;
        let (commands, _dir) = build_commands(started_at);
        let created = commands
            .create_focus(CreateFocusInput {
                title: "Task timers".into(),
                description: String::new(),
                timer_preset: None,
            })
            .unwrap();
        commands.append_task(&created.id, "one").unwrap();
        commands.append_task(&created.id, "two").unwrap();

        commands
            .start_task_timer(&created.id, 1, TimerPreset::Four)
            .unwrap();

        let focuses = commands.list_focuses().unwrap();
        assert!(focuses[0].tasks[0].timer.is_none());
        let timer = focuses[0].tasks[1]
            .timer
            .as_ref()
            .expect("task timer should be Some");
        assert_eq!(timer.duration_secs, 240);
        assert_eq!(timer.started_at, started_at);
        assert_eq!(timer.status, TimerStatus::Running);
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
