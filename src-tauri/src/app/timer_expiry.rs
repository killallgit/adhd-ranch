use std::sync::Arc;
use std::time::Duration;

use adhd_ranch_domain::{tick, FocusTimer, TimerExpiredSource, TimerStatus};
use adhd_ranch_storage::FocusStore;
use tauri::{AppHandle, Emitter, Manager};
use tauri_plugin_notification::NotificationExt;

use super::SettingsState;

pub const TIMER_EXPIRED_EVENT: &str = "timer-expired";
const TICK_INTERVAL: Duration = Duration::from_secs(1);

#[derive(Clone, serde::Serialize)]
struct TimerExpiredPayload<'a> {
    focus_id: &'a str,
    focus_title: &'a str,
}

pub fn spawn(handle: AppHandle, store: Arc<dyn FocusStore>) {
    tauri::async_runtime::spawn(async move {
        let mut interval = tokio::time::interval(TICK_INTERVAL);
        loop {
            interval.tick().await;
            run_once(&handle, store.as_ref());
        }
    });
}

fn run_once(handle: &AppHandle, store: &dyn FocusStore) {
    let focuses = match store.list() {
        Ok(f) => f,
        Err(e) => {
            log::error!("timer_expiry: list failed: {e}");
            return;
        }
    };
    let now = current_unix_secs();
    let transitions = tick(now, &focuses);
    if transitions.is_empty() {
        return;
    }

    let notifications_enabled = handle
        .try_state::<SettingsState>()
        .map(|s| {
            s.0.lock()
                .map(|g| g.notifications.is_enabled(&TimerExpiredSource))
                .unwrap_or(true)
        })
        .unwrap_or(true);

    for t in transitions {
        let focus = focuses.iter().find(|f| f.id.0 == t.focus_id);
        let Some(focus) = focus else { continue };
        let Some(running_timer) = focus.timer.as_ref() else {
            continue;
        };
        let expired = FocusTimer {
            duration_secs: running_timer.duration_secs,
            started_at: running_timer.started_at,
            status: TimerStatus::Expired,
        };
        if let Err(e) = store.update_timer(&t.focus_id, &expired) {
            log::error!("timer_expiry: update_timer for {} failed: {e}", t.focus_id);
            continue;
        }
        let _ = handle.emit(
            TIMER_EXPIRED_EVENT,
            TimerExpiredPayload {
                focus_id: &t.focus_id,
                focus_title: &t.focus_title,
            },
        );
        if notifications_enabled {
            let _ = handle
                .notification()
                .builder()
                .title("Timer expired")
                .body(format!("{} reached its timer.", t.focus_title))
                .show();
        }
    }
}

fn current_unix_secs() -> i64 {
    time::OffsetDateTime::now_utc().unix_timestamp()
}
