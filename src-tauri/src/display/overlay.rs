use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

use tauri::{AppHandle, Emitter, Manager, WebviewUrl, WebviewWindowBuilder, Wry};

use super::hit_test::PigHitTester;
use super::PrimaryRegion;

const OVERLAY_LABEL: &str = "overlay-0";

pub struct ShowParams<'a> {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    pub tester: &'a PigHitTester,
    pub already_managed: bool,
    pub primary_region: &'a PrimaryRegion,
    pub drag_active: Arc<AtomicBool>,
    /// Signals the hit-test poller to exit when set to true.
    pub stop: Arc<AtomicBool>,
}

pub fn ensure_shown(app: &AppHandle<Wry>, p: ShowParams<'_>) -> tauri::Result<()> {
    let ShowParams {
        x,
        y,
        width,
        height,
        tester,
        already_managed,
        primary_region,
        drag_active,
        stop,
    } = p;
    log::info!(
        "overlay::ensure_shown span=({x},{y} {width}×{height}) already_managed={already_managed}"
    );

    // If the window already exists (e.g. display toggle), resize/reposition it.
    // If it's new, embed size+position in the builder so WKWebView initialises
    // at the correct dimensions — set_size() after build() is overridden by
    // macOS's WKWebView initialisation and results in a default 800×600 frame.
    let window = if let Some(w) = app.get_webview_window(OVERLAY_LABEL) {
        log::info!("overlay: reusing existing window");
        let _ = w.set_size(tauri::LogicalSize::new(width, height));
        let _ = w.set_position(tauri::LogicalPosition::new(x, y));
        w
    } else {
        log::info!(
            "overlay: creating new window {}×{} @ ({},{})",
            width,
            height,
            x,
            y
        );
        let w = WebviewWindowBuilder::new(app, OVERLAY_LABEL, WebviewUrl::App(Default::default()))
            .decorations(false)
            .transparent(true)
            .resizable(false)
            .visible(false)
            .inner_size(width, height)
            .position(x, y)
            .build()?;

        // macOS demotes NSWindow level across key/resign-key transitions when
        // the window is not an NSPanel. The popover's text inputs make the
        // overlay key on click; re-apply on every focus event so the overlay
        // stays floating. Registered only on first creation — ensure_shown is
        // re-invoked on display config changes and would otherwise stack.
        let win_evt = w.clone();
        w.on_window_event(move |event| {
            if let tauri::WindowEvent::Focused(_) = event {
                crate::app::window_always_on_top::apply(&win_evt, true);
            }
        });

        w
    };

    crate::app::window_always_on_top::apply(&window, true);

    let show_result = window.show();
    log::info!("overlay: show={show_result:?}");

    // Read back to confirm OS accepted our values.
    if let (Ok(pos), Ok(sz)) = (window.outer_position(), window.inner_size()) {
        log::info!(
            "overlay: actual outer_position=({},{}) inner_size={}×{}",
            pos.x,
            pos.y,
            sz.width,
            sz.height
        );
    }
    let sf = window.scale_factor().unwrap_or(0.0);
    log::info!("overlay: window scale_factor={sf}");

    // Always emit primary region so React knows where to spawn pigs.
    // Re-emitted on display toggle so React updates the spawn zone.
    let _ = window.emit("display-region", primary_region.clone());

    if !already_managed {
        let tester_thread = tester.clone();
        let app_handle = app.clone();
        let win_clone = window.clone();
        let stop_thread = Arc::clone(&stop);
        std::thread::spawn(move || {
            let _ = win_clone.set_ignore_cursor_events(true);
            let mut last_over = false;
            loop {
                // Exit if signalled or if the overlay window has been destroyed.
                if stop_thread.load(Ordering::Relaxed)
                    || app_handle.get_webview_window(OVERLAY_LABEL).is_none()
                {
                    break;
                }
                if let Ok(cursor) = app_handle.cursor_position() {
                    // While a JS drag is active, keep the window interactive regardless of
                    // cursor position — avoids click-through breaking pointer capture when
                    // crossing monitor boundaries faster than the 16ms poll interval.
                    let over = if drag_active.load(Ordering::Relaxed) {
                        true
                    } else {
                        // cursor_position() and outer_position() are both physical pixels.
                        let origin = win_clone
                            .outer_position()
                            .map(|p| (p.x as f64, p.y as f64))
                            .unwrap_or((0.0, 0.0));
                        let local_x = cursor.x - origin.0;
                        let local_y = cursor.y - origin.1;
                        tester_thread.is_hit(local_x, local_y)
                    };
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

pub fn destroy(app: &AppHandle<Wry>) {
    if let Some(window) = app.get_webview_window(OVERLAY_LABEL) {
        let _ = window.close();
    }
}

pub fn cleanup_legacy(app: &AppHandle<Wry>, count: usize) {
    for idx in 1..=count {
        let label = format!("overlay-{idx}");
        if app.get_webview_window(&label).is_some() {
            if let Some(w) = app.get_webview_window(&label) {
                let _ = w.close();
            }
        }
    }
}
