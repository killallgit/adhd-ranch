use tauri::{Runtime, WebviewWindow};

#[cfg(target_os = "macos")]
pub fn apply<R: Runtime>(window: &WebviewWindow<R>, on: bool) {
    use objc2::msg_send;
    use objc2::runtime::AnyObject;

    let raw = match window.ns_window() {
        Ok(ptr) => ptr as *mut AnyObject,
        Err(_) => return,
    };
    if raw.is_null() {
        return;
    }

    let level: i64 = if on { 3 } else { 0 };
    unsafe {
        let _: () = msg_send![&*raw, setLevel: level];
    }
}

#[cfg(not(target_os = "macos"))]
pub fn apply<R: Runtime>(_window: &WebviewWindow<R>, _on: bool) {}
