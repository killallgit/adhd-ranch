pub mod paths;
pub mod tray;

use std::sync::Arc;
use std::time::Duration;

use adhd_ranch_http_api::{serve, ServerHandle};
use adhd_ranch_storage::{
    watch_path, FocusRepository, FocusWatcher, JsonlProposalQueue, MarkdownFocusRepository,
    ProposalQueue,
};
use tauri::{AppHandle, Emitter, Manager};

use crate::ui_bridge;

pub const FOCUSES_CHANGED_EVENT: &str = "focuses-changed";
pub const PROPOSALS_CHANGED_EVENT: &str = "proposals-changed";

pub fn run() {
    let mut builder = tauri::Builder::default()
        .plugin(tauri_plugin_positioner::init())
        .invoke_handler(tauri::generate_handler![
            ui_bridge::health,
            ui_bridge::list_focuses,
            ui_bridge::list_proposals,
        ]);

    builder = builder.setup(|app| {
        #[cfg(target_os = "macos")]
        app.set_activation_policy(tauri::ActivationPolicy::Accessory);

        let focuses_root = paths::focuses_root()?;
        std::fs::create_dir_all(&focuses_root)?;
        let proposals_path = paths::proposals_file()?;
        if let Some(parent) = proposals_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let repo: Arc<dyn FocusRepository> =
            Arc::new(MarkdownFocusRepository::new(focuses_root.clone()));
        let queue: Arc<dyn ProposalQueue> =
            Arc::new(JsonlProposalQueue::new(proposals_path.clone()));
        app.manage(ui_bridge::FocusRepoState(repo.clone()));
        app.manage(ui_bridge::ProposalQueueState(queue.clone()));

        let focuses_watcher =
            install_watcher(app.handle().clone(), &focuses_root, FOCUSES_CHANGED_EVENT)?;
        let proposals_watcher = install_watcher(
            app.handle().clone(),
            proposals_path.parent().expect("proposals path has parent"),
            PROPOSALS_CHANGED_EVENT,
        )?;
        app.manage(WatcherHandles {
            _focuses: focuses_watcher,
            _proposals: proposals_watcher,
        });

        let server = install_http_server(repo, queue)?;
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
    repo: Arc<dyn FocusRepository>,
    queue: Arc<dyn ProposalQueue>,
) -> Result<ServerHandle, Box<dyn std::error::Error>> {
    let port_file = paths::port_file()?;
    let runtime = tauri::async_runtime::handle();
    let handle = runtime.block_on(async move { serve(repo, queue, Some(port_file)).await })?;
    Ok(handle)
}
