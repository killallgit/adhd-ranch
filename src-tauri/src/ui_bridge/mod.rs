use std::sync::Arc;

use adhd_ranch_domain::{Decision, DecisionKind, Focus, NewFocus, Proposal, ProposalId};
use adhd_ranch_http_api::ProposalDispatcher;
use adhd_ranch_storage::{DecisionLog, FocusRepository, FocusWriter, ProposalQueue};
use tauri::State;
use time::format_description::well_known::Rfc3339;

use crate::api::Health;

pub struct FocusRepoState(pub Arc<dyn FocusRepository>);
pub struct FocusWriterState(pub Arc<dyn FocusWriter>);
pub struct ProposalQueueState(pub Arc<dyn ProposalQueue>);
pub struct DecisionLogState(pub Arc<dyn DecisionLog>);
pub struct DispatcherState(pub Arc<ProposalDispatcher>);

#[derive(serde::Serialize)]
pub struct DecisionResponse {
    pub id: String,
    pub target: Option<String>,
}

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

#[derive(serde::Serialize)]
pub struct CreatedFocus {
    pub id: String,
}

#[tauri::command]
pub fn create_focus(
    title: String,
    description: Option<String>,
    state: State<'_, FocusWriterState>,
) -> Result<CreatedFocus, String> {
    if title.trim().is_empty() {
        return Err("title must not be empty".into());
    }
    let id = uuid::Uuid::now_v7().to_string();
    let created_at = time::OffsetDateTime::now_utc()
        .format(&Rfc3339)
        .unwrap_or_default();
    let new_focus = NewFocus {
        title,
        description: description.unwrap_or_default(),
    };
    let slug = state
        .0
        .create_focus(&new_focus, &id, &created_at)
        .map_err(|e| e.to_string())?;
    Ok(CreatedFocus { id: slug })
}

#[tauri::command]
pub fn delete_focus(focus_id: String, state: State<'_, FocusWriterState>) -> Result<(), String> {
    state.0.delete_focus(&focus_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn append_task(
    focus_id: String,
    text: String,
    state: State<'_, FocusWriterState>,
) -> Result<(), String> {
    if text.trim().is_empty() {
        return Err("text must not be empty".into());
    }
    state
        .0
        .append_task(&focus_id, &text)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_task(
    focus_id: String,
    index: usize,
    state: State<'_, FocusWriterState>,
) -> Result<(), String> {
    state
        .0
        .delete_task(&focus_id, index)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn accept_proposal(
    id: String,
    queue: State<'_, ProposalQueueState>,
    dispatcher: State<'_, DispatcherState>,
    decisions: State<'_, DecisionLogState>,
) -> Result<DecisionResponse, String> {
    let proposal = load(&queue.0, &id)?;
    let outcome = dispatcher.0.apply(&proposal).map_err(|e| e.to_string())?;
    log_decision(
        &decisions.0,
        &proposal,
        DecisionKind::Accept,
        outcome.target.clone(),
    )?;
    queue.0.remove(&proposal.id).map_err(|e| e.to_string())?;
    Ok(DecisionResponse {
        id,
        target: outcome.target,
    })
}

#[tauri::command]
pub fn reject_proposal(
    id: String,
    queue: State<'_, ProposalQueueState>,
    decisions: State<'_, DecisionLogState>,
) -> Result<DecisionResponse, String> {
    let proposal = load(&queue.0, &id)?;
    log_decision(&decisions.0, &proposal, DecisionKind::Reject, None)?;
    queue.0.remove(&proposal.id).map_err(|e| e.to_string())?;
    Ok(DecisionResponse { id, target: None })
}

fn load(queue: &Arc<dyn ProposalQueue>, id: &str) -> Result<Proposal, String> {
    queue
        .find(&ProposalId(id.to_string()))
        .map_err(|e| e.to_string())?
        .ok_or_else(|| format!("proposal not found: {id}"))
}

fn log_decision(
    log: &Arc<dyn DecisionLog>,
    proposal: &Proposal,
    kind: DecisionKind,
    target: Option<String>,
) -> Result<(), String> {
    let ts = time::OffsetDateTime::now_utc()
        .format(&Rfc3339)
        .unwrap_or_default();
    let decision = Decision {
        ts,
        proposal_id: proposal.id.0.clone(),
        decision: kind,
        reasoning: proposal.reasoning.clone(),
        target,
    };
    log.append(&decision).map_err(|e| e.to_string())
}
