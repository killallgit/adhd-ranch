use adhd_ranch_domain::WindowLevel;
use tauri::{Runtime, WebviewWindow};

#[cfg(target_os = "macos")]
pub fn apply<R: Runtime>(window: &WebviewWindow<R>, level: WindowLevel) {
    use objc2::msg_send;
    use objc2::runtime::AnyObject;

    let raw = match window.ns_window() {
        Ok(ptr) => ptr as *mut AnyObject,
        Err(_) => return,
    };
    if raw.is_null() {
        return;
    }

    let cocoa_level: i64 = match level {
        WindowLevel::Floating => 3,
        WindowLevel::Status => 25,
        WindowLevel::Screensaver => 1000,
    };

    unsafe {
        let _: () = msg_send![&*raw, setLevel: cocoa_level];
    }
}

#[cfg(not(target_os = "macos"))]
pub fn apply<R: Runtime>(_window: &WebviewWindow<R>, _level: WindowLevel) {}
