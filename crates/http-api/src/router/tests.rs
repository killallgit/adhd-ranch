use super::*;
use adhd_ranch_domain::{Decision, DecisionKind};
use adhd_ranch_storage::{JsonlDecisionLog, JsonlProposalQueue, MarkdownFocusStore};
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
    let store: Arc<dyn FocusStore> = Arc::new(MarkdownFocusStore::new(&focuses_root));
    let proposals_path = dir.join("proposals.jsonl");
    let decisions_path = dir.join("decisions.jsonl");
    let queue = Arc::new(JsonlProposalQueue::new(proposals_path.clone()));
    let decisions = Arc::new(JsonlDecisionLog::new(decisions_path.clone()));
    let app = router_with(
        store,
        queue,
        decisions,
        ServerDeps {
            clock: Some(fixed_clock("2026-04-30T12:00:00Z")),
            id_gen: Some(fixed_id("p-test")),
            settings: None,
        },
    );
    Harness {
        app,
        focuses_root,
        proposals_path,
        decisions_path,
    }
}

async fn post_json(app: &Router, uri: &str, body: serde_json::Value) -> axum::http::Response<Body> {
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
    let content = std::fs::read_to_string(h.focuses_root.join("api-refactor/focus.md")).unwrap();
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
    let content = std::fs::read_to_string(h.focuses_root.join("api-refactor/focus.md")).unwrap();
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
