use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use adhd_ranch_domain::{DisplayConfig, PigRect, RectUpdater};
use tauri::{AppHandle, Manager, Runtime, WebviewUrl, WebviewWindowBuilder};

use crate::app::pig_hittest::PigHitTester;

const OVERLAY_LABEL: &str = "overlay-0";

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

pub struct OverlayManagerState(pub OverlayManager);

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

    /// Resize/create the single spanning overlay to cover all enabled monitors.
    /// One window spans all enabled displays so pigs roam freely across them.
    pub fn apply<R: Runtime>(
        &self,
        app: &AppHandle<R>,
        monitors: &[MonitorInfo],
        config: &DisplayConfig,
    ) {
        let enabled: Vec<&MonitorInfo> = monitors
            .iter()
            .enumerate()
            .filter(|(idx, _)| config.enabled_indices.contains(idx))
            .map(|(_, m)| m)
            .collect();

        if enabled.is_empty() {
            self.destroy(app, OVERLAY_LABEL);
            return;
        }

        // Union bounding box of all enabled monitors in physical pixels.
        let min_x = enabled.iter().map(|m| m.position.x).min().unwrap_or(0);
        let min_y = enabled.iter().map(|m| m.position.y).min().unwrap_or(0);
        let max_x = enabled
            .iter()
            .map(|m| m.position.x + m.size.width as i32)
            .max()
            .unwrap_or(1920);
        let max_y = enabled
            .iter()
            .map(|m| m.position.y + m.size.height as i32)
            .max()
            .unwrap_or(1080);

        let spanning = MonitorInfo {
            name: None,
            size: tauri::PhysicalSize::new(
                (max_x - min_x).max(1) as u32,
                (max_y - min_y).max(1) as u32,
            ),
            position: tauri::PhysicalPosition::new(min_x, min_y),
        };

        if let Err(e) = self.ensure_shown(app, OVERLAY_LABEL, &spanning) {
            log::error!("overlay_manager: {OVERLAY_LABEL} failed: {e}");
        }

        // Clean up any legacy per-monitor windows (overlay-1, overlay-2, …).
        for idx in 1..=monitors.len() {
            let label = format!("overlay-{idx}");
            if app.get_webview_window(&label).is_some() {
                self.destroy(app, &label);
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

        // overlay-0 is no longer pre-defined in tauri.conf.json so this always
        // takes the else branch on first call. Subsequent calls (display toggle)
        // hit the if branch and resize the existing window.
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

        // set_size/set_position use physical pixels — unambiguous regardless of DPR.
        // These run synchronously on the main thread before JS gets a chance to
        // execute, so window.innerWidth in React reflects the correct spanning size.
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

    fn destroy<R: Runtime>(&self, app: &AppHandle<R>, label: &str) {
        if let Some(window) = app.get_webview_window(label) {
            let _ = window.close();
        }
        if let Ok(mut map) = self.entries.lock() {
            map.remove(label);
        }
    }
}

impl RectUpdater for OverlayManager {
    fn update_rects(&self, label: &str, rects: Vec<PigRect>) {
        if let Ok(map) = self.entries.lock() {
            if let Some(entry) = map.get(label) {
                entry.tester.update(rects);
            }
        }
    }
}
