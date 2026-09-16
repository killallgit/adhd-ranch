use std::sync::Arc;
use std::time::Duration;

use adhd_ranch_commands::{NotificationRequest, NotificationSink, Timers};
use tauri::AppHandle;
use tauri_plugin_notification::NotificationExt;

const TICK_INTERVAL: Duration = Duration::from_secs(1);

pub struct TauriNotificationSink {
    handle: AppHandle,
}

impl TauriNotificationSink {
    pub fn new(handle: AppHandle) -> Self {
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

pub fn spawn(timers: Arc<Timers>) {
    tauri::async_runtime::spawn(async move {
        let mut interval = tokio::time::interval(TICK_INTERVAL);
        loop {
            interval.tick().await;
            // The store is synchronous file I/O; move it off the async runtime
            // thread so concurrent IPC handlers aren't blocked for a tick.
            let timers = timers.clone();
            match tokio::task::spawn_blocking(move || timers.expire_due(current_unix_secs())).await
            {
                Ok(Ok(())) => {}
                Ok(Err(e)) => log::error!("timer_expiry: expiry failed: {e}"),
                Err(e) => log::error!("timer_expiry: worker join failed: {e}"),
            }
        }
    });
}

fn current_unix_secs() -> i64 {
    time::OffsetDateTime::now_utc().unix_timestamp()
}
