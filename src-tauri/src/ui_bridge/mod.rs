use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use adhd_ranch_commands::{
    CommandError, Commands, CreateFocusInput, CreatedFocus, CreatedProposal, DecisionOutcome,
    ProposalEdit,
};
use adhd_ranch_domain::{Caps, Focus, Proposal, Settings};
use adhd_ranch_storage::write_settings;

use tauri::{AppHandle, Emitter, Manager, State, Wry};

use adhd_ranch_domain::{PigRect, RectUpdater};

use crate::api::Health;
use crate::app::{DebugOverlayState, SettingsPathState, SettingsState};

pub struct CommandsState(pub Arc<Commands>);
pub struct PigHitState(pub Arc<dyn RectUpdater>);
pub struct DragLockState(pub Arc<AtomicBool>);

#[tauri::command]
pub fn health() -> Health {
    Health { ok: true }
}

#[tauri::command]
pub fn list_focuses(state: State<'_, CommandsState>) -> Result<Vec<Focus>, CommandError> {
    state
        .0
        .list_focuses()
        .inspect_err(|e| log::error!("list_focuses: {e}"))
}

#[tauri::command]
pub fn list_proposals(state: State<'_, CommandsState>) -> Result<Vec<Proposal>, CommandError> {
    state
        .0
        .list_proposals()
        .inspect_err(|e| log::error!("list_proposals: {e}"))
}

#[tauri::command]
pub fn create_focus(
    title: String,
    description: Option<String>,
    timer_preset: Option<adhd_ranch_domain::TimerPreset>,
    state: State<'_, CommandsState>,
) -> Result<CreatedFocus, CommandError> {
    state
        .0
        .create_focus(CreateFocusInput {
            title: title.clone(),
            description: description.unwrap_or_default(),
            timer_preset,
        })
        .inspect(|f| log::info!("focus created: {}", f.id))
        .inspect_err(|e| log::error!("create_focus({title:?}): {e}"))
}

#[tauri::command]
pub fn delete_focus(focus_id: String, state: State<'_, CommandsState>) -> Result<(), CommandError> {
    state
        .0
        .delete_focus(&focus_id)
        .inspect(|_| log::info!("focus deleted: {focus_id}"))
        .inspect_err(|e| log::error!("delete_focus({focus_id:?}): {e}"))
}

#[tauri::command]
pub fn append_task(
    focus_id: String,
    text: String,
    state: State<'_, CommandsState>,
) -> Result<(), CommandError> {
    state
        .0
        .append_task(&focus_id, &text)
        .inspect(|_| log::info!("task appended to {focus_id}"))
        .inspect_err(|e| log::error!("append_task({focus_id:?}): {e}"))
}

#[tauri::command]
pub fn delete_task(
    focus_id: String,
    index: usize,
    state: State<'_, CommandsState>,
) -> Result<(), CommandError> {
    state
        .0
        .delete_task(&focus_id, index)
        .inspect(|_| log::info!("task {index} deleted from {focus_id}"))
        .inspect_err(|e| log::error!("delete_task({focus_id:?}, {index}): {e}"))
}

#[tauri::command]
pub fn get_caps(state: State<'_, CommandsState>) -> Caps {
    state.0.caps()
}

#[tauri::command]
pub fn accept_proposal(
    id: String,
    edit: Option<ProposalEdit>,
    state: State<'_, CommandsState>,
) -> Result<DecisionOutcome, CommandError> {
    state
        .0
        .accept_proposal(&id, edit.unwrap_or_default())
        .inspect(|o| log::info!("proposal {id} accepted → {:?}", o.target))
        .inspect_err(|e| log::error!("accept_proposal({id:?}): {e}"))
}

#[tauri::command]
pub fn reject_proposal(
    id: String,
    state: State<'_, CommandsState>,
) -> Result<DecisionOutcome, CommandError> {
    state
        .0
        .reject_proposal(&id)
        .inspect(|_| log::info!("proposal {id} rejected"))
        .inspect_err(|e| log::error!("reject_proposal({id:?}): {e}"))
}

#[tauri::command]
pub fn create_proposal(
    input: adhd_ranch_commands::CreateProposalInput,
    state: State<'_, CommandsState>,
) -> Result<CreatedProposal, CommandError> {
    state
        .0
        .create_proposal(input)
        .inspect(|p| log::info!("proposal created: {}", p.id))
        .inspect_err(|e| log::error!("create_proposal: {e}"))
}

#[tauri::command]
pub fn update_pig_rects(
    window: tauri::WebviewWindow,
    rects: Vec<PigRect>,
    state: State<'_, PigHitState>,
) {
    state.0.update_rects(window.label(), rects);
}

#[tauri::command]
pub fn set_pig_drag_active(active: bool, state: State<'_, DragLockState>) {
    state.0.store(active, Ordering::Relaxed);
}

#[tauri::command]
pub fn get_settings(state: State<'_, SettingsState>) -> Settings {
    state.0.lock().map(|s| s.clone()).unwrap_or_default()
}

#[tauri::command]
pub fn update_settings(
    settings: Settings,
    app: AppHandle<Wry>,
    state: State<'_, SettingsState>,
    path_state: State<'_, SettingsPathState>,
) -> Result<(), String> {
    let old_displays = {
        let Ok(mut s) = state.0.lock() else {
            return Err("settings lock poisoned".to_string());
        };
        let old = s.displays.clone();
        *s = settings.clone();
        old
    };

    write_settings(&path_state.0, &settings).map_err(|e| format!("persist settings: {e}"))?;

    let monitors_count = app
        .try_state::<crate::app::MonitorsState>()
        .map(|s| s.0.len())
        .unwrap_or(1);
    for i in 0..monitors_count {
        if let Some(w) = app.get_webview_window(&format!("overlay-{i}")) {
            crate::app::window_always_on_top::apply(&w, settings.widget.always_on_top);
        }
    }

    if settings.displays != old_displays {
        if let Some(display_state) = app.try_state::<crate::app::DisplayConfigState>() {
            if let Ok(mut config) = display_state.0.lock() {
                *config = settings.displays.clone();
            }
        }
        if let Some(overlay_state) = app.try_state::<crate::display::DisplayManagerState>() {
            if let Some(monitors_state) = app.try_state::<crate::app::MonitorsState>() {
                let display_mgr = Arc::clone(&overlay_state.0);
                let monitors = monitors_state.0.clone();
                let config = settings.displays.clone();
                let app_clone = app.clone();
                if let Err(e) = app.run_on_main_thread(move || {
                    display_mgr.apply(&app_clone, &monitors, &config);
                }) {
                    log::error!("update_settings: run_on_main_thread failed: {e}");
                }
            }
        }
    }

    crate::app::tray::rebuild_tray_menu(&app);

    Ok(())
}

#[derive(serde::Serialize)]
pub struct MonitorInfo {
    pub idx: usize,
    pub label: String,
}

#[tauri::command]
pub fn get_monitors(app: AppHandle<Wry>) -> Vec<MonitorInfo> {
    app.try_state::<crate::app::MonitorsState>()
        .map(|s| {
            s.0.iter()
                .enumerate()
                .map(|(i, m)| MonitorInfo {
                    idx: i,
                    label: m.label.clone(),
                })
                .collect()
        })
        .unwrap_or_default()
}

#[tauri::command]
pub fn get_debug_overlay(state: State<'_, DebugOverlayState>) -> bool {
    state.0.lock().map(|v| *v).unwrap_or(false)
}

#[tauri::command]
pub fn set_debug_overlay(enabled: bool, app: AppHandle<Wry>, state: State<'_, DebugOverlayState>) {
    if let Ok(mut v) = state.0.lock() {
        *v = enabled;
    }
    let _ = app.emit("debug-overlay-toggle", enabled);
}

#[tauri::command]
pub fn toggle_devtools(app: AppHandle<Wry>) {
    if let Some(win) = app.get_webview_window("overlay-0") {
        #[cfg(debug_assertions)]
        if win.is_devtools_open() {
            win.close_devtools();
        } else {
            win.open_devtools();
        }
        let _ = win;
    }
}

#[tauri::command]
pub fn get_devtools_open(app: AppHandle<Wry>) -> bool {
    #[cfg(debug_assertions)]
    if let Some(win) = app.get_webview_window("overlay-0") {
        return win.is_devtools_open();
    }
    false
}
