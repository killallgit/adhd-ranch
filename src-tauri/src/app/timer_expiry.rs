use std::sync::Arc;
use std::time::Duration;

use adhd_ranch_commands::{NotificationRequest, NotificationSink, TimerExpiryWorkflow};
use adhd_ranch_domain::Settings;
use adhd_ranch_storage::FocusStore;
use tauri::{AppHandle, Manager};
use tauri_plugin_notification::NotificationExt;

use super::SettingsState;

const TICK_INTERVAL: Duration = Duration::from_secs(1);

struct TauriNotificationSink {
    handle: AppHandle,
}

impl TauriNotificationSink {
    fn new(handle: AppHandle) -> Self {
        Self { handle }
    }
}

impl NotificationSink for TauriNotificationSink {
    fn notify(&self, request: NotificationRequest) {
        if let Err(e) = self
            .handle
            .notification()
            .builder()
            .title(request.title)
            .body(request.body)
            .show()
        {
            log::error!(
                "timer_expiry: system notification failed for {}: {e}",
                request.source_key
            );
        }
    }
}

pub fn spawn(handle: AppHandle, store: Arc<dyn FocusStore>) {
    tauri::async_runtime::spawn(async move {
        let mut interval = tokio::time::interval(TICK_INTERVAL);
        loop {
            interval.tick().await;
            // FocusStore is synchronous file I/O; move it off the async runtime
            // thread so concurrent IPC handlers aren't blocked for a tick.
            let handle = handle.clone();
            let store = store.clone();
            if let Err(e) = tokio::task::spawn_blocking(move || run_once(&handle, store)).await {
                log::error!("timer_expiry: worker join failed: {e}");
            }
        }
    });
}

fn run_once(handle: &AppHandle, store: Arc<dyn FocusStore>) {
    let now = current_unix_secs();
    let settings = handle
        .try_state::<SettingsState>()
        .and_then(|s| s.0.lock().ok().map(|settings| settings.clone()))
        .unwrap_or_else(Settings::default);
    let notifications: Arc<dyn NotificationSink> =
        Arc::new(TauriNotificationSink::new(handle.clone()));
    let workflow = TimerExpiryWorkflow::new(store, settings, notifications);

    if let Err(e) = workflow.run_once(now) {
        log::error!("timer_expiry: workflow failed: {e}");
    }
}

fn current_unix_secs() -> i64 {
    time::OffsetDateTime::now_utc().unix_timestamp()
}
