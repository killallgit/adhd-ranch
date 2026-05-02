use std::sync::Arc;

use adhd_ranch_commands::{
    CommandError, Commands, CreateFocusInput, CreatedFocus, CreatedProposal, DecisionOutcome,
    ProposalEdit,
};
use adhd_ranch_domain::{Caps, Focus, Proposal};

use tauri::State;

use adhd_ranch_domain::{PigRect, RectUpdater};

use crate::api::Health;

pub struct CommandsState(pub Arc<Commands>);

pub struct PigHitState(pub Arc<dyn RectUpdater>);

#[tauri::command]
pub fn health() -> Health {
    Health { ok: true }
}

#[tauri::command]
pub fn list_focuses(state: State<'_, CommandsState>) -> Result<Vec<Focus>, CommandError> {
    state
        .0
        .list_focuses()
        .inspect_err(|e| log::error!("list_focuses: {e}"))
}

#[tauri::command]
pub fn list_proposals(state: State<'_, CommandsState>) -> Result<Vec<Proposal>, CommandError> {
    state
        .0
        .list_proposals()
        .inspect_err(|e| log::error!("list_proposals: {e}"))
}

#[tauri::command]
pub fn create_focus(
    title: String,
    description: Option<String>,
    state: State<'_, CommandsState>,
) -> Result<CreatedFocus, CommandError> {
    state
        .0
        .create_focus(CreateFocusInput {
            title: title.clone(),
            description: description.unwrap_or_default(),
        })
        .inspect(|f| log::info!("focus created: {}", f.id))
        .inspect_err(|e| log::error!("create_focus({title:?}): {e}"))
}

#[tauri::command]
pub fn delete_focus(focus_id: String, state: State<'_, CommandsState>) -> Result<(), CommandError> {
    state
        .0
        .delete_focus(&focus_id)
        .inspect(|_| log::info!("focus deleted: {focus_id}"))
        .inspect_err(|e| log::error!("delete_focus({focus_id:?}): {e}"))
}

#[tauri::command]
pub fn append_task(
    focus_id: String,
    text: String,
    state: State<'_, CommandsState>,
) -> Result<(), CommandError> {
    state
        .0
        .append_task(&focus_id, &text)
        .inspect(|_| log::info!("task appended to {focus_id}"))
        .inspect_err(|e| log::error!("append_task({focus_id:?}): {e}"))
}

#[tauri::command]
pub fn delete_task(
    focus_id: String,
    index: usize,
    state: State<'_, CommandsState>,
) -> Result<(), CommandError> {
    state
        .0
        .delete_task(&focus_id, index)
        .inspect(|_| log::info!("task {index} deleted from {focus_id}"))
        .inspect_err(|e| log::error!("delete_task({focus_id:?}, {index}): {e}"))
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
) -> Result<DecisionOutcome, CommandError> {
    state
        .0
        .accept_proposal(&id, edit.unwrap_or_default())
        .inspect(|o| log::info!("proposal {id} accepted → {:?}", o.target))
        .inspect_err(|e| log::error!("accept_proposal({id:?}): {e}"))
}

#[tauri::command]
pub fn reject_proposal(
    id: String,
    state: State<'_, CommandsState>,
) -> Result<DecisionOutcome, CommandError> {
    state
        .0
        .reject_proposal(&id)
        .inspect(|_| log::info!("proposal {id} rejected"))
        .inspect_err(|e| log::error!("reject_proposal({id:?}): {e}"))
}

#[tauri::command]
pub fn create_proposal(
    input: adhd_ranch_commands::CreateProposalInput,
    state: State<'_, CommandsState>,
) -> Result<CreatedProposal, CommandError> {
    state
        .0
        .create_proposal(input)
        .inspect(|p| log::info!("proposal created: {}", p.id))
        .inspect_err(|e| log::error!("create_proposal: {e}"))
}

#[tauri::command]
pub fn update_pig_rects(
    window: tauri::WebviewWindow,
    rects: Vec<PigRect>,
    state: State<'_, PigHitState>,
) {
    state.0.update_rects(window.label(), rects);
}
