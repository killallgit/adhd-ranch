use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use adhd_ranch_commands::{
    AgentDebug, AgentSessions, CommandError, Commands, CreateFocusInput, CreatedFocus, Timers,
};
use adhd_ranch_domain::agents::hooks::{HookFiring, HookWiring};
use adhd_ranch_domain::session::AgentSession;
use adhd_ranch_domain::{Focus, Settings, TimerOwner, TimerPreset};

use tauri::{AppHandle, Emitter, Manager, State, Wry};

use adhd_ranch_domain::{PigRect, RectUpdater};

use crate::app::{DebugOverlayState, SettingsPathState, SettingsState};

pub struct CommandsState(pub Arc<Commands>);
pub struct AgentSessionsState(pub Arc<AgentSessions>);
pub struct AgentDebugState(pub Arc<AgentDebug>);
pub struct TimersState(pub Arc<Timers>);
pub struct PigHitState(pub Arc<dyn RectUpdater>);
pub struct DragLockState(pub Arc<AtomicBool>);

#[tauri::command]
pub fn list_agent_sessions(state: State<'_, AgentSessionsState>) -> Vec<AgentSession> {
    state.0.list()
}

#[tauri::command]
pub fn list_hook_firings(state: State<'_, AgentDebugState>) -> Vec<HookFiring> {
    state.0.recent_firings()
}

#[tauri::command]
pub fn agent_wiring(state: State<'_, AgentDebugState>) -> HookWiring {
    state.0.wiring()
}

#[tauri::command]
pub fn list_focuses(state: State<'_, CommandsState>) -> Result<Vec<Focus>, CommandError> {
    state
        .0
        .list_focuses()
        .inspect_err(|e| log::error!("list_focuses: {e}"))
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
pub fn duplicate_focus(
    focus_id: String,
    state: State<'_, CommandsState>,
) -> Result<CreatedFocus, CommandError> {
    state
        .0
        .duplicate_focus(&focus_id)
        .inspect(|f| log::info!("focus duplicated: {focus_id} -> {}", f.id))
        .inspect_err(|e| log::error!("duplicate_focus({focus_id:?}): {e}"))
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
pub fn rename_focus(
    focus_id: String,
    title: String,
    state: State<'_, CommandsState>,
) -> Result<(), CommandError> {
    state
        .0
        .rename_focus(&focus_id, &title)
        .inspect(|_| log::info!("focus renamed: {focus_id}"))
        .inspect_err(|e| log::error!("rename_focus({focus_id:?}): {e}"))
}

#[tauri::command]
pub fn update_task(
    focus_id: String,
    index: usize,
    text: String,
    state: State<'_, CommandsState>,
) -> Result<(), CommandError> {
    state
        .0
        .update_task(&focus_id, index, &text)
        .inspect(|_| log::info!("task {index} updated in {focus_id}"))
        .inspect_err(|e| log::error!("update_task({focus_id:?}, {index}): {e}"))
}

#[tauri::command]
pub fn toggle_task(
    focus_id: String,
    index: usize,
    done: bool,
    state: State<'_, CommandsState>,
) -> Result<(), CommandError> {
    state
        .0
        .toggle_task(&focus_id, index, done)
        .inspect(|_| log::info!("task {index} in {focus_id} toggled to {done}"))
        .inspect_err(|e| log::error!("toggle_task({focus_id:?}, {index}, {done}): {e}"))
}

#[tauri::command]
pub fn start_timer(
    owner: TimerOwner,
    preset: TimerPreset,
    state: State<'_, TimersState>,
) -> Result<(), CommandError> {
    state
        .0
        .start(&owner, &preset)
        .inspect(|_| log::info!("timer started on {owner:?}"))
        .inspect_err(|e| log::error!("start_timer({owner:?}): {e}"))
}

#[tauri::command]
pub fn clear_timer(owner: TimerOwner, state: State<'_, TimersState>) -> Result<(), CommandError> {
    state
        .0
        .clear(&owner)
        .inspect(|_| log::info!("timer cleared on {owner:?}"))
        .inspect_err(|e| log::error!("clear_timer({owner:?}): {e}"))
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
    crate::app::settings_workflow::workflow_for_app(app, Arc::clone(&state.0), path_state.0.clone())
        .update(settings)
        .map_err(|e| format!("{e:?}"))
}

use adhd_ranch_domain::MonitorInfo;

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
    let Ok(mut v) = state.0.lock() else { return };
    *v = enabled;
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

#[cfg(debug_assertions)]
#[tauri::command]
pub fn get_devtools_open(app: AppHandle<Wry>) -> bool {
    app.get_webview_window("overlay-0")
        .is_some_and(|win| win.is_devtools_open())
}

#[cfg(not(debug_assertions))]
#[tauri::command]
pub fn get_devtools_open() -> bool {
    false
}
