use std::path::PathBuf;

use tauri::menu::{
    CheckMenuItemBuilder, Menu, MenuEvent, MenuItemBuilder, MenuItemKind, PredefinedMenuItem,
    SubmenuBuilder,
};
use tauri::{AppHandle, Manager, Runtime};

pub const SHOW_RANCH_ID: &str = "show-ranch";
pub const CLOSE_WINDOW_ID: &str = "close-window";
pub const SHOW_DEBUG_OVERLAY_ID: &str = "show-debug-overlay";
const MAIN_WINDOW: &str = "overlay-0";

pub fn build<R: Runtime>(handle: &AppHandle<R>) -> tauri::Result<Menu<R>> {
    let app_submenu = SubmenuBuilder::new(handle, "Adhd Ranch")
        .item(&PredefinedMenuItem::about(handle, None, None)?)
        .separator()
        .item(&PredefinedMenuItem::quit(handle, None)?)
        .build()?;

    let file_submenu = SubmenuBuilder::new(handle, "File")
        .item(
            &MenuItemBuilder::with_id(CLOSE_WINDOW_ID, "Close")
                .accelerator("CmdOrCtrl+W")
                .build(handle)?,
        )
        .build()?;

    let edit_submenu = SubmenuBuilder::new(handle, "Edit")
        .item(&PredefinedMenuItem::undo(handle, None)?)
        .item(&PredefinedMenuItem::redo(handle, None)?)
        .separator()
        .item(&PredefinedMenuItem::cut(handle, None)?)
        .item(&PredefinedMenuItem::copy(handle, None)?)
        .item(&PredefinedMenuItem::paste(handle, None)?)
        .separator()
        .item(&PredefinedMenuItem::select_all(handle, None)?)
        .build()?;

    let mut window_sub = SubmenuBuilder::new(handle, "Window");

    #[cfg(debug_assertions)]
    {
        let debug_overlay = CheckMenuItemBuilder::with_id(SHOW_DEBUG_OVERLAY_ID, "Show Debug Overlay")
            .checked(true)
            .build(handle)?;
        window_sub = window_sub.item(&debug_overlay).separator();
    }

    let window_submenu = window_sub
        .item(&MenuItemBuilder::with_id(SHOW_RANCH_ID, "Show Ranch").build(handle)?)
        .build()?;

    Menu::with_items(
        handle,
        &[&app_submenu, &file_submenu, &edit_submenu, &window_submenu],
    )
}

pub fn handle_event<R: Runtime>(app: &AppHandle<R>, event: MenuEvent, _settings_path: &PathBuf) {
    match event.id().0.as_str() {
        CLOSE_WINDOW_ID => {
            if let Some(window) = app.get_webview_window(MAIN_WINDOW) {
                let _ = window.hide();
            }
        }
        SHOW_RANCH_ID => {
            if let Some(window) = app.get_webview_window(MAIN_WINDOW) {
                let _ = window.show();
                let _ = window.set_focus();
            }
        }
        SHOW_DEBUG_OVERLAY_ID => toggle_debug_overlay(app),
        _ => {}
    }
}

fn toggle_debug_overlay<R: Runtime>(app: &AppHandle<R>) {
    use tauri::Emitter;
    let Some(menu) = app.menu() else { return };
    let Some(MenuItemKind::Check(item)) = menu.get(SHOW_DEBUG_OVERLAY_ID) else {
        return;
    };
    let new_val = !item.is_checked().unwrap_or(false);
    let _ = item.set_checked(new_val);
    let _ = app.emit("debug-overlay-toggle", new_val);
}
