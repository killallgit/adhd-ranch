use tauri::{Runtime, WebviewWindow};

#[cfg(target_os = "macos")]
pub fn apply<R: Runtime>(window: &WebviewWindow<R>, name: &str) {
    use objc2::msg_send;
    use objc2::runtime::AnyObject;
    use objc2_foundation::NSString;

    let raw = match window.ns_window() {
        Ok(ptr) => ptr as *mut AnyObject,
        Err(_) => return,
    };
    if raw.is_null() {
        return;
    }

    let ns_name = NSString::from_str(name);
    unsafe {
        let _: () = msg_send![&*raw, setFrameAutosaveName: &*ns_name];
    }
}

#[cfg(not(target_os = "macos"))]
pub fn apply<R: Runtime>(_window: &WebviewWindow<R>, _name: &str) {}
