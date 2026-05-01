use std::path::PathBuf;

use adhd_ranch_domain::Settings;
use adhd_ranch_storage::write_settings;
use tauri::menu::{
    CheckMenuItemBuilder, Menu, MenuEvent, MenuItemBuilder, MenuItemKind, PredefinedMenuItem,
    SubmenuBuilder,
};
use tauri::{AppHandle, Manager, Runtime};

use super::window_always_on_top;

pub const ALWAYS_ON_TOP_ID: &str = "always-on-top";
pub const SHOW_RANCH_ID: &str = "show-ranch";
pub const CLOSE_WINDOW_ID: &str = "close-window";
const MAIN_WINDOW: &str = "main";

pub fn build<R: Runtime>(handle: &AppHandle<R>, always_on_top: bool) -> tauri::Result<Menu<R>> {
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

    let window_submenu = SubmenuBuilder::new(handle, "Window")
        .item(
            &CheckMenuItemBuilder::with_id(ALWAYS_ON_TOP_ID, "Always on Top")
                .checked(always_on_top)
                .build(handle)?,
        )
        .separator()
        .item(&MenuItemBuilder::with_id(SHOW_RANCH_ID, "Show Ranch").build(handle)?)
        .build()?;

    Menu::with_items(
        handle,
        &[&app_submenu, &file_submenu, &edit_submenu, &window_submenu],
    )
}

pub fn handle_event<R: Runtime>(app: &AppHandle<R>, event: MenuEvent, settings_path: &PathBuf) {
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
        ALWAYS_ON_TOP_ID => toggle_always_on_top(app, settings_path),
        _ => {}
    }
}

fn toggle_always_on_top<R: Runtime>(app: &AppHandle<R>, settings_path: &PathBuf) {
    let Some(menu) = app.menu() else { return };
    let Some(MenuItemKind::Check(item)) = menu.get(ALWAYS_ON_TOP_ID) else {
        return;
    };

    let new_val = !item.is_checked().unwrap_or(false);
    let _ = item.set_checked(new_val);

    if let Some(window) = app.get_webview_window(MAIN_WINDOW) {
        window_always_on_top::apply(&window, new_val);
    }

    let raw = std::fs::read_to_string(settings_path).unwrap_or_default();
    let mut settings = Settings::parse_yaml(&raw);
    settings.widget.always_on_top = new_val;
    let _ = write_settings(settings_path, &settings);
}
