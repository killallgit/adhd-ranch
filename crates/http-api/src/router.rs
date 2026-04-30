use std::sync::Arc;

use adhd_ranch_storage::FocusRepository;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Json};
use axum::routing::get;
use axum::Router;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FocusCatalogEntry {
    pub id: String,
    pub title: String,
    pub description: String,
}

#[derive(Clone)]
struct AppState {
    repo: Arc<dyn FocusRepository>,
}

pub fn router(repo: Arc<dyn FocusRepository>) -> Router {
    let state = AppState { repo };
    Router::new()
        .route("/health", get(health))
        .route("/focuses", get(list_focuses))
        .with_state(state)
}

async fn health() -> impl IntoResponse {
    Json(serde_json::json!({"ok": true}))
}

async fn list_focuses(
    State(state): State<AppState>,
) -> Result<Json<Vec<FocusCatalogEntry>>, ApiError> {
    let focuses = state.repo.list().map_err(ApiError::from)?;
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

#[derive(Debug)]
struct ApiError(String);

impl<E: std::error::Error> From<E> for ApiError {
    fn from(error: E) -> Self {
        Self(error.to_string())
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": self.0})),
        )
            .into_response()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use adhd_ranch_storage::MarkdownFocusRepository;
    use axum::body::Body;
    use axum::http::Request;
    use http_body_util::BodyExt;
    use std::fs;
    use tempfile::TempDir;
    use tower::ServiceExt;

    fn write_focus(root: &std::path::Path, slug: &str, body: &str) {
        let dir = root.join(slug);
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("focus.md"), body).unwrap();
    }

    fn focus_md(id: &str, title: &str, description: &str, created_at: &str) -> String {
        format!("---\nid: {id}\ntitle: {title}\ndescription: {description}\ncreated_at: {created_at}\n---\n- [ ] one\n")
    }

    #[tokio::test]
    async fn health_returns_200_with_ok_payload() {
        let dir = TempDir::new().unwrap();
        let repo = Arc::new(MarkdownFocusRepository::new(dir.path()));
        let app = router(repo);

        let resp = app
            .oneshot(
                Request::builder()
                    .uri("/health")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(resp.status(), StatusCode::OK);
        let bytes = resp.into_body().collect().await.unwrap().to_bytes();
        let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(json["ok"], true);
    }

    #[tokio::test]
    async fn focuses_returns_catalog_projection_only() {
        let dir = TempDir::new().unwrap();
        write_focus(
            dir.path(),
            "a",
            &focus_md("a", "Alpha", "first focus", "2026-04-30T12:00:00Z"),
        );
        write_focus(
            dir.path(),
            "b",
            &focus_md("b", "Beta", "second focus", "2026-04-30T12:01:00Z"),
        );
        let repo = Arc::new(MarkdownFocusRepository::new(dir.path()));
        let app = router(repo);

        let resp = app
            .oneshot(
                Request::builder()
                    .uri("/focuses")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(resp.status(), StatusCode::OK);
        let bytes = resp.into_body().collect().await.unwrap().to_bytes();
        let entries: Vec<FocusCatalogEntry> = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].id, "a");
        assert_eq!(entries[0].title, "Alpha");
        assert_eq!(entries[0].description, "first focus");
        assert_eq!(entries[1].title, "Beta");

        let raw: Vec<serde_json::Value> = serde_json::from_slice(&bytes).unwrap();
        assert!(raw[0].get("tasks").is_none(), "catalog must not leak tasks");
        assert!(
            raw[0].get("created_at").is_none(),
            "catalog must not leak created_at"
        );
    }

    #[tokio::test]
    async fn focuses_empty_when_root_missing() {
        let dir = TempDir::new().unwrap();
        let repo = Arc::new(MarkdownFocusRepository::new(dir.path().join("missing")));
        let app = router(repo);

        let resp = app
            .oneshot(
                Request::builder()
                    .uri("/focuses")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(resp.status(), StatusCode::OK);
        let bytes = resp.into_body().collect().await.unwrap().to_bytes();
        let entries: Vec<FocusCatalogEntry> = serde_json::from_slice(&bytes).unwrap();
        assert!(entries.is_empty());
    }
}
