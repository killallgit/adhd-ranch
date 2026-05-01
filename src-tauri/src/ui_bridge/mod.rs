use std::sync::Arc;

use adhd_ranch_commands::{
    Commands, CreateFocusInput, CreatedFocus, CreatedProposal, DecisionOutcome, ProposalEdit,
};
use adhd_ranch_domain::{Caps, Focus, Proposal};

use tauri::State;

use crate::api::Health;

pub struct CommandsState(pub Arc<Commands>);

#[tauri::command]
pub fn health() -> Health {
    Health { ok: true }
}

#[tauri::command]
pub fn list_focuses(state: State<'_, CommandsState>) -> Result<Vec<Focus>, String> {
    state.0.list_focuses().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn list_proposals(state: State<'_, CommandsState>) -> Result<Vec<Proposal>, String> {
    state.0.list_proposals().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn create_focus(
    title: String,
    description: Option<String>,
    state: State<'_, CommandsState>,
) -> Result<CreatedFocus, String> {
    state
        .0
        .create_focus(CreateFocusInput {
            title,
            description: description.unwrap_or_default(),
        })
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_focus(focus_id: String, state: State<'_, CommandsState>) -> Result<(), String> {
    state.0.delete_focus(&focus_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn append_task(
    focus_id: String,
    text: String,
    state: State<'_, CommandsState>,
) -> Result<(), String> {
    state
        .0
        .append_task(&focus_id, &text)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_task(
    focus_id: String,
    index: usize,
    state: State<'_, CommandsState>,
) -> Result<(), String> {
    state
        .0
        .delete_task(&focus_id, index)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_caps(state: State<'_, CommandsState>) -> Caps {
    state.0.caps()
}

#[tauri::command]
pub fn accept_proposal(
    id: String,
    edit: Option<ProposalEdit>,
    state: State<'_, CommandsState>,
) -> Result<DecisionOutcome, String> {
    state
        .0
        .accept_proposal(&id, edit.unwrap_or_default())
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn reject_proposal(
    id: String,
    state: State<'_, CommandsState>,
) -> Result<DecisionOutcome, String> {
    state.0.reject_proposal(&id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn create_proposal(
    input: adhd_ranch_commands::CreateProposalInput,
    state: State<'_, CommandsState>,
) -> Result<CreatedProposal, String> {
    state.0.create_proposal(input).map_err(|e| e.to_string())
}
