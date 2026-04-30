use std::sync::Arc;

use adhd_ranch_domain::{Focus, Proposal};
use adhd_ranch_storage::{FocusRepository, ProposalQueue};
use tauri::State;

use crate::api::Health;

pub struct FocusRepoState(pub Arc<dyn FocusRepository>);
pub struct ProposalQueueState(pub Arc<dyn ProposalQueue>);

#[tauri::command]
pub fn health() -> Health {
    Health { ok: true }
}

#[tauri::command]
pub fn list_focuses(state: State<'_, FocusRepoState>) -> Result<Vec<Focus>, String> {
    state.0.list().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn list_proposals(state: State<'_, ProposalQueueState>) -> Result<Vec<Proposal>, String> {
    state.0.list().map_err(|e| e.to_string())
}
