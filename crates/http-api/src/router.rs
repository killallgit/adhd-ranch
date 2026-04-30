use std::sync::Arc;

use adhd_ranch_domain::{
    Decision, DecisionKind, Proposal, ProposalId, ProposalKind, ProposalValidationError,
};
use adhd_ranch_storage::{DecisionLog, FocusRepository, FocusWriter, ProposalQueue};
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Json};
use axum::routing::{delete, get, post};
use axum::Router;
use serde::{Deserialize, Serialize};
use time::format_description::well_known::Rfc3339;

use crate::applier::ProposalDispatcher;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FocusCatalogEntry {
    pub id: String,
    pub title: String,
    pub description: String,
}

type Clock = Arc<dyn Fn() -> String + Send + Sync>;
type IdGen = Arc<dyn Fn() -> String + Send + Sync>;

#[derive(Clone)]
struct AppState {
    repo: Arc<dyn FocusRepository>,
    writer: Arc<dyn FocusWriter>,
    queue: Arc<dyn ProposalQueue>,
    decisions: Arc<dyn DecisionLog>,
    dispatcher: Arc<ProposalDispatcher>,
    clock: Clock,
    id_gen: IdGen,
}

#[derive(Clone, Default)]
pub struct ServerDeps {
    pub clock: Option<Clock>,
    pub id_gen: Option<IdGen>,
}

pub fn router(
    repo: Arc<dyn FocusRepository>,
    writer: Arc<dyn FocusWriter>,
    queue: Arc<dyn ProposalQueue>,
    decisions: Arc<dyn DecisionLog>,
    dispatcher: Arc<ProposalDispatcher>,
) -> Router {
    router_with(
        repo,
        writer,
        queue,
        decisions,
        dispatcher,
        ServerDeps::default(),
    )
}

pub fn router_with(
    repo: Arc<dyn FocusRepository>,
    writer: Arc<dyn FocusWriter>,
    queue: Arc<dyn ProposalQueue>,
    decisions: Arc<dyn DecisionLog>,
    dispatcher: Arc<ProposalDispatcher>,
    deps: ServerDeps,
) -> Router {
    let clock: Clock = deps.clock.unwrap_or_else(|| Arc::new(now_rfc3339));
    let id_gen: IdGen = deps
        .id_gen
        .unwrap_or_else(|| Arc::new(|| uuid::Uuid::now_v7().to_string()));

    let state = AppState {
        repo,
        writer,
        queue,
        decisions,
        dispatcher,
        clock,
        id_gen,
    };
    Router::new()
        .route("/health", get(health))
        .route("/focuses", get(list_focuses).post(create_focus))
        .route("/focuses/:id", delete(delete_focus))
        .route("/focuses/:id/tasks", post(append_task))
        .route("/focuses/:id/tasks/:idx", delete(delete_task))
        .route("/proposals", get(list_proposals).post(create_proposal))
        .route("/proposals/:id/accept", post(accept_proposal))
        .route("/proposals/:id/reject", post(reject_proposal))
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
pub struct CreateFocusRequest {
    pub title: String,
    #[serde(default)]
    pub description: String,
}

#[derive(Debug, Serialize)]
pub struct CreateFocusResponse {
    pub id: String,
}

async fn create_focus(
    State(state): State<AppState>,
    Json(req): Json<CreateFocusRequest>,
) -> Result<(StatusCode, Json<CreateFocusResponse>), ApiError> {
    if req.title.trim().is_empty() {
        return Err(ApiError::bad_request("title must not be empty"));
    }
    let id = (state.id_gen)();
    let created_at = (state.clock)();
    let new_focus = adhd_ranch_domain::NewFocus {
        title: req.title,
        description: req.description,
    };
    let slug = state
        .writer
        .create_focus(&new_focus, &id, &created_at)
        .map_err(ApiError::from_writer)?;
    Ok((StatusCode::CREATED, Json(CreateFocusResponse { id: slug })))
}

async fn delete_focus(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, ApiError> {
    state
        .writer
        .delete_focus(&id)
        .map_err(ApiError::from_writer)?;
    Ok(StatusCode::NO_CONTENT)
}

#[derive(Debug, Deserialize)]
pub struct AppendTaskRequest {
    pub text: String,
}

async fn append_task(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<AppendTaskRequest>,
) -> Result<StatusCode, ApiError> {
    if req.text.trim().is_empty() {
        return Err(ApiError::bad_request("text must not be empty"));
    }
    state
        .writer
        .append_task(&id, &req.text)
        .map_err(ApiError::from_writer)?;
    Ok(StatusCode::CREATED)
}

async fn delete_task(
    State(state): State<AppState>,
    Path((id, idx)): Path<(String, usize)>,
) -> Result<StatusCode, ApiError> {
    state
        .writer
        .delete_task(&id, idx)
        .map_err(ApiError::from_writer)?;
    Ok(StatusCode::NO_CONTENT)
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

#[derive(Debug, Serialize)]
struct DecisionResponse {
    pub id: String,
    pub target: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
pub struct ProposalEdit {
    pub target_focus_id: Option<String>,
    pub task_text: Option<String>,
    pub new_focus: Option<adhd_ranch_domain::NewFocus>,
}

async fn accept_proposal(
    State(state): State<AppState>,
    Path(id): Path<String>,
    body: Option<Json<ProposalEdit>>,
) -> Result<Json<DecisionResponse>, ApiError> {
    let original = load_proposal(&state, &id)?;
    let edit = body.map(|Json(e)| e).unwrap_or_default();
    let (proposal, edited) = apply_edit(original, &edit);
    proposal.validate().map_err(ApiError::validation)?;
    let outcome = state
        .dispatcher
        .apply(&proposal)
        .map_err(ApiError::internal)?;
    record_decision(
        &state,
        &proposal,
        DecisionKind::Accept,
        outcome.target.clone(),
        edited,
    )?;
    state
        .queue
        .remove(&proposal.id)
        .map_err(ApiError::internal)?;
    Ok(Json(DecisionResponse {
        id,
        target: outcome.target,
    }))
}

async fn reject_proposal(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<DecisionResponse>, ApiError> {
    let proposal = load_proposal(&state, &id)?;
    record_decision(&state, &proposal, DecisionKind::Reject, None, false)?;
    state
        .queue
        .remove(&proposal.id)
        .map_err(ApiError::internal)?;
    Ok(Json(DecisionResponse { id, target: None }))
}

fn apply_edit(mut proposal: Proposal, edit: &ProposalEdit) -> (Proposal, bool) {
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

fn load_proposal(state: &AppState, id: &str) -> Result<Proposal, ApiError> {
    state
        .queue
        .find(&ProposalId(id.to_string()))
        .map_err(ApiError::internal)?
        .ok_or_else(|| ApiError {
            status: StatusCode::NOT_FOUND,
            message: format!("proposal not found: {id}"),
        })
}

fn record_decision(
    state: &AppState,
    proposal: &Proposal,
    kind: DecisionKind,
    target: Option<String>,
    edited: bool,
) -> Result<(), ApiError> {
    let decision = Decision {
        ts: (state.clock)(),
        proposal_id: proposal.id.0.clone(),
        decision: kind,
        reasoning: proposal.reasoning.clone(),
        target,
        edited,
    };
    state
        .decisions
        .append(&decision)
        .map_err(ApiError::internal)
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

    fn from_writer(e: adhd_ranch_storage::WriterError) -> Self {
        use adhd_ranch_storage::WriterError;
        match e {
            WriterError::FocusNotFound(_) => Self {
                status: StatusCode::NOT_FOUND,
                message: e.to_string(),
            },
            WriterError::FocusAlreadyExists(_) => Self {
                status: StatusCode::CONFLICT,
                message: e.to_string(),
            },
            WriterError::TaskIndexOutOfRange { .. } => Self::bad_request(e.to_string()),
            WriterError::Io(_) => Self::internal(e),
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
mod tests {
    use super::*;
    use adhd_ranch_storage::{
        JsonlDecisionLog, JsonlProposalQueue, MarkdownFocusRepository, MarkdownFocusWriter,
    };
    use axum::body::Body;
    use axum::http::Request;
    use http_body_util::BodyExt;
    use std::fs;
    use tempfile::TempDir;
    use tower::ServiceExt;

    fn read_decisions_for_test(path: &std::path::Path) -> Vec<Decision> {
        std::fs::read_to_string(path)
            .map(|raw| {
                raw.lines()
                    .filter(|l| !l.trim().is_empty())
                    .map(|l| serde_json::from_str(l).unwrap())
                    .collect()
            })
            .unwrap_or_default()
    }

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

    struct Harness {
        app: Router,
        focuses_root: std::path::PathBuf,
        proposals_path: std::path::PathBuf,
        decisions_path: std::path::PathBuf,
    }

    fn make_app(dir: &std::path::Path) -> Harness {
        let focuses_root = dir.join("focuses");
        fs::create_dir_all(&focuses_root).unwrap();
        let repo = Arc::new(MarkdownFocusRepository::new(&focuses_root));
        let writer: Arc<dyn adhd_ranch_storage::FocusWriter> =
            Arc::new(MarkdownFocusWriter::new(&focuses_root));
        let proposals_path = dir.join("proposals.jsonl");
        let decisions_path = dir.join("decisions.jsonl");
        let queue = Arc::new(JsonlProposalQueue::new(proposals_path.clone()));
        let decisions = Arc::new(JsonlDecisionLog::new(decisions_path.clone()));
        let dispatcher = Arc::new(ProposalDispatcher::from_writer(
            writer.clone(),
            fixed_clock("2026-04-30T12:00:00Z"),
            fixed_id("focus-id-1"),
        ));
        let app = router_with(
            repo,
            writer,
            queue,
            decisions,
            dispatcher,
            ServerDeps {
                clock: Some(fixed_clock("2026-04-30T12:00:00Z")),
                id_gen: Some(fixed_id("p-test")),
            },
        );
        Harness {
            app,
            focuses_root,
            proposals_path,
            decisions_path,
        }
    }

    async fn post_json(
        app: &Router,
        uri: &str,
        body: serde_json::Value,
    ) -> axum::http::Response<Body> {
        app.clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(uri)
                    .header("content-type", "application/json")
                    .body(Body::from(body.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap()
    }

    async fn post_empty(app: &Router, uri: &str) -> axum::http::Response<Body> {
        app.clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(uri)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap()
    }

    #[tokio::test]
    async fn health_returns_ok() {
        let dir = TempDir::new().unwrap();
        let h = make_app(dir.path());
        let resp = h
            .app
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
        let h = make_app(dir.path());
        write_focus(
            &h.focuses_root,
            "a",
            &focus_md("a", "Alpha", "first", "2026-04-30T12:00:00Z"),
        );
        let resp = h
            .app
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
        let raw: Vec<serde_json::Value> = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(raw.len(), 1);
        assert!(raw[0].get("tasks").is_none());
    }

    #[tokio::test]
    async fn post_proposal_validates_payload_and_persists() {
        let dir = TempDir::new().unwrap();
        let h = make_app(dir.path());
        let resp = post_json(
            &h.app,
            "/proposals",
            serde_json::json!({
                "kind": "discard",
                "summary": "s",
                "reasoning": "r"
            }),
        )
        .await;
        assert_eq!(resp.status(), StatusCode::CREATED);
        let on_disk = std::fs::read_to_string(&h.proposals_path).unwrap();
        assert!(on_disk.contains("p-test"));
    }

    #[tokio::test]
    async fn post_proposal_rejects_empty_summary_with_400() {
        let dir = TempDir::new().unwrap();
        let h = make_app(dir.path());
        let resp = post_json(
            &h.app,
            "/proposals",
            serde_json::json!({"kind": "discard", "summary": "", "reasoning": "x"}),
        )
        .await;
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn post_proposal_rejects_unknown_kind_with_400() {
        let dir = TempDir::new().unwrap();
        let h = make_app(dir.path());
        let resp = post_json(
            &h.app,
            "/proposals",
            serde_json::json!({"kind": "merge_focus", "summary": "x", "reasoning": "x"}),
        )
        .await;
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn accept_add_task_appends_bullet_logs_decision_and_clears_queue() {
        let dir = TempDir::new().unwrap();
        let h = make_app(dir.path());
        write_focus(
            &h.focuses_root,
            "f1",
            &focus_md("f1", "F1", "x", "2026-04-30T12:00:00Z"),
        );

        let resp = post_json(
            &h.app,
            "/proposals",
            serde_json::json!({
                "kind": "add_task",
                "target_focus_id": "f1",
                "task_text": "ship it",
                "summary": "s",
                "reasoning": "r"
            }),
        )
        .await;
        assert_eq!(resp.status(), StatusCode::CREATED);

        let resp = post_empty(&h.app, "/proposals/p-test/accept").await;
        assert_eq!(resp.status(), StatusCode::OK);

        let focus = std::fs::read_to_string(h.focuses_root.join("f1/focus.md")).unwrap();
        assert!(focus.contains("- [ ] ship it"));

        let decisions = read_decisions_for_test(&h.decisions_path);
        assert_eq!(decisions.len(), 1);
        assert_eq!(decisions[0].decision, DecisionKind::Accept);
        assert_eq!(decisions[0].target.as_deref(), Some("f1"));

        let resp = h
            .app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/proposals")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        let bytes = resp.into_body().collect().await.unwrap().to_bytes();
        let listed: Vec<serde_json::Value> = serde_json::from_slice(&bytes).unwrap();
        assert!(listed.is_empty());
    }

    #[tokio::test]
    async fn accept_new_focus_creates_dir() {
        let dir = TempDir::new().unwrap();
        let h = make_app(dir.path());

        let _ = post_json(
            &h.app,
            "/proposals",
            serde_json::json!({
                "kind": "new_focus",
                "new_focus": {"title": "Customer X bug", "description": "ship"},
                "summary": "s",
                "reasoning": "r"
            }),
        )
        .await;

        let resp = post_empty(&h.app, "/proposals/p-test/accept").await;
        assert_eq!(resp.status(), StatusCode::OK);

        assert!(h.focuses_root.join("customer-x-bug/focus.md").exists());
        let decisions = read_decisions_for_test(&h.decisions_path);
        assert_eq!(decisions[0].target.as_deref(), Some("customer-x-bug"));
    }

    #[tokio::test]
    async fn reject_logs_and_clears_without_mutation() {
        let dir = TempDir::new().unwrap();
        let h = make_app(dir.path());

        let _ = post_json(
            &h.app,
            "/proposals",
            serde_json::json!({"kind": "discard", "summary": "s", "reasoning": "r"}),
        )
        .await;
        let resp = post_empty(&h.app, "/proposals/p-test/reject").await;
        assert_eq!(resp.status(), StatusCode::OK);

        let decisions = read_decisions_for_test(&h.decisions_path);
        assert_eq!(decisions.len(), 1);
        assert_eq!(decisions[0].decision, DecisionKind::Reject);
        assert_eq!(decisions[0].target, None);
    }

    #[tokio::test]
    async fn post_focus_creates_dir_and_returns_slug() {
        let dir = TempDir::new().unwrap();
        let h = make_app(dir.path());
        let resp = post_json(
            &h.app,
            "/focuses",
            serde_json::json!({"title": "API refactor", "description": "ship"}),
        )
        .await;
        assert_eq!(resp.status(), StatusCode::CREATED);
        let bytes = resp.into_body().collect().await.unwrap().to_bytes();
        let payload: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(payload["id"], "api-refactor");
        assert!(h.focuses_root.join("api-refactor/focus.md").exists());
    }

    #[tokio::test]
    async fn post_focus_rejects_empty_title_with_400() {
        let dir = TempDir::new().unwrap();
        let h = make_app(dir.path());
        let resp = post_json(
            &h.app,
            "/focuses",
            serde_json::json!({"title": "  ", "description": "x"}),
        )
        .await;
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn post_focus_collision_returns_409() {
        let dir = TempDir::new().unwrap();
        let h = make_app(dir.path());
        write_focus(
            &h.focuses_root,
            "api-refactor",
            &focus_md("api-refactor", "API refactor", "x", "2026-04-30T12:00:00Z"),
        );
        let resp = post_json(
            &h.app,
            "/focuses",
            serde_json::json!({"title": "API refactor"}),
        )
        .await;
        assert_eq!(resp.status(), StatusCode::CONFLICT);
    }

    #[tokio::test]
    async fn delete_focus_removes_dir_and_returns_204() {
        let dir = TempDir::new().unwrap();
        let h = make_app(dir.path());
        write_focus(
            &h.focuses_root,
            "api-refactor",
            &focus_md("api-refactor", "x", "x", "2026-04-30T12:00:00Z"),
        );
        let resp = h
            .app
            .clone()
            .oneshot(
                Request::builder()
                    .method("DELETE")
                    .uri("/focuses/api-refactor")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::NO_CONTENT);
        assert!(!h.focuses_root.join("api-refactor").exists());
    }

    #[tokio::test]
    async fn delete_focus_unknown_returns_404() {
        let dir = TempDir::new().unwrap();
        let h = make_app(dir.path());
        let resp = h
            .app
            .clone()
            .oneshot(
                Request::builder()
                    .method("DELETE")
                    .uri("/focuses/nope")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn post_task_appends_bullet_and_returns_201() {
        let dir = TempDir::new().unwrap();
        let h = make_app(dir.path());
        write_focus(
            &h.focuses_root,
            "api-refactor",
            &focus_md("api-refactor", "x", "x", "2026-04-30T12:00:00Z"),
        );
        let resp = post_json(
            &h.app,
            "/focuses/api-refactor/tasks",
            serde_json::json!({"text": "extract pipeline"}),
        )
        .await;
        assert_eq!(resp.status(), StatusCode::CREATED);
        let content =
            std::fs::read_to_string(h.focuses_root.join("api-refactor/focus.md")).unwrap();
        assert!(content.contains("- [ ] extract pipeline"));
    }

    #[tokio::test]
    async fn delete_task_by_index_removes_bullet() {
        let dir = TempDir::new().unwrap();
        let h = make_app(dir.path());
        std::fs::create_dir_all(h.focuses_root.join("api-refactor")).unwrap();
        std::fs::write(
            h.focuses_root.join("api-refactor/focus.md"),
            "---\nid: api-refactor\ntitle: A\ndescription:\ncreated_at: 2026-04-30T12:00:00Z\n---\n- [ ] one\n- [ ] two\n- [ ] three\n",
        )
        .unwrap();
        let resp = h
            .app
            .clone()
            .oneshot(
                Request::builder()
                    .method("DELETE")
                    .uri("/focuses/api-refactor/tasks/1")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::NO_CONTENT);
        let content =
            std::fs::read_to_string(h.focuses_root.join("api-refactor/focus.md")).unwrap();
        assert!(content.contains("- [ ] one"));
        assert!(!content.contains("- [ ] two"));
        assert!(content.contains("- [ ] three"));
    }

    #[tokio::test]
    async fn delete_task_out_of_range_returns_400() {
        let dir = TempDir::new().unwrap();
        let h = make_app(dir.path());
        write_focus(
            &h.focuses_root,
            "api-refactor",
            &focus_md("api-refactor", "x", "x", "2026-04-30T12:00:00Z"),
        );
        let resp = h
            .app
            .clone()
            .oneshot(
                Request::builder()
                    .method("DELETE")
                    .uri("/focuses/api-refactor/tasks/99")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn accept_add_task_with_edit_uses_overrides_and_marks_decision() {
        let dir = TempDir::new().unwrap();
        let h = make_app(dir.path());
        write_focus(
            &h.focuses_root,
            "f1",
            &focus_md("f1", "F1", "x", "2026-04-30T12:00:00Z"),
        );
        write_focus(
            &h.focuses_root,
            "f2",
            &focus_md("f2", "F2", "x", "2026-04-30T12:00:00Z"),
        );

        let _ = post_json(
            &h.app,
            "/proposals",
            serde_json::json!({
                "kind": "add_task",
                "target_focus_id": "f1",
                "task_text": "ship it",
                "summary": "s",
                "reasoning": "r"
            }),
        )
        .await;

        let resp = post_json(
            &h.app,
            "/proposals/p-test/accept",
            serde_json::json!({
                "target_focus_id": "f2",
                "task_text": "ship it (edited)"
            }),
        )
        .await;
        assert_eq!(resp.status(), StatusCode::OK);

        let f1 = std::fs::read_to_string(h.focuses_root.join("f1/focus.md")).unwrap();
        assert!(!f1.contains("ship it"));
        let f2 = std::fs::read_to_string(h.focuses_root.join("f2/focus.md")).unwrap();
        assert!(f2.contains("- [ ] ship it (edited)"));

        let decisions = read_decisions_for_test(&h.decisions_path);
        assert_eq!(decisions.len(), 1);
        assert!(decisions[0].edited);
        assert_eq!(decisions[0].target.as_deref(), Some("f2"));
    }

    #[tokio::test]
    async fn accept_without_body_does_not_mark_edited() {
        let dir = TempDir::new().unwrap();
        let h = make_app(dir.path());
        write_focus(
            &h.focuses_root,
            "f1",
            &focus_md("f1", "F1", "x", "2026-04-30T12:00:00Z"),
        );
        let _ = post_json(
            &h.app,
            "/proposals",
            serde_json::json!({
                "kind": "add_task",
                "target_focus_id": "f1",
                "task_text": "ship it",
                "summary": "s",
                "reasoning": "r"
            }),
        )
        .await;
        let resp = post_empty(&h.app, "/proposals/p-test/accept").await;
        assert_eq!(resp.status(), StatusCode::OK);
        let decisions = read_decisions_for_test(&h.decisions_path);
        assert!(!decisions[0].edited);
    }

    #[tokio::test]
    async fn accept_unknown_id_returns_404() {
        let dir = TempDir::new().unwrap();
        let h = make_app(dir.path());
        let resp = post_empty(&h.app, "/proposals/missing/accept").await;
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    }
}
