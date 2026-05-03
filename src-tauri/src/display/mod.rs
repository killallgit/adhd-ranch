pub mod hit_test;
pub mod monitor;
pub mod overlay;

use std::collections::HashMap;
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex};

use adhd_ranch_domain::{DisplayConfig, PigRect, RectUpdater};
use serde::Serialize;
use tauri::{AppHandle, Runtime};

use hit_test::PigHitTester;
use monitor::LogicalMonitor;

const OVERLAY_LABEL: &str = "overlay-0";

/// CSS-space region of the first enabled monitor within the spanning overlay window.
/// Sent to React so pig spawning is confined to a visible display.
#[derive(Serialize, Clone, Debug)]
pub struct PrimaryRegion {
    pub x: f64,
    pub y: f64,
    pub w: f64,
    pub h: f64,
}

struct OverlayEntry {
    tester: PigHitTester,
}

#[derive(Clone)]
pub struct DisplayManager {
    entries: Arc<Mutex<HashMap<String, OverlayEntry>>>,
    /// Set true while JS has an active drag; hit-test thread forces interactive.
    drag_active: Arc<AtomicBool>,
}

pub struct DisplayManagerState(pub DisplayManager);

impl Default for DisplayManager {
    fn default() -> Self {
        Self::new()
    }
}

impl DisplayManager {
    pub fn new() -> Self {
        Self {
            entries: Arc::new(Mutex::new(HashMap::new())),
            drag_active: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn drag_active(&self) -> Arc<AtomicBool> {
        Arc::clone(&self.drag_active)
    }

    pub fn apply<R: Runtime>(
        &self,
        app: &AppHandle<R>,
        monitors: &[LogicalMonitor],
        config: &DisplayConfig,
    ) {
        let enabled: Vec<&LogicalMonitor> = monitors
            .iter()
            .enumerate()
            .filter(|(idx, _)| config.enabled_indices.contains(idx))
            .map(|(_, m)| m)
            .collect();

        if enabled.is_empty() {
            overlay::destroy(app);
            if let Ok(mut map) = self.entries.lock() {
                map.remove(OVERLAY_LABEL);
            }
            return;
        }

        let bounds =
            monitor::compute_span(&enabled.iter().map(|m| (*m).clone()).collect::<Vec<_>>());

        // CSS offset of first enabled monitor within the span.
        let primary = enabled[0];
        let primary_region = PrimaryRegion {
            x: primary.position.0 - bounds.x,
            y: primary.position.1 - bounds.y,
            w: primary.size.0,
            h: primary.size.1,
        };
        log::info!("display: primary_region={primary_region:?}");

        let already_managed = self
            .entries
            .lock()
            .map(|map| map.contains_key(OVERLAY_LABEL))
            .unwrap_or(false);

        let tester = if already_managed {
            self.entries
                .lock()
                .ok()
                .and_then(|map| map.get(OVERLAY_LABEL).map(|e| e.tester.clone()))
                .unwrap_or_default()
        } else {
            let t = PigHitTester::new();
            if let Ok(mut map) = self.entries.lock() {
                map.insert(
                    OVERLAY_LABEL.to_string(),
                    OverlayEntry { tester: t.clone() },
                );
            }
            t
        };

        if let Err(e) = overlay::ensure_shown(
            app,
            overlay::ShowParams {
                x: bounds.x,
                y: bounds.y,
                width: bounds.width,
                height: bounds.height,
                tester: &tester,
                already_managed,
                primary_region: &primary_region,
                drag_active: Arc::clone(&self.drag_active),
            },
        ) {
            log::error!("display: overlay failed: {e}");
        }

        overlay::cleanup_legacy(app, monitors.len());
    }
}

impl RectUpdater for DisplayManager {
    fn update_rects(&self, label: &str, rects: Vec<PigRect>) {
        if let Ok(map) = self.entries.lock() {
            if let Some(entry) = map.get(label) {
                entry.tester.update(rects);
            }
        }
    }
}
