pub mod paths;
pub mod tray;

use std::sync::Arc;
use std::time::Duration;

use adhd_ranch_http_api::{serve, ServerHandle};
use adhd_ranch_storage::{watch_focuses, FocusRepository, FocusWatcher, MarkdownFocusRepository};
use tauri::{AppHandle, Emitter, Manager};

use crate::ui_bridge;

pub const FOCUSES_CHANGED_EVENT: &str = "focuses-changed";

pub fn run() {
    let mut builder = tauri::Builder::default()
        .plugin(tauri_plugin_positioner::init())
        .invoke_handler(tauri::generate_handler![
            ui_bridge::health,
            ui_bridge::list_focuses
        ]);

    builder = builder.setup(|app| {
        #[cfg(target_os = "macos")]
        app.set_activation_policy(tauri::ActivationPolicy::Accessory);

        let focuses_root = paths::focuses_root()?;
        std::fs::create_dir_all(&focuses_root)?;

        let repo: Arc<dyn FocusRepository> =
            Arc::new(MarkdownFocusRepository::new(focuses_root.clone()));
        app.manage(ui_bridge::FocusRepoState(repo.clone()));

        let watcher = install_watcher(app.handle().clone(), &focuses_root)?;
        app.manage(WatcherHandle(watcher));

        let server = install_http_server(repo)?;
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

struct WatcherHandle(#[allow(dead_code)] FocusWatcher);

fn install_watcher(
    handle: AppHandle,
    root: &std::path::Path,
) -> Result<FocusWatcher, Box<dyn std::error::Error>> {
    let watcher = watch_focuses(root, Duration::from_millis(200), move || {
        let _ = handle.emit(FOCUSES_CHANGED_EVENT, ());
    })?;
    Ok(watcher)
}

fn install_http_server(
    repo: Arc<dyn FocusRepository>,
) -> Result<ServerHandle, Box<dyn std::error::Error>> {
    let port_file = paths::port_file()?;
    let runtime = tauri::async_runtime::handle();
    let handle = runtime.block_on(async move { serve(repo, Some(port_file)).await })?;
    Ok(handle)
}
