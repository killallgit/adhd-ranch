use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use adhd_ranch_domain::DisplayConfig;
use tauri::{AppHandle, Manager, Runtime, WebviewUrl, WebviewWindowBuilder};

use crate::app::pig_hittest::{PigHitTester, PigRect};

#[derive(Clone)]
pub struct MonitorInfo {
    pub name: Option<String>,
    pub size: tauri::PhysicalSize<u32>,
    pub position: tauri::PhysicalPosition<i32>,
}

struct OverlayEntry {
    tester: PigHitTester,
}

#[derive(Clone)]
pub struct OverlayManager {
    entries: Arc<Mutex<HashMap<String, OverlayEntry>>>,
}

impl Default for OverlayManager {
    fn default() -> Self {
        Self::new()
    }
}

impl OverlayManager {
    pub fn new() -> Self {
        Self {
            entries: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn update_rects(&self, label: &str, rects: Vec<PigRect>) {
        if let Ok(map) = self.entries.lock() {
            if let Some(entry) = map.get(label) {
                entry.tester.update(rects);
            }
        }
    }

    pub fn apply<R: Runtime>(
        &self,
        app: &AppHandle<R>,
        monitors: &[MonitorInfo],
        config: &DisplayConfig,
    ) {
        for (idx, monitor) in monitors.iter().enumerate() {
            let label = format!("overlay-{idx}");
            if config.enabled_indices.contains(&idx) {
                if let Err(e) = self.ensure_shown(app, &label, monitor) {
                    log::error!("overlay_manager: {label} failed: {e}");
                }
            } else if let Some(window) = app.get_webview_window(&label) {
                let _ = window.hide();
            }
        }
    }

    fn ensure_shown<R: Runtime>(
        &self,
        app: &AppHandle<R>,
        label: &str,
        monitor: &MonitorInfo,
    ) -> tauri::Result<()> {
        let already_managed = self
            .entries
            .lock()
            .map(|map| map.contains_key(label))
            .unwrap_or(false);

        let window = if let Some(w) = app.get_webview_window(label) {
            w
        } else {
            WebviewWindowBuilder::new(app, label, WebviewUrl::App(Default::default()))
                .decorations(false)
                .transparent(true)
                .resizable(false)
                .visible(false)
                .build()?
        };

        let _ = window.set_size(tauri::PhysicalSize::new(
            monitor.size.width,
            monitor.size.height,
        ));
        let _ = window.set_position(tauri::PhysicalPosition::new(
            monitor.position.x,
            monitor.position.y,
        ));
        crate::app::window_always_on_top::apply(&window, true);
        let _ = window.show();

        if !already_managed {
            let tester = PigHitTester::new();
            let tester_thread = tester.clone();

            if let Ok(mut map) = self.entries.lock() {
                map.insert(label.to_string(), OverlayEntry { tester });
            }

            let app_handle = app.clone();
            let win_clone = window.clone();
            let label_owned = label.to_string();
            std::thread::spawn(move || {
                let _ = win_clone.set_ignore_cursor_events(true);
                let mut last_over = false;
                loop {
                    if app_handle.get_webview_window(&label_owned).is_none() {
                        break;
                    }
                    if let Ok(cursor) = app_handle.cursor_position() {
                        let origin = win_clone
                            .outer_position()
                            .map(|p| (p.x as f64, p.y as f64))
                            .unwrap_or((0.0, 0.0));
                        let local_x = cursor.x - origin.0;
                        let local_y = cursor.y - origin.1;
                        let over = tester_thread.is_hit(local_x, local_y);
                        if over != last_over {
                            let _ = win_clone.set_ignore_cursor_events(!over);
                            last_over = over;
                        }
                    }
                    std::thread::sleep(Duration::from_millis(16));
                }
            });
        }

        Ok(())
    }
}
