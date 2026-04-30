use adhd_ranch_commands::{CreateProposalInput, CreatedProposal, DecisionOutcome, ProposalEdit};
use adhd_ranch_domain::Proposal;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::Json;
use axum::routing::{get, post};
use axum::Router;

use super::{ApiError, AppState};

pub(super) fn routes() -> Router<AppState> {
    Router::new()
        .route("/proposals", get(list).post(create))
        .route("/proposals/:id/accept", post(accept))
        .route("/proposals/:id/reject", post(reject))
}

async fn list(State(state): State<AppState>) -> Result<Json<Vec<Proposal>>, ApiError> {
    state
        .commands
        .list_proposals()
        .map(Json)
        .map_err(ApiError::from)
}

async fn create(
    State(state): State<AppState>,
    Json(req): Json<CreateProposalInput>,
) -> Result<(StatusCode, Json<CreatedProposal>), ApiError> {
    let created = state
        .commands
        .create_proposal(req)
        .map_err(ApiError::from)?;
    Ok((StatusCode::CREATED, Json(created)))
}

async fn accept(
    State(state): State<AppState>,
    Path(id): Path<String>,
    body: Option<Json<ProposalEdit>>,
) -> Result<Json<DecisionOutcome>, ApiError> {
    let edit = body.map(|Json(e)| e).unwrap_or_default();
    let outcome = state
        .commands
        .accept_proposal(&id, edit)
        .map_err(ApiError::from)?;
    Ok(Json(outcome))
}

async fn reject(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<DecisionOutcome>, ApiError> {
    let outcome = state
        .commands
        .reject_proposal(&id)
        .map_err(ApiError::from)?;
    Ok(Json(outcome))
}
