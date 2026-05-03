pub mod hit_test;
pub mod monitor;
pub mod overlay;

use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use adhd_ranch_domain::{DisplayConfig, PigRect, RectUpdater};
use serde::Serialize;
use tauri::{AppHandle, Wry};

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
    /// Signals the associated hit-test poller thread to exit.
    stop: Arc<AtomicBool>,
}

#[derive(Clone)]
pub struct DisplayManager {
    entries: Arc<Mutex<HashMap<String, OverlayEntry>>>,
    /// Set true while JS has an active drag; hit-test thread forces interactive.
    drag_active: Arc<AtomicBool>,
}

/// Trait boundary for the display subsystem so callers depend on behaviour, not concrete type.
pub trait DisplayService: Send + Sync {
    fn drag_active(&self) -> Arc<AtomicBool>;
    fn apply(&self, app: &AppHandle<Wry>, monitors: &[LogicalMonitor], config: &DisplayConfig);
}

pub struct DisplayManagerState(pub Arc<dyn DisplayService>);

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
}

impl DisplayService for DisplayManager {
    fn drag_active(&self) -> Arc<AtomicBool> {
        Arc::clone(&self.drag_active)
    }

    fn apply(&self, app: &AppHandle<Wry>, monitors: &[LogicalMonitor], config: &DisplayConfig) {
        // Filter by the monitor's own index field, not its enumeration position.
        let enabled: Vec<&LogicalMonitor> = monitors
            .iter()
            .filter(|m| config.enabled_indices.contains(&m.index))
            .collect();

        if enabled.is_empty() {
            overlay::destroy(app);
            if let Ok(mut map) = self.entries.lock() {
                if let Some(entry) = map.remove(OVERLAY_LABEL) {
                    // Signal the old poller to exit before the entry is dropped.
                    entry.stop.store(true, Ordering::Relaxed);
                }
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

        let (tester, stop) = if already_managed {
            let (t, s) = self
                .entries
                .lock()
                .ok()
                .and_then(|map| {
                    map.get(OVERLAY_LABEL)
                        .map(|e| (e.tester.clone(), Arc::clone(&e.stop)))
                })
                .unwrap_or_else(|| (PigHitTester::new(), Arc::new(AtomicBool::new(false))));
            (t, s)
        } else {
            let t = PigHitTester::new();
            let s = Arc::new(AtomicBool::new(false));
            if let Ok(mut map) = self.entries.lock() {
                map.insert(
                    OVERLAY_LABEL.to_string(),
                    OverlayEntry {
                        tester: t.clone(),
                        stop: Arc::clone(&s),
                    },
                );
            }
            (t, s)
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
                stop,
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
