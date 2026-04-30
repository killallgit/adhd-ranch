pub mod tray;

use crate::ui_bridge;

pub fn run() {
    let mut builder = tauri::Builder::default()
        .plugin(tauri_plugin_positioner::init())
        .invoke_handler(tauri::generate_handler![ui_bridge::health]);

    builder = builder.setup(|app| {
        #[cfg(target_os = "macos")]
        app.set_activation_policy(tauri::ActivationPolicy::Accessory);

        tray::install(app.handle())?;
        Ok(())
    });

    builder = builder.on_window_event(|window, event| {
        if let tauri::WindowEvent::Focused(false) = event {
            if window.label() == "main" {
                let _ = window.hide();
            }
        }
    });

    builder
        .run(tauri::generate_context!())
        .expect("tauri runtime error");
}
