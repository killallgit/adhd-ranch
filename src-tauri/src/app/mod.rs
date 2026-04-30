pub mod paths;
pub mod tray;

use std::sync::Arc;
use std::time::Duration;

use adhd_ranch_commands::{Commands, ProposalDispatcher};
use adhd_ranch_domain::{cap_state, CapTransition, Caps, OverCapMonitor, Settings};
use adhd_ranch_http_api::{serve, ServerHandle};
use adhd_ranch_storage::{
    watch_path, DecisionLog, FocusStore, FocusWatcher, JsonlDecisionLog, JsonlProposalQueue,
    MarkdownFocusStore, ProposalQueue,
};
use tauri::{AppHandle, Emitter, Manager};
use tauri_plugin_notification::NotificationExt;
use time::format_description::well_known::Rfc3339;

use crate::ui_bridge;

pub const FOCUSES_CHANGED_EVENT: &str = "focuses-changed";
pub const PROPOSALS_CHANGED_EVENT: &str = "proposals-changed";

pub fn run() {
    let mut builder = tauri::Builder::default()
        .plugin(tauri_plugin_positioner::init())
        .plugin(tauri_plugin_notification::init())
        .invoke_handler(tauri::generate_handler![
            ui_bridge::health,
            ui_bridge::list_focuses,
            ui_bridge::list_proposals,
            ui_bridge::accept_proposal,
            ui_bridge::reject_proposal,
            ui_bridge::create_focus,
            ui_bridge::create_proposal,
            ui_bridge::delete_focus,
            ui_bridge::append_task,
            ui_bridge::delete_task,
            ui_bridge::get_caps,
        ]);

    builder = builder.setup(|app| {
        #[cfg(target_os = "macos")]
        app.set_activation_policy(tauri::ActivationPolicy::Accessory);

        let focuses_root = paths::focuses_root()?;
        std::fs::create_dir_all(&focuses_root)?;
        let proposals_path = paths::proposals_file()?;
        let decisions_path = paths::decisions_file()?;
        let settings_path = paths::settings_file()?;
        if let Some(parent) = proposals_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let settings = load_settings(&settings_path);

        let store: Arc<dyn FocusStore> = Arc::new(MarkdownFocusStore::new(focuses_root.clone()));
        let queue: Arc<dyn ProposalQueue> =
            Arc::new(JsonlProposalQueue::new(proposals_path.clone()));
        let decision_log: Arc<dyn DecisionLog> =
            Arc::new(JsonlDecisionLog::new(decisions_path.clone()));

        let dispatcher = Arc::new(ProposalDispatcher::from_store(
            store.clone(),
            Arc::new(now_rfc3339),
            Arc::new(|| uuid::Uuid::now_v7().to_string()),
        ));

        let commands = Arc::new(Commands::new(
            store.clone(),
            queue.clone(),
            decision_log.clone(),
            dispatcher.clone(),
            Arc::new(now_rfc3339),
            Arc::new(|| uuid::Uuid::now_v7().to_string()),
            settings,
        ));

        let monitor = Arc::new(OverCapMonitor::new());

        app.manage(ui_bridge::CommandsState(commands));
        app.manage(ui_bridge::MonitorState(monitor.clone()));

        let cap_handle = app.handle().clone();
        let cap_store = store.clone();
        let cap_monitor = monitor.clone();
        let focuses_watcher = watch_path(&focuses_root, Duration::from_millis(200), move || {
            let _ = cap_handle.emit(FOCUSES_CHANGED_EVENT, ());
            evaluate_caps(
                &cap_handle,
                cap_store.as_ref(),
                cap_monitor.as_ref(),
                settings,
            );
        })?;
        let proposals_watcher = install_watcher(
            app.handle().clone(),
            proposals_path.parent().expect("proposals path has parent"),
            PROPOSALS_CHANGED_EVENT,
        )?;
        app.manage(WatcherHandles {
            _focuses: focuses_watcher,
            _proposals: proposals_watcher,
        });

        let server = install_http_server(store, queue, decision_log, dispatcher)?;
        app.manage(server);

        tray::install(app.handle())?;
        Ok(())
    });

    builder = builder.on_window_event(|window, event| {
        if let tauri::WindowEvent::Focused(false) = event {
            if window.label() == "main" {
                let _ = window.hide();
            }
        }
    });

    builder
        .run(tauri::generate_context!())
        .expect("tauri runtime error");
}

fn now_rfc3339() -> String {
    time::OffsetDateTime::now_utc()
        .format(&Rfc3339)
        .unwrap_or_default()
}

fn load_settings(path: &std::path::Path) -> Settings {
    match std::fs::read_to_string(path) {
        Ok(raw) => Settings::parse_yaml(&raw),
        Err(_) => Settings::default(),
    }
}

fn evaluate_caps(
    handle: &AppHandle,
    store: &dyn FocusStore,
    monitor: &OverCapMonitor,
    settings: Settings,
) {
    let focuses = match store.list() {
        Ok(f) => f,
        Err(_) => return,
    };
    let state = cap_state(&focuses, settings.caps);
    let transition = monitor.evaluate(&state);
    if !settings.alerts.system_notifications {
        return;
    }
    notify_transitions(handle, &transition, settings.caps);
}

fn notify_transitions(handle: &AppHandle, transition: &CapTransition, caps: Caps) {
    if transition.focuses_to_over {
        let _ = handle
            .notification()
            .builder()
            .title("Too many focuses")
            .body(format!(
                "You're over the limit of {} focuses — trim one.",
                caps.max_focuses
            ))
            .show();
    }
    for id in &transition.task_to_over_focus_ids {
        let _ = handle
            .notification()
            .builder()
            .title("Focus has too many tasks")
            .body(format!(
                "Focus {id} has more than {} tasks.",
                caps.max_tasks_per_focus
            ))
            .show();
    }
}

#[allow(dead_code)]
struct WatcherHandles {
    _focuses: FocusWatcher,
    _proposals: FocusWatcher,
}

fn install_watcher(
    handle: AppHandle,
    path: &std::path::Path,
    event: &'static str,
) -> Result<FocusWatcher, Box<dyn std::error::Error>> {
    let watcher = watch_path(path, Duration::from_millis(200), move || {
        let _ = handle.emit(event, ());
    })?;
    Ok(watcher)
}

fn install_http_server(
    store: Arc<dyn FocusStore>,
    queue: Arc<dyn ProposalQueue>,
    decisions: Arc<dyn DecisionLog>,
    dispatcher: Arc<ProposalDispatcher>,
) -> Result<ServerHandle, Box<dyn std::error::Error>> {
    let port_file = paths::port_file()?;
    let runtime = tauri::async_runtime::handle();
    let handle = runtime.block_on(async move {
        serve(store, queue, decisions, dispatcher, Some(port_file)).await
    })?;
    Ok(handle)
}
