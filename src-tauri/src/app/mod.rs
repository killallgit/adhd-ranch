pub mod cap_notifier;
pub mod menu;
pub mod paths;
pub mod pig_hittest;
pub mod tray;
pub mod window_always_on_top;

use std::sync::Arc;
use std::time::Duration;

use adhd_ranch_commands::{CapEvaluator, Commands};
use adhd_ranch_domain::{OverCapMonitor, Settings};
use adhd_ranch_http_api::{serve, ServerHandle};
use adhd_ranch_storage::{
    watch_path, DecisionLog, FocusStore, FocusWatcher, JsonlDecisionLog, JsonlProposalQueue,
    MarkdownFocusStore, ProposalQueue,
};
use tauri::{AppHandle, Emitter, Manager};
use time::format_description::well_known::Rfc3339;

use crate::ui_bridge;
use cap_notifier::TauriCapNotifier;

pub const FOCUSES_CHANGED_EVENT: &str = "focuses-changed";
pub const PROPOSALS_CHANGED_EVENT: &str = "proposals-changed";

pub fn run() {
    let settings_path = paths::settings_file().expect("settings path");
    let settings = load_settings(&settings_path);

    let event_settings_path = settings_path.clone();

    let mut builder = tauri::Builder::default()
        .plugin(tauri_plugin_log::Builder::new().build())
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
            ui_bridge::update_pig_rects,
        ])
        .menu(move |handle| menu::build(handle, settings.widget.always_on_top));

    builder = builder.on_menu_event(move |app, event| {
        menu::handle_event(app, event, &event_settings_path);
    });

    builder = builder.setup(move |app| {
        let focuses_root = paths::focuses_root()?;
        std::fs::create_dir_all(&focuses_root)?;
        let proposals_path = paths::proposals_file()?;
        let decisions_path = paths::decisions_file()?;
        if let Some(parent) = proposals_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let store: Arc<dyn FocusStore> = Arc::new(MarkdownFocusStore::new(focuses_root.clone()));
        let queue: Arc<dyn ProposalQueue> =
            Arc::new(JsonlProposalQueue::new(proposals_path.clone()));
        let decision_log: Arc<dyn DecisionLog> =
            Arc::new(JsonlDecisionLog::new(decisions_path.clone()));

        let commands = Arc::new(Commands::new(
            store.clone(),
            queue.clone(),
            decision_log.clone(),
            Arc::new(now_rfc3339),
            Arc::new(|| uuid::Uuid::now_v7().to_string()),
            settings,
        ));

        let monitor = Arc::new(OverCapMonitor::new());
        let notifier = Arc::new(TauriCapNotifier::new(app.handle().clone()));
        let evaluator = Arc::new(CapEvaluator::new(
            store.clone(),
            monitor,
            notifier,
            settings,
        ));

        app.manage(ui_bridge::CommandsState(commands));

        let tray_icon = tray::setup(app.handle(), store.clone(), settings)?;

        let focuses_watcher = install_change_handlers(
            &focuses_root,
            vec![
                emit_event_handler(app.handle().clone(), FOCUSES_CHANGED_EVENT),
                evaluate_caps_handler(evaluator.clone()),
                tray::rebuild_handler(
                    tray_icon.clone(),
                    app.handle().clone(),
                    store.clone(),
                    settings,
                ),
            ],
        )?;

        app.manage(TrayHandle(tray_icon));
        let proposals_watcher = install_change_handlers(
            proposals_path.parent().expect("proposals path has parent"),
            vec![emit_event_handler(
                app.handle().clone(),
                PROPOSALS_CHANGED_EVENT,
            )],
        )?;
        app.manage(WatcherHandles {
            _focuses: focuses_watcher,
            _proposals: proposals_watcher,
        });

        let server = install_http_server(store, queue, decision_log)?;
        app.manage(server);

        let hittester = pig_hittest::PigHitTester::new();
        app.manage(ui_bridge::PigHitState(hittester.clone()));

        if let Some(window) = app.get_webview_window("main") {
            // Size the overlay to cover the primary monitor.
            if let Ok(Some(monitor)) = window.current_monitor() {
                let size = monitor.size();
                let pos = monitor.position();
                let _ = window.set_size(tauri::PhysicalSize::new(size.width, size.height));
                let _ = window.set_position(tauri::PhysicalPosition::new(pos.x, pos.y));
            }

            window_always_on_top::apply(&window, true);
            let _ = window.show();

            // Polling thread: toggle click-through based on pig hit-test.
            let app_handle = app.handle().clone();
            let window_clone = window.clone();
            std::thread::spawn(move || {
                let mut last_over = false;
                loop {
                    if let Ok(cursor) = app_handle.cursor_position() {
                        let over = hittester.is_hit(cursor.x, cursor.y);
                        if over != last_over {
                            let _ = window_clone.set_ignore_cursor_events(!over);
                            last_over = over;
                        }
                    }
                    std::thread::sleep(std::time::Duration::from_millis(16));
                }
            });
        }

        Ok(())
    });

    builder
        .build(tauri::generate_context!())
        .expect("tauri build error")
        .run(|app, event| {
            #[cfg(target_os = "macos")]
            if let tauri::RunEvent::Reopen { .. } = event {
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.show();
                    let _ = window.set_focus();
                }
            }
            let _ = (app, event);
        });
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

#[allow(dead_code)]
struct WatcherHandles {
    _focuses: FocusWatcher,
    _proposals: FocusWatcher,
}

#[allow(dead_code)]
struct TrayHandle<R: tauri::Runtime>(tauri::tray::TrayIcon<R>);

type ChangeHandler = Box<dyn Fn() + Send + 'static>;

const WATCH_DEBOUNCE: Duration = Duration::from_millis(200);

fn install_change_handlers(
    path: &std::path::Path,
    handlers: Vec<ChangeHandler>,
) -> Result<FocusWatcher, Box<dyn std::error::Error>> {
    let watcher = watch_path(path, WATCH_DEBOUNCE, move || {
        for handler in &handlers {
            handler();
        }
    })?;
    Ok(watcher)
}

fn emit_event_handler(handle: AppHandle, event: &'static str) -> ChangeHandler {
    Box::new(move || {
        let _ = handle.emit(event, ());
    })
}

fn evaluate_caps_handler(evaluator: Arc<CapEvaluator>) -> ChangeHandler {
    Box::new(move || {
        let _ = evaluator.evaluate();
    })
}

fn install_http_server(
    store: Arc<dyn FocusStore>,
    queue: Arc<dyn ProposalQueue>,
    decisions: Arc<dyn DecisionLog>,
) -> Result<ServerHandle, Box<dyn std::error::Error>> {
    let port_file = paths::port_file()?;
    let runtime = tauri::async_runtime::handle();
    let handle =
        runtime.block_on(async move { serve(store, queue, decisions, Some(port_file)).await })?;
    Ok(handle)
}
