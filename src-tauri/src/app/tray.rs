use std::sync::Arc;

use adhd_ranch_domain::{cap_state, Focus, Settings};
use adhd_ranch_storage::FocusStore;
use tauri::image::Image;
use tauri::menu::{IsMenuItem, Menu, MenuItemBuilder, PredefinedMenuItem};
use tauri::tray::{TrayIcon, TrayIconBuilder};
use tauri::{AppHandle, Runtime};

const QUIT_ID: &str = "tray-quit";
const NO_FOCUSES_ID: &str = "tray-no-focuses";

pub fn setup<R: Runtime>(
    app: &AppHandle<R>,
    store: Arc<dyn FocusStore>,
    settings: Settings,
) -> tauri::Result<TrayIcon<R>> {
    let focuses = store.list().unwrap_or_default();
    let menu = build_menu(app, &focuses)?;
    let over_cap = cap_state(&focuses, settings.caps).any_over();

    let mut builder = TrayIconBuilder::new()
        .menu(&menu)
        .show_menu_on_left_click(true)
        .on_menu_event(|app, event| {
            if event.id().0 == QUIT_ID {
                app.exit(0);
            }
        });

    if let Some(icon) = app.default_window_icon() {
        builder = builder.icon(icon.clone());
    }

    let tray = builder.build(app)?;

    if over_cap {
        let _ = tray.set_icon(Some(red_icon()));
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
        let focuses = store.list().unwrap_or_default();
        if let Ok(menu) = build_menu(&handle, &focuses) {
            let _ = tray.set_menu(Some(menu));
        }
        let over_cap = cap_state(&focuses, settings.caps).any_over();
        if over_cap {
            let _ = tray.set_icon(Some(red_icon()));
        } else if let Some(icon) = handle.default_window_icon() {
            let _ = tray.set_icon(Some(icon.clone()));
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
        for (i, focus) in focuses.iter().enumerate() {
            let item = MenuItemBuilder::with_id(format!("tray-focus-{i}"), &focus.title)
                .enabled(false)
                .build(handle)?;
            items.push(Box::new(item));
        }
    }

    let sep = PredefinedMenuItem::separator(handle)?;
    let quit = MenuItemBuilder::with_id(QUIT_ID, "Quit").build(handle)?;
    items.push(Box::new(sep));
    items.push(Box::new(quit));

    let item_refs: Vec<&dyn IsMenuItem<R>> = items.iter().map(|b| b.as_ref()).collect();
    Menu::with_items(handle, &item_refs)
}

fn red_icon() -> Image<'static> {
    const SIZE: u32 = 16;
    let rgba: Vec<u8> = (0..SIZE * SIZE)
        .flat_map(|_| [220u8, 38, 38, 255])
        .collect();
    Image::new_owned(rgba, SIZE, SIZE)
}
