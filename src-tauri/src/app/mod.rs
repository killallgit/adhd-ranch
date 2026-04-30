use crate::ui_bridge;

pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![ui_bridge::health])
        .run(tauri::generate_context!())
        .expect("tauri runtime error");
}
