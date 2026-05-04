use std::path::PathBuf;
use std::sync::Arc;

use adhd_ranch_domain::{cap_state, Focus, Settings, Widget};
use adhd_ranch_storage::{write_settings, FocusStore};
use tauri::image::Image;
use tauri::menu::{
    CheckMenuItemBuilder, IsMenuItem, Menu, MenuItemBuilder, PredefinedMenuItem, SubmenuBuilder,
};
use tauri::tray::{TrayIcon, TrayIconBuilder};
use tauri::{AppHandle, Emitter, Manager, Wry};

use super::{DisplayConfigState, MonitorsState, SettingsState};
use crate::display::DisplayManagerState;

const QUIT_ID: &str = "tray-quit";
const NO_FOCUSES_ID: &str = "tray-no-focuses";
const NEW_FOCUS_ID: &str = "tray-new-focus";
const GATHER_PIGS_ID: &str = "tray-gather-pigs";
const DELETE_PREFIX: &str = "tray-delete-";
const DISPLAY_PREFIX: &str = "tray-display-";
const TRAY_ALWAYS_ON_TOP_ID: &str = "tray-always-on-top";
const TRAY_CONFIRM_DELETE_ID: &str = "tray-confirm-delete";
#[cfg(debug_assertions)]
const DEVTOOLS_ID: &str = "tray-devtools";

pub fn setup(
    app: &AppHandle<Wry>,
    store: Arc<dyn FocusStore>,
    settings: Settings,
    settings_path: PathBuf,
) -> tauri::Result<TrayIcon<Wry>> {
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
            #[cfg(debug_assertions)]
            if id == DEVTOOLS_ID {
                if let Some(win) = app.get_webview_window("overlay-0") {
                    win.open_devtools();
                }
                return;
            }
            if id == GATHER_PIGS_ID {
                if let Some(win) = app.get_webview_window("overlay-0") {
                    let _ = win.emit("gather-pigs", ());
                }
            } else if id == QUIT_ID {
                app.exit(0);
            } else if id == NEW_FOCUS_ID {
                if let Some(win) = app.get_webview_window("new-focus") {
                    let _ = win.show();
                    let _ = win.set_focus();
                }
            } else if id == TRAY_ALWAYS_ON_TOP_ID {
                let app_handle = app.clone();
                let path = settings_path_for_handler.clone();
                std::thread::spawn(move || handle_always_on_top_toggle(app_handle, path));
            } else if id == TRAY_CONFIRM_DELETE_ID {
                let app_handle = app.clone();
                let path = settings_path_for_handler.clone();
                std::thread::spawn(move || handle_confirm_delete_toggle(app_handle, path));
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

pub fn rebuild_handler(
    tray: TrayIcon<Wry>,
    handle: AppHandle<Wry>,
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

/// Threshold: more monitors nest under a "Displays" submenu inside Settings.
const DISPLAY_SUBMENU_MIN_COUNT: usize = 4;

fn build_menu(handle: &AppHandle<Wry>, focuses: &[Focus]) -> tauri::Result<Menu<Wry>> {
    let widget = handle
        .try_state::<SettingsState>()
        .and_then(|s| s.0.lock().ok().map(|s| s.widget))
        .unwrap_or_default();

    let mut items: Vec<Box<dyn IsMenuItem<Wry>>> = Vec::new();

    let gather = MenuItemBuilder::with_id(GATHER_PIGS_ID, "Gather Pigs").build(handle)?;
    items.push(Box::new(gather));
    items.push(Box::new(PredefinedMenuItem::separator(handle)?));
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

    items.push(Box::new(PredefinedMenuItem::separator(handle)?));
    items.push(Box::new(build_settings_submenu(handle, &widget)?));

    #[cfg(debug_assertions)]
    {
        items.push(Box::new(PredefinedMenuItem::separator(handle)?));
        let devtools =
            MenuItemBuilder::with_id(DEVTOOLS_ID, "Open Overlay DevTools").build(handle)?;
        items.push(Box::new(devtools));
    }
    items.push(Box::new(PredefinedMenuItem::separator(handle)?));
    let quit = MenuItemBuilder::with_id(QUIT_ID, "Quit").build(handle)?;
    items.push(Box::new(quit));

    let item_refs: Vec<&dyn IsMenuItem<Wry>> = items.iter().map(|b| b.as_ref()).collect();
    Menu::with_items(handle, &item_refs)
}

fn build_settings_submenu(
    handle: &AppHandle<Wry>,
    widget: &Widget,
) -> tauri::Result<impl IsMenuItem<Wry>> {
    let always_on_top = CheckMenuItemBuilder::with_id(TRAY_ALWAYS_ON_TOP_ID, "Always on Top")
        .checked(widget.always_on_top)
        .build(handle)?;
    let confirm_delete =
        CheckMenuItemBuilder::with_id(TRAY_CONFIRM_DELETE_ID, "Confirm Before Delete")
            .checked(widget.confirm_delete)
            .build(handle)?;

    let window_sub = SubmenuBuilder::new(handle, "Window")
        .item(&always_on_top)
        .item(&confirm_delete)
        .build()?;

    let display_items = build_display_setting_items(handle)?;

    let mut settings_sub = SubmenuBuilder::new(handle, "Settings").item(&window_sub);

    for di in &display_items {
        settings_sub = settings_sub.item(di.as_ref());
    }

    settings_sub.build()
}

fn build_display_setting_items(
    handle: &AppHandle<Wry>,
) -> tauri::Result<Vec<Box<dyn IsMenuItem<Wry>>>> {
    let Some(monitors_state) = handle.try_state::<MonitorsState>() else {
        return Ok(Vec::new());
    };
    if monitors_state.0.is_empty() {
        return Ok(Vec::new());
    }

    let enabled_indices = handle
        .try_state::<DisplayConfigState>()
        .and_then(|s| s.0.lock().ok().map(|c| c.enabled_indices.clone()))
        .unwrap_or_else(|| vec![0]);

    let monitors = &monitors_state.0;
    let mut out: Vec<Box<dyn IsMenuItem<Wry>>> = Vec::new();

    if monitors.len() >= DISPLAY_SUBMENU_MIN_COUNT {
        let mut display_sub = SubmenuBuilder::new(handle, "Displays");
        for (idx, monitor) in monitors.iter().enumerate() {
            let checked = enabled_indices.contains(&idx);
            let item =
                CheckMenuItemBuilder::with_id(format!("{DISPLAY_PREFIX}{idx}"), &monitor.label)
                    .checked(checked)
                    .build(handle)?;
            display_sub = display_sub.item(&item);
        }
        out.push(Box::new(display_sub.build()?));
    } else {
        for (idx, monitor) in monitors.iter().enumerate() {
            let checked = enabled_indices.contains(&idx);
            let item =
                CheckMenuItemBuilder::with_id(format!("{DISPLAY_PREFIX}{idx}"), &monitor.label)
                    .checked(checked)
                    .build(handle)?;
            out.push(Box::new(item));
        }
    }

    Ok(out)
}

fn handle_always_on_top_toggle(app: AppHandle<Wry>, settings_path: PathBuf) {
    let new_val = {
        let Some(state) = app.try_state::<SettingsState>() else {
            return;
        };
        let Ok(mut s) = state.0.lock() else { return };
        s.widget.always_on_top = !s.widget.always_on_top;
        let v = s.widget.always_on_top;
        // Persist under lock so the applied value and persisted value are always in sync.
        if let Err(e) = write_settings(&settings_path, &s) {
            log::error!("tray: failed to persist settings: {e}");
        }
        v
    };

    if let Some(win) = app.get_webview_window("overlay-0") {
        super::window_always_on_top::apply(&win, new_val);
    }

    rebuild_tray_menu(&app);
}

fn handle_confirm_delete_toggle(app: AppHandle<Wry>, settings_path: PathBuf) {
    {
        let Some(state) = app.try_state::<SettingsState>() else {
            return;
        };
        let Ok(mut s) = state.0.lock() else { return };
        s.widget.confirm_delete = !s.widget.confirm_delete;
    }

    persist_settings(&app, &settings_path);
    rebuild_tray_menu(&app);
}

fn handle_delete(app: AppHandle<Wry>, focus_id: String) {
    let confirm = app
        .try_state::<SettingsState>()
        .and_then(|s| s.0.lock().ok().map(|s| s.widget.confirm_delete))
        .unwrap_or(true);

    if confirm {
        use tauri_plugin_dialog::{DialogExt, MessageDialogButtons, MessageDialogKind};
        let title = app
            .try_state::<crate::ui_bridge::CommandsState>()
            .and_then(|s| {
                s.0.list_focuses()
                    .ok()
                    .and_then(|fs| fs.into_iter().find(|f| f.id.0 == focus_id))
                    .map(|f| f.title)
            })
            .unwrap_or_else(|| "this focus".to_string());

        let confirmed = app
            .dialog()
            .message(format!("Delete \"{title}\"? This cannot be undone."))
            .title("Delete Focus")
            .kind(MessageDialogKind::Warning)
            .buttons(MessageDialogButtons::OkCancel)
            .blocking_show();

        if !confirmed {
            return;
        }
    }

    if let Some(state) = app.try_state::<crate::ui_bridge::CommandsState>() {
        if let Err(e) = state.0.delete_focus(&focus_id) {
            log::error!("tray delete_focus({focus_id:?}): {e}");
        }
    }
}

fn handle_display_toggle(app: AppHandle<Wry>, idx: usize, settings_path: PathBuf) {
    let Some(display_state) = app.try_state::<DisplayConfigState>() else {
        return;
    };
    let Some(monitors_state) = app.try_state::<MonitorsState>() else {
        return;
    };
    let Some(overlay_state) = app.try_state::<DisplayManagerState>() else {
        return;
    };

    if idx >= monitors_state.0.len() {
        return;
    }

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

    // Sync DisplayConfig change into SettingsState so persist_settings writes the full config.
    if let Some(state) = app.try_state::<SettingsState>() {
        if let Ok(mut s) = state.0.lock() {
            s.displays = new_config.clone();
        }
    }

    persist_settings(&app, &settings_path);

    let display_mgr = Arc::clone(&overlay_state.0);
    let monitors = monitors_state.0.clone();
    let config_for_main = new_config.clone();
    let app_for_main = app.clone();
    if let Err(e) = app.run_on_main_thread(move || {
        display_mgr.apply(&app_for_main, &monitors, &config_for_main);
    }) {
        log::error!("tray: run_on_main_thread failed: {e}");
    }

    rebuild_tray_menu(&app);
}

fn persist_settings(app: &AppHandle<Wry>, settings_path: &std::path::Path) {
    let Some(state) = app.try_state::<SettingsState>() else {
        return;
    };
    let Ok(s) = state.0.lock() else { return };
    if let Err(e) = write_settings(settings_path, &s) {
        log::error!("tray: failed to persist settings: {e}");
    }
}

fn rebuild_tray_menu(app: &AppHandle<Wry>) {
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
