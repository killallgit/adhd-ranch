use adhd_ranch_commands::{CreateFocusInput, CreatedFocus};
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::Json;
use axum::routing::{delete, get, post};
use axum::Router;
use serde::Deserialize;

use super::{ApiError, AppState, FocusCatalogEntry};

pub(super) fn routes() -> Router<AppState> {
    Router::new()
        .route("/focuses", get(list).post(create))
        .route("/focuses/:id", delete(delete_focus))
        .route("/focuses/:id/tasks", post(append_task))
        .route("/focuses/:id/tasks/:idx", delete(delete_task))
}

async fn list(State(state): State<AppState>) -> Result<Json<Vec<FocusCatalogEntry>>, ApiError> {
    let focuses = state.commands.list_focuses().map_err(ApiError::from)?;
    let catalog = focuses
        .into_iter()
        .map(|f| FocusCatalogEntry {
            id: f.id.0,
            title: f.title,
            description: f.description,
        })
        .collect();
    Ok(Json(catalog))
}

async fn create(
    State(state): State<AppState>,
    Json(req): Json<CreateFocusInput>,
) -> Result<(StatusCode, Json<CreatedFocus>), ApiError> {
    let created = state.commands.create_focus(req).map_err(ApiError::from)?;
    Ok((StatusCode::CREATED, Json(created)))
}

async fn delete_focus(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, ApiError> {
    state.commands.delete_focus(&id).map_err(ApiError::from)?;
    Ok(StatusCode::NO_CONTENT)
}

#[derive(Debug, Deserialize)]
struct AppendTaskRequest {
    text: String,
}

async fn append_task(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<AppendTaskRequest>,
) -> Result<StatusCode, ApiError> {
    state
        .commands
        .append_task(&id, &req.text)
        .map_err(ApiError::from)?;
    Ok(StatusCode::CREATED)
}

async fn delete_task(
    State(state): State<AppState>,
    Path((id, idx)): Path<(String, usize)>,
) -> Result<StatusCode, ApiError> {
    state
        .commands
        .delete_task(&id, idx)
        .map_err(ApiError::from)?;
    Ok(StatusCode::NO_CONTENT)
}
