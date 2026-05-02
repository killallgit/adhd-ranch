use std::path::PathBuf;
use std::sync::Arc;

use adhd_ranch_domain::{cap_state, DisplayConfig, Focus, Settings};
use adhd_ranch_storage::{write_settings, FocusStore};
use tauri::image::Image;
use tauri::menu::{IsMenuItem, Menu, MenuItemBuilder, PredefinedMenuItem, SubmenuBuilder};
use tauri::tray::{TrayIcon, TrayIconBuilder};
use tauri::{AppHandle, Manager, Runtime};

use super::{DisplayConfigState, MonitorsState};

const QUIT_ID: &str = "tray-quit";
const NO_FOCUSES_ID: &str = "tray-no-focuses";
const NEW_FOCUS_ID: &str = "tray-new-focus";
const DELETE_PREFIX: &str = "tray-delete-";
const DISPLAY_PREFIX: &str = "tray-display-";

pub fn setup<R: Runtime>(
    app: &AppHandle<R>,
    store: Arc<dyn FocusStore>,
    settings: Settings,
    settings_path: PathBuf,
) -> tauri::Result<TrayIcon<R>> {
    let focuses = match store.list() {
        Ok(f) => f,
        Err(e) => {
            log::error!("tray setup: failed to read focuses: {e}");
            Vec::new()
        }
    };
    let menu = build_menu(app, &focuses)?;
    let over_cap = cap_state(&focuses, settings.caps).any_over();

    let settings_path_for_handler = settings_path.clone();
    let mut builder = TrayIconBuilder::with_id("main-tray")
        .menu(&menu)
        .show_menu_on_left_click(true)
        .on_menu_event(move |app, event| {
            let id = event.id().0.as_str();
            if id == QUIT_ID {
                app.exit(0);
            } else if id == NEW_FOCUS_ID {
                if let Some(win) = app.get_webview_window("new-focus") {
                    let _ = win.show();
                    let _ = win.set_focus();
                }
            } else if let Some(focus_id) = id.strip_prefix(DELETE_PREFIX) {
                let focus_id = focus_id.to_string();
                let app_handle = app.clone();
                std::thread::spawn(move || handle_delete(app_handle, focus_id));
            } else if let Some(idx_str) = id.strip_prefix(DISPLAY_PREFIX) {
                if let Ok(idx) = idx_str.parse::<usize>() {
                    let app_handle = app.clone();
                    let path = settings_path_for_handler.clone();
                    std::thread::spawn(move || handle_display_toggle(app_handle, idx, path));
                }
            }
        });

    if let Some(icon) = app.default_window_icon() {
        builder = builder.icon(icon.clone());
    }

    let tray = builder.build(app)?;

    if over_cap {
        let _ = tray.set_icon(Some(red_icon()));
    } else if app.default_window_icon().is_none() {
        let _ = tray.set_icon(None);
    }

    Ok(tray)
}

pub fn rebuild_handler<R: Runtime>(
    tray: TrayIcon<R>,
    handle: AppHandle<R>,
    store: Arc<dyn FocusStore>,
    settings: Settings,
) -> Box<dyn Fn() + Send + 'static> {
    Box::new(move || {
        let focuses = match store.list() {
            Ok(f) => f,
            Err(e) => {
                log::error!("tray rebuild: failed to read focuses: {e}");
                return;
            }
        };
        if let Ok(menu) = build_menu(&handle, &focuses) {
            let _ = tray.set_menu(Some(menu));
        }
        let over_cap = cap_state(&focuses, settings.caps).any_over();
        if over_cap {
            let _ = tray.set_icon(Some(red_icon()));
        } else if let Some(icon) = handle.default_window_icon() {
            let _ = tray.set_icon(Some(icon.clone()));
        } else {
            let _ = tray.set_icon(None);
        }
    })
}

fn build_menu<R: Runtime>(handle: &AppHandle<R>, focuses: &[Focus]) -> tauri::Result<Menu<R>> {
    let mut items: Vec<Box<dyn IsMenuItem<R>>> = Vec::new();

    // Displays submenu — reads current config from managed state.
    if let Some(monitors_state) = handle.try_state::<MonitorsState>() {
        let enabled_indices = handle
            .try_state::<DisplayConfigState>()
            .and_then(|s| s.0.lock().ok().map(|c| c.enabled_indices.clone()))
            .unwrap_or_else(|| vec![0]);

        if !monitors_state.0.is_empty() {
            let mut sub = SubmenuBuilder::new(handle, "Displays");
            for (idx, monitor) in monitors_state.0.iter().enumerate() {
                let check = if enabled_indices.contains(&idx) {
                    "✓ "
                } else {
                    "  "
                };
                let name = monitor.name.as_deref().unwrap_or("Unknown Display");
                let item = MenuItemBuilder::with_id(
                    format!("{DISPLAY_PREFIX}{idx}"),
                    format!("{check}{name}"),
                )
                .build(handle)?;
                sub = sub.item(&item);
            }
            let displays_submenu = sub.build()?;
            items.push(Box::new(displays_submenu));
            items.push(Box::new(PredefinedMenuItem::separator(handle)?));
        }
    }

    let new_focus = MenuItemBuilder::with_id(NEW_FOCUS_ID, "+ New Focus").build(handle)?;
    items.push(Box::new(new_focus));
    items.push(Box::new(PredefinedMenuItem::separator(handle)?));

    if focuses.is_empty() {
        let item = MenuItemBuilder::with_id(NO_FOCUSES_ID, "No focuses yet")
            .enabled(false)
            .build(handle)?;
        items.push(Box::new(item));
    } else {
        for focus in focuses.iter() {
            let delete_item = MenuItemBuilder::with_id(
                format!("{DELETE_PREFIX}{}", focus.id.0),
                format!("Delete \"{}\"…", focus.title),
            )
            .build(handle)?;
            let submenu = SubmenuBuilder::new(handle, &focus.title)
                .item(&delete_item)
                .build()?;
            items.push(Box::new(submenu));
        }
    }

    let sep = PredefinedMenuItem::separator(handle)?;
    let quit = MenuItemBuilder::with_id(QUIT_ID, "Quit").build(handle)?;
    items.push(Box::new(sep));
    items.push(Box::new(quit));

    let item_refs: Vec<&dyn IsMenuItem<R>> = items.iter().map(|b| b.as_ref()).collect();
    Menu::with_items(handle, &item_refs)
}

fn handle_delete<R: Runtime>(app: AppHandle<R>, focus_id: String) {
    use tauri_plugin_dialog::{DialogExt, MessageDialogButtons};

    let title = {
        app.try_state::<crate::ui_bridge::CommandsState>()
            .and_then(|state| state.0.list_focuses().ok())
            .and_then(|focuses| focuses.into_iter().find(|f| f.id.0 == focus_id))
            .map(|f| f.title)
            .unwrap_or_else(|| focus_id.clone())
    };

    let confirmed = app
        .dialog()
        .message(format!("\"{}\" will be permanently removed.", title))
        .title("Delete Focus?")
        .buttons(MessageDialogButtons::OkCancelCustom(
            "Delete".to_string(),
            "Cancel".to_string(),
        ))
        .blocking_show();

    if confirmed {
        if let Some(state) = app.try_state::<crate::ui_bridge::CommandsState>() {
            if let Err(e) = state.0.delete_focus(&focus_id) {
                log::error!("tray delete_focus({focus_id:?}): {e}");
            }
        }
    }
}

fn handle_display_toggle<R: Runtime>(app: AppHandle<R>, idx: usize, settings_path: PathBuf) {
    let Some(display_state) = app.try_state::<DisplayConfigState>() else {
        return;
    };
    let Some(monitors_state) = app.try_state::<MonitorsState>() else {
        return;
    };
    let Some(hit_state) = app.try_state::<crate::ui_bridge::PigHitState>() else {
        return;
    };

    let new_config = {
        let Ok(mut config) = display_state.0.lock() else {
            return;
        };
        if config.enabled_indices.contains(&idx) {
            config.enabled_indices.retain(|&i| i != idx);
        } else {
            config.enabled_indices.push(idx);
            config.enabled_indices.sort_unstable();
        }
        config.clone()
    };

    persist_display_config(&settings_path, &new_config);

    // Window creation/show/hide must happen on the main thread on macOS.
    let overlay_mgr = hit_state.0.clone();
    let monitors = monitors_state.0.clone();
    let config_for_main = new_config.clone();
    let app_for_main = app.clone();
    if let Err(e) = app.run_on_main_thread(move || {
        overlay_mgr.apply(&app_for_main, &monitors, &config_for_main);
    }) {
        log::error!("tray: run_on_main_thread failed: {e}");
    }

    rebuild_tray_menu(&app);
}

fn persist_display_config(settings_path: &PathBuf, config: &DisplayConfig) {
    let raw = std::fs::read_to_string(settings_path).unwrap_or_default();
    let mut settings = Settings::parse_yaml(&raw);
    settings.displays = config.clone();
    if let Err(e) = write_settings(settings_path, &settings) {
        log::error!("tray: failed to persist display config: {e}");
    }
}

fn rebuild_tray_menu<R: Runtime>(app: &AppHandle<R>) {
    let focuses = app
        .try_state::<crate::ui_bridge::CommandsState>()
        .and_then(|s| s.0.list_focuses().ok())
        .unwrap_or_default();
    if let Some(tray) = app.tray_by_id("main-tray") {
        if let Ok(menu) = build_menu(app, &focuses) {
            let _ = tray.set_menu(Some(menu));
        }
    }
}

fn red_icon() -> Image<'static> {
    const SIZE: u32 = 16;
    let rgba: Vec<u8> = (0..SIZE * SIZE)
        .flat_map(|_| [220u8, 38, 38, 255])
        .collect();
    Image::new_owned(rgba, SIZE, SIZE)
}
