mod agent_hooks;
pub mod cap_notifier;
mod claude_hook;
pub mod menu;
pub mod paths;
pub mod seed;
pub mod settings_workflow;
pub mod timer_expiry;
pub mod tray;
pub mod window_always_on_top;

use std::sync::{Arc, Mutex};
use std::time::Duration;

use crate::display::monitor::LogicalMonitor;
use crate::display::{DisplayManager, DisplayManagerState, DisplayService};
use adhd_ranch_commands::{AgentDebug, AgentSessions, CapEvaluator, Commands, HookPaths, Timers};
use adhd_ranch_domain::{DisplayConfig, OverCapMonitor, RectUpdater, Settings};
use adhd_ranch_storage::{
    watch_path, ClaudeCodeHooks, FocusStore, FocusWatcher, HookEventSink, HookHistory, HookJournal,
    LiveSessions, MarkdownFocusStore,
};
use tauri::{AppHandle, Emitter, Manager, WebviewUrl, WebviewWindowBuilder};
use time::format_description::well_known::Rfc3339;

use crate::ui_bridge;
use cap_notifier::TauriCapNotifier;

pub const FOCUSES_CHANGED_EVENT: &str = "focuses-changed";
pub const AGENT_SESSIONS_CHANGED_EVENT: &str = "agent-sessions-changed";
/// The overlay draws from settings it cannot see change any other way; without
/// this a resized pen only takes effect on the next launch.
pub const SETTINGS_CHANGED_EVENT: &str = "settings-changed";
/// Every firing, not only the ones that changed something — the debug window is
/// the one place that cares about a hook the ranch heard and ignored. Nothing emits
/// it where there is no socket to hear one.
pub const AGENT_HOOK_FIRED_EVENT: &str = "agent-hook-fired";

pub struct MonitorsState(pub Vec<LogicalMonitor>);
pub struct DisplayConfigState(pub Arc<Mutex<DisplayConfig>>);
pub struct SettingsState(pub Arc<Mutex<Settings>>);
pub struct SettingsPathState(pub std::path::PathBuf);
pub struct DebugOverlayState(pub Arc<Mutex<bool>>);
/// Held so the settings workflow can empty it when agents are switched off.
pub struct LiveSessionsState(pub Arc<LiveSessions>);

pub fn run() {
    let settings_path = paths::settings_file().expect("settings path");
    let settings = load_settings(&settings_path);

    let mut builder = tauri::Builder::default()
        .plugin(
            tauri_plugin_log::Builder::new()
                .level(log::LevelFilter::Info)
                // Windowing internals log every keystroke and redraw at TRACE,
                // which buries everything the ranch itself has to say.
                .level_for("tao", log::LevelFilter::Warn)
                .level_for("wry", log::LevelFilter::Warn)
                .level_for("tauri_runtime_wry", log::LevelFilter::Warn)
                .build(),
        )
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            ui_bridge::list_agent_sessions,
            ui_bridge::list_hook_firings,
            ui_bridge::agent_wiring,
            ui_bridge::list_focuses,
            ui_bridge::create_focus,
            ui_bridge::duplicate_focus,
            ui_bridge::delete_focus,
            ui_bridge::append_task,
            ui_bridge::delete_task,
            ui_bridge::rename_focus,
            ui_bridge::update_task,
            ui_bridge::toggle_task,
            ui_bridge::start_timer,
            ui_bridge::clear_timer,
            ui_bridge::update_pig_rects,
            ui_bridge::set_pig_drag_active,
            ui_bridge::get_settings,
            ui_bridge::update_settings,
            ui_bridge::get_monitors,
            ui_bridge::get_debug_overlay,
            ui_bridge::set_debug_overlay,
            ui_bridge::toggle_devtools,
            ui_bridge::get_devtools_open,
        ])
        .menu(menu::build);

    builder = builder.on_menu_event(menu::handle_event);

    builder = builder.setup(move |app| {
        let focuses_root = paths::focuses_root()?;
        std::fs::create_dir_all(&focuses_root)?;

        let markdown_store = Arc::new(MarkdownFocusStore::new(focuses_root.clone()));
        let store: Arc<dyn FocusStore> = markdown_store.clone();

        let settings_state = Arc::new(Mutex::new(settings.clone()));
        let settings_provider: adhd_ranch_commands::SettingsProvider = {
            let settings_state = Arc::clone(&settings_state);
            Arc::new(move || match settings_state.lock() {
                Ok(settings) => settings.clone(),
                Err(poisoned) => {
                    log::warn!("settings provider: lock poisoned; using recovered settings");
                    poisoned.into_inner().clone()
                }
            })
        };

        let timers = Arc::new(Timers::new(
            markdown_store,
            Arc::new(now_unix_secs),
            Arc::clone(&settings_provider),
            Arc::new(timer_expiry::TauriNotificationSink::new(
                app.handle().clone(),
            )),
        ));

        let commands = Arc::new(Commands::new(
            store.clone(),
            Arc::clone(&timers),
            Arc::new(now_rfc3339),
            Arc::new(|| uuid::Uuid::now_v7().to_string()),
            Arc::clone(&settings_provider),
        ));
        seed::ensure_example_focus(&commands, &focuses_root)?;

        let cap_monitor = Arc::new(OverCapMonitor::new());
        let notifier = Arc::new(TauriCapNotifier::new(app.handle().clone()));
        let evaluator = Arc::new(CapEvaluator::new(
            store.clone(),
            cap_monitor,
            notifier,
            Arc::clone(&settings_provider),
        ));

        app.manage(ui_bridge::CommandsState(commands));
        app.manage(ui_bridge::TimersState(Arc::clone(&timers)));

        // Sessions arrive by hook, pushed straight into memory — there is no file to
        // write, watch or clean up, and nothing survives the app to go stale.
        let live_sessions = Arc::new(LiveSessions::new());
        // Wrapped on every platform, not only where the socket exists, so the debug
        // window has the same shape everywhere and simply shows nothing arriving.
        let journal = Arc::new(HookJournal::new(
            Arc::clone(&live_sessions) as Arc<dyn HookEventSink>,
            Arc::new(now_rfc3339),
        ));
        let server = agent_hooks::serve(
            app.handle(),
            paths::agent_hook_socket()?,
            Arc::clone(&journal) as Arc<dyn HookEventSink>,
        )?;
        app.manage(HookServerHandle(server));
        // A startup failure here is worth knowing about but not worth refusing to
        // launch over: the ranch still runs, it just has no animals to draw.
        if let Err(e) = claude_hook::reconcile(settings.agents.enabled) {
            log::error!("{e}");
        }
        app.manage(LiveSessionsState(Arc::clone(&live_sessions)));
        app.manage(ui_bridge::AgentSessionsState(Arc::new(AgentSessions::new(
            Arc::clone(&live_sessions) as Arc<dyn adhd_ranch_storage::AgentSessionStore>,
            Arc::clone(&settings_provider),
        ))));

        let claude_paths = claude_hook::hook_paths()?;
        let hook_paths = HookPaths {
            settings_file: claude_paths.settings_file.to_string_lossy().into_owned(),
            client_bin: claude_paths.client_bin.to_string_lossy().into_owned(),
            socket_path: claude_paths.socket_path.to_string_lossy().into_owned(),
        };
        log::info!(
            "agent hooks: client={} agent settings={}",
            hook_paths.client_bin,
            hook_paths.settings_file
        );
        app.manage(ui_bridge::AgentDebugState(Arc::new(AgentDebug::new(
            Arc::new(ClaudeCodeHooks::new(claude_paths)),
            Arc::clone(&journal) as Arc<dyn HookHistory>,
            Arc::clone(&settings_provider),
            hook_paths,
        ))));

        // Enumerate connected monitors and store for tray + overlay management.
        let mut monitor_infos: Vec<LogicalMonitor> = match app.available_monitors() {
            Ok(monitors) => monitors
                .into_iter()
                .enumerate()
                .map(|(idx, m)| LogicalMonitor::from_tauri(idx, &m))
                .collect(),
            Err(e) => {
                log::error!("setup: failed to enumerate monitors: {e}");
                Vec::new()
            }
        };
        crate::display::monitor::disambiguate_names(&mut monitor_infos);

        let display_config = settings.displays.clone();
        app.manage(MonitorsState(monitor_infos.clone()));
        app.manage(DisplayConfigState(Arc::new(Mutex::new(
            display_config.clone(),
        ))));
        app.manage(SettingsState(Arc::clone(&settings_state)));
        app.manage(SettingsPathState(settings_path.clone()));
        app.manage(DebugOverlayState(Arc::new(Mutex::new(false))));

        // DisplayManager must be managed before windows are shown so invoke
        // calls from React can find PigHitState immediately.
        let display_manager = DisplayManager::new();
        let rect_updater: Arc<dyn RectUpdater> = Arc::new(display_manager.clone());
        let display_svc: Arc<dyn DisplayService> = Arc::new(display_manager.clone());
        app.manage(ui_bridge::PigHitState(rect_updater));
        app.manage(ui_bridge::DragLockState(display_manager.drag_active()));
        app.manage(DisplayManagerState(Arc::clone(&display_svc)));
        display_svc.apply(app.handle(), &monitor_infos, &display_config);

        let tray_icon = tray::setup(app.handle(), store.clone(), settings.clone())?;

        let focuses_watcher = install_change_handlers(
            &focuses_root,
            vec![
                emit_event_handler(app.handle().clone(), FOCUSES_CHANGED_EVENT),
                evaluate_caps_handler(evaluator.clone()),
                tray::rebuild_handler(
                    tray_icon.clone(),
                    app.handle().clone(),
                    store.clone(),
                    Arc::clone(&settings_provider),
                ),
            ],
        )?;
        app.manage(TrayHandle(tray_icon));
        app.manage(WatcherHandles {
            _focuses: focuses_watcher,
        });

        timer_expiry::spawn(timers);

        Ok(())
    });

    builder
        .build(tauri::generate_context!())
        .expect("tauri build error")
        .run(|app, event| {
            #[cfg(target_os = "macos")]
            if let tauri::RunEvent::Reopen { .. } = event {
                if let Some(window) = app.get_webview_window("overlay-0") {
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

fn now_unix_secs() -> i64 {
    time::OffsetDateTime::now_utc().unix_timestamp()
}

pub fn open_settings_window<R: tauri::Runtime>(app: &AppHandle<R>) {
    if let Some(win) = app.get_webview_window("settings") {
        if win.is_visible().unwrap_or(false) {
            let _ = win.set_focus();
            return;
        }
        // Label still registered but window is closed — destroy to free it
        let _ = win.destroy();
    }
    match WebviewWindowBuilder::new(app, "settings", WebviewUrl::App("settings.html".into()))
        .title("Preferences")
        .inner_size(380.0, 300.0)
        .min_inner_size(380.0, 100.0)
        .decorations(true)
        .resizable(false)
        .build()
    {
        Ok(win) => {
            let _ = win.show();
        }
        Err(e) => log::error!("open_settings_window: {e}"),
    }
}

/// A separate window rather than a panel on the overlay: the overlay is
/// click-through and has no room, and this has to be readable while the ranch is
/// doing the thing being debugged.
pub fn open_agent_debug_window<R: tauri::Runtime>(app: &AppHandle<R>) {
    if let Some(win) = app.get_webview_window("agent-debug") {
        if win.is_visible().unwrap_or(false) {
            let _ = win.set_focus();
            return;
        }
        let _ = win.destroy();
    }
    match WebviewWindowBuilder::new(
        app,
        "agent-debug",
        WebviewUrl::App("agent-debug.html".into()),
    )
    .title("Agent Hooks")
    .inner_size(520.0, 620.0)
    .min_inner_size(420.0, 320.0)
    .decorations(true)
    .always_on_top(true)
    .build()
    {
        Ok(win) => {
            let _ = win.show();
        }
        Err(e) => log::error!("open_agent_debug_window: {e}"),
    }
}

fn load_settings(path: &std::path::Path) -> Settings {
    match std::fs::read_to_string(path) {
        Ok(raw) => Settings::parse_yaml(&raw),
        Err(_) => Settings::default(),
    }
}

#[allow(dead_code)]
struct TrayHandle(tauri::tray::TrayIcon<tauri::Wry>);

#[allow(dead_code)]
struct WatcherHandles {
    _focuses: FocusWatcher,
}

/// Held only so the listener outlives setup; dropping it shuts the listener down.
#[allow(dead_code)]
struct HookServerHandle(adhd_ranch_storage::HookServer);

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
