use tauri::{
    AppHandle, LogicalPosition, LogicalRect, LogicalSize, Manager, Monitor, Rect, Runtime,
    WebviewUrl, WebviewWindowBuilder,
};

fn panel_layout(
    work: LogicalRect<f64, f64>,
    anchor: Option<LogicalRect<f64, f64>>,
    frame: LogicalSize<f64>,
) -> LogicalRect<f64, f64> {
    let right = work.position.x + work.size.width;
    let bottom = work.position.y + work.size.height;
    let (center_x, top, available_height, above) = match anchor {
        Some(anchor) => {
            let below = (anchor.position.y + anchor.size.height).max(work.position.y);
            let above = anchor.position.y.min(bottom);
            if bottom - below >= above - work.position.y {
                (
                    anchor.position.x + anchor.size.width / 2.0,
                    below,
                    bottom - below,
                    false,
                )
            } else {
                (
                    anchor.position.x + anchor.size.width / 2.0,
                    above,
                    above - work.position.y,
                    true,
                )
            }
        }
        None => (right - 190.0, work.position.y, work.size.height, false),
    };
    let width = (380.0 + frame.width).min(work.size.width);
    let height = (620.0 + frame.height).min(available_height.max(frame.height + 1.0));
    LogicalRect {
        position: LogicalPosition::new(
            (center_x - width / 2.0).clamp(work.position.x, right - width),
            if above { top - height } else { top },
        ),
        size: LogicalSize::new(
            (width - frame.width).max(1.0),
            (height - frame.height).max(1.0),
        ),
    }
}

pub(super) fn open<R: Runtime>(app: &AppHandle<R>) -> tauri::Result<()> {
    let (monitor, anchor) = monitor_and_anchor(app)?;
    let window = match app.get_webview_window("settings") {
        Some(window) => window,
        None => WebviewWindowBuilder::new(app, "settings", WebviewUrl::App("settings.html".into()))
            .title("Preferences")
            .inner_size(380.0, 620.0)
            .decorations(true)
            .resizable(false)
            .always_on_top(true)
            .visible(false)
            .build()?,
    };
    let scale = monitor.scale_factor();
    let outer = window
        .outer_size()?
        .to_logical::<f64>(window.scale_factor()?);
    let inner = window
        .inner_size()?
        .to_logical::<f64>(window.scale_factor()?);
    let work = monitor.work_area();
    let layout = panel_layout(
        LogicalRect {
            position: work.position.to_logical(scale),
            size: work.size.to_logical(scale),
        },
        anchor,
        LogicalSize::new(
            (outer.width - inner.width).max(0.0),
            (outer.height - inner.height).max(0.0),
        ),
    );
    window.set_always_on_top(true)?;
    window.set_size(layout.size)?;
    #[cfg(target_os = "macos")]
    window.set_position(layout.position)?;
    #[cfg(not(target_os = "macos"))]
    window.set_position(layout.position.to_physical::<i32>(scale))?;
    #[cfg(target_os = "macos")]
    {
        use objc2::{msg_send, runtime::AnyObject};

        let raw = window.ns_window()? as *mut AnyObject;
        if let Some(window) = unsafe { raw.as_ref() } {
            unsafe {
                let _: () = msg_send![window, setMovable: false];
            }
        }
    }
    window.unminimize()?;
    window.show()?;
    window.set_focus()
}

fn monitor_and_anchor<R: Runtime>(
    app: &AppHandle<R>,
) -> tauri::Result<(Monitor, Option<LogicalRect<f64, f64>>)> {
    if let Some(tray) = app.tray_by_id("main-tray") {
        if let Some(rect) = tray.rect()? {
            #[cfg(target_os = "macos")]
            let point = tray_scale(&tray)?.map(|scale| rect.position.to_logical::<f64>(scale));
            #[cfg(not(target_os = "macos"))]
            let point = Some(rect.position.to_physical::<f64>(1.0));
            if let Some(point) = point {
                if let Some(monitor) = app.monitor_from_point(point.x, point.y)? {
                    let anchor = logical_rect(rect, monitor.scale_factor());
                    return Ok((monitor, Some(anchor)));
                }
            }
        }
    }
    let monitor = app
        .primary_monitor()?
        .or(app.available_monitors()?.into_iter().next());
    monitor.map(|monitor| (monitor, None)).ok_or_else(|| {
        std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "no monitor available for Settings",
        )
        .into()
    })
}

fn logical_rect(rect: Rect, scale: f64) -> LogicalRect<f64, f64> {
    LogicalRect {
        position: rect.position.to_logical(scale),
        size: rect.size.to_logical(scale),
    }
}

#[cfg(target_os = "macos")]
fn tray_scale<R: Runtime>(tray: &tauri::tray::TrayIcon<R>) -> tauri::Result<Option<f64>> {
    tray.with_inner_tray_icon(|tray| {
        use objc2::{msg_send, runtime::AnyObject};

        let item = tray.ns_status_item()?;
        let raw = &*item as *const _ as *const AnyObject;
        // Tray rectangles use their own screen's scale, which can differ from the primary screen.
        unsafe {
            let button: *mut AnyObject = msg_send![&*raw, button];
            let button = button.as_ref()?;
            let window: *mut AnyObject = msg_send![button, window];
            let window = window.as_ref()?;
            let scale: f64 = msg_send![window, backingScaleFactor];
            (scale.is_finite() && scale > 0.0).then_some(scale)
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rect(x: f64, y: f64, width: f64, height: f64) -> LogicalRect<f64, f64> {
        LogicalRect {
            position: (x, y).into(),
            size: (width, height).into(),
        }
    }

    #[test]
    fn opens_beneath_the_menu_bar_icon() {
        let layout = panel_layout(
            rect(0.0, 24.0, 1440.0, 840.0),
            Some(rect(800.0, 0.0, 24.0, 24.0)),
            (0.0, 28.0).into(),
        );

        assert_eq!(layout.position, LogicalPosition::new(622.0, 24.0));
        assert_eq!(layout.size, LogicalSize::new(380.0, 620.0));
    }

    #[test]
    fn clamps_to_the_right_edge_of_a_negative_origin_monitor() {
        let layout = panel_layout(
            rect(-1280.0, -776.0, 1280.0, 752.0),
            Some(rect(-24.0, -800.0, 24.0, 24.0)),
            (0.0, 28.0).into(),
        );

        assert_eq!(layout.position, LogicalPosition::new(-380.0, -776.0));
    }

    #[test]
    fn limits_inner_height_to_the_work_area_after_window_decorations() {
        let layout = panel_layout(
            rect(0.0, 40.0, 800.0, 360.0),
            Some(rect(700.0, 0.0, 24.0, 40.0)),
            (2.0, 30.0).into(),
        );

        assert_eq!(layout.position, LogicalPosition::new(418.0, 40.0));
        assert_eq!(layout.size, LogicalSize::new(380.0, 330.0));
    }

    #[test]
    fn opens_above_a_bottom_taskbar_icon() {
        let layout = panel_layout(
            rect(0.0, 0.0, 1920.0, 1040.0),
            Some(rect(1850.0, 1040.0, 24.0, 40.0)),
            (0.0, 28.0).into(),
        );

        assert_eq!(layout.position, LogicalPosition::new(1540.0, 392.0));
        assert_eq!(layout.size.height, 620.0);
    }

    #[test]
    fn missing_tray_rect_uses_the_work_area_top_right() {
        let layout = panel_layout(rect(-800.0, 40.0, 800.0, 360.0), None, (0.0, 28.0).into());

        assert_eq!(layout.position, LogicalPosition::new(-380.0, 40.0));
        assert_eq!(layout.size, LogicalSize::new(380.0, 332.0));
    }

    #[test]
    fn tray_and_work_area_use_the_same_secondary_monitor_scale() {
        let anchor = logical_rect(
            Rect {
                position: tauri::PhysicalPosition::new(-2400, -1600).into(),
                size: tauri::PhysicalSize::new(48, 48).into(),
            },
            2.0,
        );
        let work = logical_rect(
            Rect {
                position: tauri::PhysicalPosition::new(-2560, -1552).into(),
                size: tauri::PhysicalSize::new(2560, 1504).into(),
            },
            2.0,
        );

        let layout = panel_layout(work, Some(anchor), (0.0, 28.0).into());

        assert_eq!(layout.position, LogicalPosition::new(-1280.0, -776.0));
        assert_eq!(layout.size, LogicalSize::new(380.0, 620.0));
    }
}
