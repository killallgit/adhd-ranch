use std::sync::Arc;

use adhd_ranch_commands::{Clock, CommandError, Commands, IdGen, ProposalDispatcher};
use adhd_ranch_domain::Settings;
use adhd_ranch_storage::{DecisionLog, FocusStore, ProposalQueue};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Json};
use axum::Router;
use serde::Serialize;
use time::format_description::well_known::Rfc3339;

mod focuses;
mod health;
mod proposals;

#[derive(Debug, Clone, Serialize)]
pub struct FocusCatalogEntry {
    pub id: String,
    pub title: String,
    pub description: String,
}

#[derive(Clone, Default)]
pub struct ServerDeps {
    pub clock: Option<Clock>,
    pub id_gen: Option<IdGen>,
    pub settings: Option<Settings>,
}

#[derive(Clone)]
pub(crate) struct AppState {
    pub(crate) commands: Arc<Commands>,
}

pub fn router(
    store: Arc<dyn FocusStore>,
    queue: Arc<dyn ProposalQueue>,
    decisions: Arc<dyn DecisionLog>,
    dispatcher: Arc<ProposalDispatcher>,
) -> Router {
    router_with(store, queue, decisions, dispatcher, ServerDeps::default())
}

pub fn router_with(
    store: Arc<dyn FocusStore>,
    queue: Arc<dyn ProposalQueue>,
    decisions: Arc<dyn DecisionLog>,
    dispatcher: Arc<ProposalDispatcher>,
    deps: ServerDeps,
) -> Router {
    let clock: Clock = deps.clock.unwrap_or_else(|| Arc::new(now_rfc3339));
    let id_gen: IdGen = deps
        .id_gen
        .unwrap_or_else(|| Arc::new(|| uuid::Uuid::now_v7().to_string()));
    let settings = deps.settings.unwrap_or_default();

    let commands = Arc::new(Commands::new(
        store, queue, decisions, dispatcher, clock, id_gen, settings,
    ));
    let state = AppState { commands };

    Router::new()
        .merge(health::routes())
        .merge(focuses::routes())
        .merge(proposals::routes())
        .with_state(state)
}

fn now_rfc3339() -> String {
    time::OffsetDateTime::now_utc()
        .format(&Rfc3339)
        .unwrap_or_else(|_| String::new())
}

#[derive(Debug)]
pub(crate) struct ApiError {
    status: StatusCode,
    message: String,
}

impl From<CommandError> for ApiError {
    fn from(e: CommandError) -> Self {
        let status = match e {
            CommandError::BadRequest(_) | CommandError::Validation(_) => StatusCode::BAD_REQUEST,
            CommandError::NotFound(_) => StatusCode::NOT_FOUND,
            CommandError::AlreadyExists(_) => StatusCode::CONFLICT,
            CommandError::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
        };
        Self {
            status,
            message: e.to_string(),
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        (
            self.status,
            Json(serde_json::json!({"error": self.message})),
        )
            .into_response()
    }
}

#[cfg(test)]
mod tests;
