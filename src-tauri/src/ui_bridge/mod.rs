use std::sync::Arc;

use adhd_ranch_domain::{
    Caps, Decision, DecisionKind, Focus, NewFocus, OverCapMonitor, Proposal, ProposalId, Settings,
};
use adhd_ranch_http_api::ProposalDispatcher;
use adhd_ranch_storage::{DecisionLog, FocusStore, ProposalQueue};

use tauri::State;
use time::format_description::well_known::Rfc3339;

use crate::api::Health;

pub struct FocusStoreState(pub Arc<dyn FocusStore>);
pub struct ProposalQueueState(pub Arc<dyn ProposalQueue>);
pub struct DecisionLogState(pub Arc<dyn DecisionLog>);
pub struct DispatcherState(pub Arc<ProposalDispatcher>);
pub struct SettingsState(pub Settings);
pub struct MonitorState(pub Arc<OverCapMonitor>);

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
pub fn list_focuses(state: State<'_, FocusStoreState>) -> Result<Vec<Focus>, String> {
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
    state: State<'_, FocusStoreState>,
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
pub fn delete_focus(focus_id: String, state: State<'_, FocusStoreState>) -> Result<(), String> {
    state.0.delete_focus(&focus_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn append_task(
    focus_id: String,
    text: String,
    state: State<'_, FocusStoreState>,
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
    state: State<'_, FocusStoreState>,
) -> Result<(), String> {
    state
        .0
        .delete_task(&focus_id, index)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_caps(state: State<'_, SettingsState>) -> Caps {
    state.0.caps
}

#[derive(Debug, Default, serde::Deserialize)]
pub struct ProposalEdit {
    pub target_focus_id: Option<String>,
    pub task_text: Option<String>,
    pub new_focus: Option<NewFocus>,
}

#[tauri::command]
pub fn accept_proposal(
    id: String,
    edit: Option<ProposalEdit>,
    queue: State<'_, ProposalQueueState>,
    dispatcher: State<'_, DispatcherState>,
    decisions: State<'_, DecisionLogState>,
) -> Result<DecisionResponse, String> {
    let original = load(&queue.0, &id)?;
    let (proposal, edited) = apply_edit(original, &edit.unwrap_or_default());
    proposal.validate().map_err(|e| e.to_string())?;
    let outcome = dispatcher.0.apply(&proposal).map_err(|e| e.to_string())?;
    log_decision(
        &decisions.0,
        &proposal,
        DecisionKind::Accept,
        outcome.target.clone(),
        edited,
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
    log_decision(&decisions.0, &proposal, DecisionKind::Reject, None, false)?;
    queue.0.remove(&proposal.id).map_err(|e| e.to_string())?;
    Ok(DecisionResponse { id, target: None })
}

fn apply_edit(mut proposal: Proposal, edit: &ProposalEdit) -> (Proposal, bool) {
    use adhd_ranch_domain::ProposalKind;
    let mut edited = false;
    match &mut proposal.kind {
        ProposalKind::AddTask {
            target_focus_id,
            task_text,
        } => {
            if let Some(new_id) = edit.target_focus_id.as_ref() {
                if new_id != target_focus_id {
                    *target_focus_id = new_id.clone();
                    edited = true;
                }
            }
            if let Some(new_text) = edit.task_text.as_ref() {
                if new_text != task_text {
                    *task_text = new_text.clone();
                    edited = true;
                }
            }
        }
        ProposalKind::NewFocus { new_focus } => {
            if let Some(replacement) = edit.new_focus.as_ref() {
                if replacement != new_focus {
                    *new_focus = replacement.clone();
                    edited = true;
                }
            }
        }
        ProposalKind::Discard => {}
    }
    (proposal, edited)
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
    edited: bool,
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
        edited,
    };
    log.append(&decision).map_err(|e| e.to_string())
}
