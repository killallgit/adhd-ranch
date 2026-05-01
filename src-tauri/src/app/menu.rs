use tauri::menu::{Menu, MenuEvent, MenuItemBuilder, PredefinedMenuItem, Submenu, SubmenuBuilder};
use tauri::{AppHandle, Manager, Runtime};

pub const HIDE_WIDGET_ID: &str = "hide-widget";

pub fn build<R: Runtime>(handle: &AppHandle<R>) -> tauri::Result<Menu<R>> {
    let app_submenu: Submenu<R> = SubmenuBuilder::new(handle, "Adhd Ranch")
        .item(&PredefinedMenuItem::quit(handle, None)?)
        .build()?;

    let window_submenu: Submenu<R> = SubmenuBuilder::new(handle, "Window")
        .item(
            &MenuItemBuilder::with_id(HIDE_WIDGET_ID, "Hide Widget")
                .accelerator("CmdOrCtrl+W")
                .build(handle)?,
        )
        .build()?;

    Menu::with_items(handle, &[&app_submenu, &window_submenu])
}

pub fn handle_event<R: Runtime>(app: &AppHandle<R>, event: MenuEvent) {
    if event.id().0 == HIDE_WIDGET_ID {
        if let Some(window) = app.get_webview_window("main") {
            let _ = window.hide();
        }
    }
}
