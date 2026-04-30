use std::sync::Arc;

use adhd_ranch_domain::Focus;
use adhd_ranch_storage::FocusRepository;
use tauri::State;

use crate::api::Health;

pub struct FocusRepoState(pub Arc<dyn FocusRepository>);

#[tauri::command]
pub fn health() -> Health {
    Health { ok: true }
}

#[tauri::command]
pub fn list_focuses(state: State<'_, FocusRepoState>) -> Result<Vec<Focus>, String> {
    state.0.list().map_err(|e| e.to_string())
}
