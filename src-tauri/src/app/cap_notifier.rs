use adhd_ranch_commands::CapNotifier;
use tauri::AppHandle;
use tauri_plugin_notification::NotificationExt;

pub struct TauriCapNotifier {
    handle: AppHandle,
}

impl TauriCapNotifier {
    pub fn new(handle: AppHandle) -> Self {
        Self { handle }
    }
}

impl CapNotifier for TauriCapNotifier {
    fn focuses_over_cap(&self, max: usize) {
        let _ = self
            .handle
            .notification()
            .builder()
            .title("Too many focuses")
            .body(format!(
                "You're over the limit of {max} focuses — trim one."
            ))
            .show();
    }

    fn focuses_under_cap(&self) {}

    fn task_over_cap(&self, focus_id: &str, max: usize) {
        let _ = self
            .handle
            .notification()
            .builder()
            .title("Focus has too many tasks")
            .body(format!("Focus {focus_id} has more than {max} tasks."))
            .show();
    }

    fn task_under_cap(&self, _focus_id: &str) {}
}
