use std::sync::Arc;

use adhd_ranch_domain::{Caps, Focus, NewFocus};
use adhd_ranch_storage::FocusStore;
use serde::{Deserialize, Serialize};

use crate::error::CommandError;
use crate::{Clock, Commands, IdGen};

#[derive(Debug, Clone, Deserialize)]
pub struct CreateFocusInput {
    pub title: String,
    #[serde(default)]
    pub description: String,
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
) -> Result<String, CommandError> {
    let id = id_gen();
    let created_at = clock();
    Ok(store.create_focus(new_focus, &id, &created_at)?)
}

impl Commands {
    pub fn list_focuses(&self) -> Result<Vec<Focus>, CommandError> {
        Ok(self.store.list()?)
    }

    pub fn create_focus(&self, input: CreateFocusInput) -> Result<CreatedFocus, CommandError> {
        if input.title.trim().is_empty() {
            return Err(CommandError::BadRequest("title must not be empty".into()));
        }
        let new_focus = NewFocus {
            title: input.title,
            description: input.description,
        };
        let slug = create_focus_in_store(&self.store, &self.clock, &self.id_gen, &new_focus)?;
        Ok(CreatedFocus { id: slug })
    }

    pub fn delete_focus(&self, focus_id: &str) -> Result<(), CommandError> {
        self.store.delete_focus(focus_id)?;
        Ok(())
    }

    pub fn append_task(&self, focus_id: &str, text: &str) -> Result<(), CommandError> {
        if text.trim().is_empty() {
            return Err(CommandError::BadRequest("text must not be empty".into()));
        }
        self.store.append_task(focus_id, text)?;
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
