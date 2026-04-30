use std::sync::Arc;

use adhd_ranch_domain::{Proposal, ProposalId, ProposalKind, ProposalValidationError};
use adhd_ranch_storage::{FocusRepository, ProposalQueue};
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Json};
use axum::routing::get;
use axum::Router;
use serde::{Deserialize, Serialize};
use time::format_description::well_known::Rfc3339;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FocusCatalogEntry {
    pub id: String,
    pub title: String,
    pub description: String,
}

#[derive(Clone)]
struct AppState {
    repo: Arc<dyn FocusRepository>,
    queue: Arc<dyn ProposalQueue>,
    clock: Clock,
    id_gen: IdGen,
}

type Clock = Arc<dyn Fn() -> String + Send + Sync>;
type IdGen = Arc<dyn Fn() -> String + Send + Sync>;

#[derive(Clone, Default)]
pub struct ServerDeps {
    pub clock: Option<Clock>,
    pub id_gen: Option<IdGen>,
}

pub fn router(repo: Arc<dyn FocusRepository>, queue: Arc<dyn ProposalQueue>) -> Router {
    router_with(repo, queue, ServerDeps::default())
}

pub fn router_with(
    repo: Arc<dyn FocusRepository>,
    queue: Arc<dyn ProposalQueue>,
    deps: ServerDeps,
) -> Router {
    let clock: Clock = deps.clock.unwrap_or_else(|| Arc::new(now_rfc3339));
    let id_gen: IdGen = deps
        .id_gen
        .unwrap_or_else(|| Arc::new(|| uuid::Uuid::now_v7().to_string()));

    let state = AppState {
        repo,
        queue,
        clock,
        id_gen,
    };
    Router::new()
        .route("/health", get(health))
        .route("/focuses", get(list_focuses))
        .route("/proposals", get(list_proposals).post(create_proposal))
        .with_state(state)
}

fn now_rfc3339() -> String {
    time::OffsetDateTime::now_utc()
        .format(&Rfc3339)
        .unwrap_or_else(|_| String::new())
}

async fn health() -> impl IntoResponse {
    Json(serde_json::json!({"ok": true}))
}

async fn list_focuses(
    State(state): State<AppState>,
) -> Result<Json<Vec<FocusCatalogEntry>>, ApiError> {
    let focuses = state.repo.list().map_err(ApiError::internal)?;
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

#[derive(Debug, Deserialize)]
pub struct CreateProposalRequest {
    pub kind: String,
    pub target_focus_id: Option<String>,
    pub task_text: Option<String>,
    pub new_focus: Option<adhd_ranch_domain::NewFocus>,
    pub summary: String,
    pub reasoning: String,
}

#[derive(Debug, Serialize)]
pub struct CreateProposalResponse {
    pub id: String,
}

async fn create_proposal(
    State(state): State<AppState>,
    Json(req): Json<CreateProposalRequest>,
) -> Result<(StatusCode, Json<CreateProposalResponse>), ApiError> {
    let kind = match req.kind.as_str() {
        "add_task" => ProposalKind::AddTask {
            target_focus_id: req.target_focus_id.clone().unwrap_or_default(),
            task_text: req.task_text.clone().unwrap_or_default(),
        },
        "new_focus" => ProposalKind::NewFocus {
            new_focus: req
                .new_focus
                .clone()
                .unwrap_or(adhd_ranch_domain::NewFocus {
                    title: String::new(),
                    description: String::new(),
                }),
        },
        "discard" => ProposalKind::Discard,
        other => {
            return Err(ApiError::bad_request(format!("unknown kind: {other}")));
        }
    };

    let id = (state.id_gen)();
    let proposal = Proposal {
        id: ProposalId(id.clone()),
        kind,
        summary: req.summary,
        reasoning: req.reasoning,
        created_at: (state.clock)(),
    };

    proposal.validate().map_err(ApiError::validation)?;
    state.queue.append(&proposal).map_err(ApiError::internal)?;

    Ok((StatusCode::CREATED, Json(CreateProposalResponse { id })))
}

async fn list_proposals(State(state): State<AppState>) -> Result<Json<Vec<Proposal>>, ApiError> {
    state.queue.list().map(Json).map_err(ApiError::internal)
}

#[derive(Debug)]
struct ApiError {
    status: StatusCode,
    message: String,
}

impl ApiError {
    fn internal<E: std::fmt::Display>(e: E) -> Self {
        Self {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            message: e.to_string(),
        }
    }

    fn bad_request(message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::BAD_REQUEST,
            message: message.into(),
        }
    }

    fn validation(e: ProposalValidationError) -> Self {
        Self::bad_request(e.to_string())
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
mod tests {
    use super::*;
    use adhd_ranch_storage::{JsonlProposalQueue, MarkdownFocusRepository};
    use axum::body::Body;
    use axum::http::Request;
    use http_body_util::BodyExt;
    use std::fs;
    use tempfile::TempDir;
    use tower::ServiceExt;

    fn fixed_clock(stamp: &'static str) -> Clock {
        Arc::new(move || stamp.to_string())
    }

    fn fixed_id(value: &'static str) -> IdGen {
        Arc::new(move || value.to_string())
    }

    fn write_focus(root: &std::path::Path, slug: &str, body: &str) {
        let dir = root.join(slug);
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("focus.md"), body).unwrap();
    }

    fn focus_md(id: &str, title: &str, description: &str, created_at: &str) -> String {
        format!("---\nid: {id}\ntitle: {title}\ndescription: {description}\ncreated_at: {created_at}\n---\n- [ ] one\n")
    }

    fn make_app(dir: &std::path::Path) -> (Router, std::path::PathBuf) {
        let repo = Arc::new(MarkdownFocusRepository::new(dir.join("focuses")));
        let queue_path = dir.join("proposals.jsonl");
        let queue = Arc::new(JsonlProposalQueue::new(queue_path.clone()));
        let app = router_with(
            repo,
            queue,
            ServerDeps {
                clock: Some(fixed_clock("2026-04-30T12:00:00Z")),
                id_gen: Some(fixed_id("p-test")),
            },
        );
        (app, queue_path)
    }

    #[tokio::test]
    async fn health_returns_ok() {
        let dir = TempDir::new().unwrap();
        let (app, _) = make_app(dir.path());
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
    }

    #[tokio::test]
    async fn focuses_returns_catalog_projection_only() {
        let dir = TempDir::new().unwrap();
        write_focus(
            &dir.path().join("focuses"),
            "a",
            &focus_md("a", "Alpha", "first", "2026-04-30T12:00:00Z"),
        );
        let (app, _) = make_app(dir.path());
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
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].id, "a");
        let raw: Vec<serde_json::Value> = serde_json::from_slice(&bytes).unwrap();
        assert!(raw[0].get("tasks").is_none());
    }

    #[tokio::test]
    async fn post_proposal_validates_payload_and_persists() {
        let dir = TempDir::new().unwrap();
        let (app, queue_path) = make_app(dir.path());
        let body = serde_json::json!({
            "kind": "add_task",
            "target_focus_id": "f1",
            "task_text": "ship it",
            "summary": "did a thing",
            "reasoning": "fits"
        });
        let resp = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/proposals")
                    .header("content-type", "application/json")
                    .body(Body::from(body.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::CREATED);
        let bytes = resp.into_body().collect().await.unwrap().to_bytes();
        let payload: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(payload["id"], "p-test");

        let on_disk = std::fs::read_to_string(queue_path).unwrap();
        assert!(on_disk.contains("p-test"));
        assert!(on_disk.contains("ship it"));
    }

    #[tokio::test]
    async fn post_proposal_rejects_empty_summary_with_400() {
        let dir = TempDir::new().unwrap();
        let (app, _) = make_app(dir.path());
        let body = serde_json::json!({
            "kind": "discard",
            "summary": "",
            "reasoning": "x"
        });
        let resp = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/proposals")
                    .header("content-type", "application/json")
                    .body(Body::from(body.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn post_proposal_rejects_unknown_kind_with_400() {
        let dir = TempDir::new().unwrap();
        let (app, _) = make_app(dir.path());
        let body = serde_json::json!({
            "kind": "merge_focus",
            "summary": "x",
            "reasoning": "x"
        });
        let resp = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/proposals")
                    .header("content-type", "application/json")
                    .body(Body::from(body.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn get_proposals_returns_listed_items() {
        let dir = TempDir::new().unwrap();
        let (app, _) = make_app(dir.path());

        let body = serde_json::json!({
            "kind": "discard",
            "summary": "thing",
            "reasoning": "why"
        });
        let _ = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/proposals")
                    .header("content-type", "application/json")
                    .body(Body::from(body.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();

        let resp = app
            .oneshot(
                Request::builder()
                    .uri("/proposals")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let bytes = resp.into_body().collect().await.unwrap().to_bytes();
        let listed: Vec<serde_json::Value> = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(listed.len(), 1);
        assert_eq!(listed[0]["id"], "p-test");
        assert_eq!(listed[0]["kind"], "discard");
    }
}
