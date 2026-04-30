use axum::response::{IntoResponse, Json};
use axum::routing::get;
use axum::Router;

use super::AppState;

pub(super) fn routes() -> Router<AppState> {
    Router::new().route("/health", get(health))
}

async fn health() -> impl IntoResponse {
    Json(serde_json::json!({"ok": true}))
}
