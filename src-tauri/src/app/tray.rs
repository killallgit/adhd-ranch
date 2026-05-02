use std::sync::Arc;

use adhd_ranch_domain::{cap_state, Focus, Settings};
use adhd_ranch_storage::FocusStore;
use tauri::image::Image;
use tauri::menu::{IsMenuItem, Menu, MenuItemBuilder, PredefinedMenuItem, SubmenuBuilder};
use tauri::tray::{TrayIcon, TrayIconBuilder};
use tauri::{AppHandle, Manager, Runtime};

const QUIT_ID: &str = "tray-quit";
const NO_FOCUSES_ID: &str = "tray-no-focuses";
const DELETE_PREFIX: &str = "tray-delete-";

pub fn setup<R: Runtime>(
    app: &AppHandle<R>,
    store: Arc<dyn FocusStore>,
    settings: Settings,
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

    let mut builder = TrayIconBuilder::new()
        .menu(&menu)
        .show_menu_on_left_click(true)
        .on_menu_event(|app, event| {
            let id = event.id().0.as_str();
            if id == QUIT_ID {
                app.exit(0);
            } else if let Some(focus_id) = id.strip_prefix(DELETE_PREFIX) {
                let focus_id = focus_id.to_string();
                let app_handle = app.clone();
                std::thread::spawn(move || handle_delete(app_handle, focus_id));
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

fn red_icon() -> Image<'static> {
    const SIZE: u32 = 16;
    let rgba: Vec<u8> = (0..SIZE * SIZE)
        .flat_map(|_| [220u8, 38, 38, 255])
        .collect();
    Image::new_owned(rgba, SIZE, SIZE)
}
