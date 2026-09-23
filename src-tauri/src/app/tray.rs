use std::sync::Arc;

use adhd_ranch_commands::SettingsProvider;
use adhd_ranch_domain::{cap_state, Focus, Settings, TimerStatus};
use adhd_ranch_storage::FocusStore;
use tauri::image::Image;
use tauri::menu::{
    CheckMenuItemBuilder, IsMenuItem, Menu, MenuItemBuilder, PredefinedMenuItem, SubmenuBuilder,
};
use tauri::tray::{TrayIcon, TrayIconBuilder};
use tauri::{AppHandle, Emitter, Manager, Wry};

use super::{SettingsPathState, SettingsState};

const QUIT_ID: &str = "tray-quit";
const NO_FOCUSES_ID: &str = "tray-no-focuses";
const NEW_FOCUS_ID: &str = "tray-new-focus";
const GATHER_PIGS_ID: &str = "tray-gather-pigs";
const AGENTS_AS_ANIMALS_ID: &str = "tray-agents-as-animals";
const INSTALL_HOOKS_ID: &str = "tray-install-hooks";
const DELETE_PREFIX: &str = "tray-delete-";
const OPEN_FOCUS_PREFIX: &str = "tray-open-focus-";
const TRAY_OPEN_PREFS_ID: &str = "tray-open-prefs";
const OPEN_FOCUS_DETAIL_EVENT: &str = "open-focus-detail";
#[cfg(debug_assertions)]
const DEVTOOLS_ID: &str = "tray-devtools";

pub fn setup(
    app: &AppHandle<Wry>,
    store: Arc<dyn FocusStore>,
    settings: Settings,
) -> tauri::Result<TrayIcon<Wry>> {
    let focuses = match store.list() {
        Ok(f) => f,
        Err(e) => {
            log::error!("tray setup: failed to read focuses: {e}");
            Vec::new()
        }
    };
    let menu = build_menu(app, &focuses, settings.agents.enabled)?;
    let over_cap = cap_state(&focuses, settings.caps).any_over();

    let mut builder = TrayIconBuilder::with_id("main-tray")
        .menu(&menu)
        .show_menu_on_left_click(true)
        .on_menu_event(move |app, event| {
            let id = event.id().0.as_str();
            #[cfg(debug_assertions)]
            if id == DEVTOOLS_ID {
                if let Some(win) = app.get_webview_window("overlay-0") {
                    if win.is_devtools_open() {
                        win.close_devtools();
                    } else {
                        win.open_devtools();
                    }
                }
                return;
            }
            if id == GATHER_PIGS_ID {
                if let Some(win) = app.get_webview_window("overlay-0") {
                    let _ = win.emit("gather-pigs", ());
                }
            } else if id == AGENTS_AS_ANIMALS_ID {
                let app_handle = app.clone();
                std::thread::spawn(move || toggle_agents(app_handle));
            } else if id == INSTALL_HOOKS_ID {
                super::open_install_hooks_window(app);
            } else if id == QUIT_ID {
                app.exit(0);
            } else if id == NEW_FOCUS_ID {
                if let Some(win) = app.get_webview_window("new-focus") {
                    let _ = win.show();
                    let _ = win.set_focus();
                }
            } else if id == TRAY_OPEN_PREFS_ID {
                super::open_settings_window(app);
            } else if let Some(focus_id) = id.strip_prefix(DELETE_PREFIX) {
                let focus_id = focus_id.to_string();
                let app_handle = app.clone();
                std::thread::spawn(move || handle_delete(app_handle, focus_id));
            } else if let Some(focus_id) = id.strip_prefix(OPEN_FOCUS_PREFIX) {
                if let Some(win) = app.get_webview_window("overlay-0") {
                    let _ = win.emit(OPEN_FOCUS_DETAIL_EVENT, focus_id.to_string());
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
    settings: SettingsProvider,
) -> Box<dyn Fn() + Send + 'static> {
    Box::new(move || {
        let focuses = match store.list() {
            Ok(f) => f,
            Err(e) => {
                log::error!("tray rebuild: failed to read focuses: {e}");
                return;
            }
        };
        let current = settings.get();
        if let Ok(menu) = build_menu(&handle, &focuses, current.agents.enabled) {
            let _ = tray.set_menu(Some(menu));
        }
        let over_cap = cap_state(&focuses, current.caps).any_over();
        if over_cap {
            let _ = tray.set_icon(Some(red_icon()));
        } else if let Some(icon) = handle.default_window_icon() {
            let _ = tray.set_icon(Some(icon.clone()));
        } else {
            let _ = tray.set_icon(None);
        }
    })
}

fn build_menu(
    handle: &AppHandle<Wry>,
    focuses: &[Focus],
    agents_enabled: bool,
) -> tauri::Result<Menu<Wry>> {
    let mut items: Vec<Box<dyn IsMenuItem<Wry>>> = Vec::new();

    let gather = MenuItemBuilder::with_id(GATHER_PIGS_ID, "Gather Pigs").build(handle)?;
    items.push(Box::new(gather));
    let agents = CheckMenuItemBuilder::with_id(AGENTS_AS_ANIMALS_ID, "Agents as Animals")
        .checked(agents_enabled)
        .build(handle)?;
    items.push(Box::new(agents));
    let install_hooks =
        MenuItemBuilder::with_id(INSTALL_HOOKS_ID, "Install Hooks…").build(handle)?;
    items.push(Box::new(install_hooks));
    items.push(Box::new(PredefinedMenuItem::separator(handle)?));
    let new_focus = MenuItemBuilder::with_id(NEW_FOCUS_ID, "+ New Focus").build(handle)?;
    items.push(Box::new(new_focus));
    items.push(Box::new(PredefinedMenuItem::separator(handle)?));

    let sections = partition_focuses_for_menu(focuses);

    if sections.list.is_empty() {
        let item = MenuItemBuilder::with_id(NO_FOCUSES_ID, "No focuses yet")
            .enabled(false)
            .build(handle)?;
        items.push(Box::new(item));
    } else {
        for focus in sections.list {
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
        if !sections.expired.is_empty() {
            items.push(Box::new(PredefinedMenuItem::separator(handle)?));
            let expired_items = sections
                .expired
                .iter()
                .map(|focus| {
                    MenuItemBuilder::with_id(
                        format!("{OPEN_FOCUS_PREFIX}{}", focus.id.0),
                        focus.title.clone(),
                    )
                    .build(handle)
                })
                .collect::<tauri::Result<Vec<_>>>()?;
            let expired_refs: Vec<&dyn IsMenuItem<Wry>> = expired_items
                .iter()
                .map(|item| item as &dyn IsMenuItem<Wry>)
                .collect();
            let expired_submenu = SubmenuBuilder::new(handle, "Expired")
                .items(&expired_refs)
                .build()?;
            items.push(Box::new(expired_submenu));
        }
    }

    items.push(Box::new(PredefinedMenuItem::separator(handle)?));
    let settings_item = MenuItemBuilder::with_id(TRAY_OPEN_PREFS_ID, "Settings…").build(handle)?;
    items.push(Box::new(settings_item));

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

struct MenuFocusSections<'a> {
    list: Vec<&'a Focus>,
    expired: Vec<&'a Focus>,
}

fn partition_focuses_for_menu(focuses: &[Focus]) -> MenuFocusSections<'_> {
    let mut list = Vec::new();
    let mut expired = Vec::new();
    for focus in focuses {
        list.push(focus);
        if matches!(
            focus.timer.as_ref().map(|timer| &timer.status),
            Some(TimerStatus::Expired)
        ) {
            expired.push(focus);
        }
    }
    MenuFocusSections { list, expired }
}

fn with_agents_toggled(settings: &Settings) -> Settings {
    let mut next = settings.clone();
    next.agents.enabled = !settings.agents.enabled;
    next
}

fn toggle_agents(app: AppHandle<Wry>) {
    let (Some(state), Some(path)) = (
        app.try_state::<SettingsState>(),
        app.try_state::<SettingsPathState>(),
    ) else {
        return;
    };
    let Ok(current) = state.0.lock().map(|settings| settings.clone()) else {
        log::error!("tray toggle agents: settings lock poisoned");
        return;
    };
    let workflow = super::settings_workflow::workflow_for_app(
        app.clone(),
        Arc::clone(&state.0),
        path.0.clone(),
    );
    if let Err(e) = workflow.update(with_agents_toggled(&current)) {
        log::error!("tray toggle agents: {e:?}");
    } else if !current.agents.enabled {
        // The user enables the Ranch projection; Claude plugin installation is
        // still their choice and is never performed by this toggle.
        match super::claude_hook::plugin_enabled() {
            Ok(true) => {}
            Ok(false) => {
                let handle = app.clone();
                let _ = app.run_on_main_thread(move || super::open_install_hooks_window(&handle));
            }
            Err(error) => log::warn!("cannot check Claude plugin status: {error}"),
        }
    }
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

pub fn rebuild_tray_menu(app: &AppHandle<Wry>) {
    let focuses = app
        .try_state::<crate::ui_bridge::CommandsState>()
        .and_then(|s| s.0.list_focuses().ok())
        .unwrap_or_default();
    let agents_enabled = app
        .try_state::<SettingsState>()
        .and_then(|s| s.0.lock().ok().map(|s| s.agents.enabled))
        .unwrap_or(false);
    if let Some(tray) = app.tray_by_id("main-tray") {
        if let Ok(menu) = build_menu(app, &focuses, agents_enabled) {
            let _ = tray.set_menu(Some(menu));
        }
        let over_cap = app
            .try_state::<crate::ui_bridge::CommandsState>()
            .map(|s| cap_state(&focuses, s.0.caps()).any_over())
            .unwrap_or(false);
        if over_cap {
            let _ = tray.set_icon(Some(red_icon()));
        } else if let Some(icon) = app.default_window_icon() {
            let _ = tray.set_icon(Some(icon.clone()));
        } else {
            let _ = tray.set_icon(None);
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

#[cfg(test)]
mod tests {
    use super::*;
    use adhd_ranch_domain::{FocusId, FocusTimer};

    fn focus(id: &str, title: &str, status: Option<TimerStatus>) -> Focus {
        Focus {
            id: FocusId(id.to_string()),
            title: title.to_string(),
            description: String::new(),
            created_at: String::new(),
            tasks: Vec::new(),
            timer: status.map(|status| FocusTimer {
                duration_secs: 120,
                started_at: 1_000,
                status,
            }),
        }
    }

    #[test]
    fn toggling_agents_flips_only_the_agents_setting() {
        let settings = Settings::default();

        let toggled = with_agents_toggled(&settings);

        assert!(toggled.agents.enabled);
        assert_eq!(
            Settings {
                agents: settings.agents,
                ..toggled
            },
            settings
        );
    }

    #[test]
    fn keeps_expired_focuses_in_main_list_and_expired_section() {
        let focuses = vec![
            focus("a", "Active", Some(TimerStatus::Running)),
            focus("b", "Expired", Some(TimerStatus::Expired)),
            focus("c", "No timer", None),
        ];

        let sections = partition_focuses_for_menu(&focuses);

        assert_eq!(
            sections
                .list
                .iter()
                .map(|focus| focus.title.as_str())
                .collect::<Vec<_>>(),
            vec!["Active", "Expired", "No timer"]
        );
        assert_eq!(
            sections
                .expired
                .iter()
                .map(|focus| focus.title.as_str())
                .collect::<Vec<_>>(),
            vec!["Expired"]
        );
    }
}
