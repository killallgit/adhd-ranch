//! Announcing hook firings to the windows that draw them.
//!
//! Three separate things happen to every firing, and they are separate on purpose:
//! the store decides what the ranch now looks like, the journal remembers what
//! arrived, and this tells the windows. Only this last one knows about Tauri.

use std::path::PathBuf;
use std::sync::Arc;

use adhd_ranch_commands::SettingsProvider;
use adhd_ranch_domain::agents::hooks::HookAction;
use adhd_ranch_storage::{HookEventSink, HookServer};
use tauri::{AppHandle, Emitter};

use super::{AGENT_HOOK_FIRED_EVENT, AGENT_SESSIONS_CHANGED_EVENT};

pub fn serve(
    app: &AppHandle,
    socket_path: PathBuf,
    recording: Arc<dyn HookEventSink>,
    settings: SettingsProvider,
) -> std::io::Result<HookServer> {
    adhd_ranch_storage::serve(
        socket_path,
        Arc::new(Announcing {
            recording,
            settings,
            app: app.clone(),
        }),
        {
            let app = app.clone();
            Arc::new(move || emit(&app, AGENT_SESSIONS_CHANGED_EVENT))
        },
    )
}

/// Applies and announces firings only while Agents as Animals is on. The plugin
/// remains installed when off, but Ranch keeps no hidden sessions in that state.
struct Announcing {
    recording: Arc<dyn HookEventSink>,
    settings: SettingsProvider,
    app: AppHandle,
}

impl HookEventSink for Announcing {
    fn apply(&self, action: HookAction, payload: &str) -> bool {
        if !self.settings.get().agents.enabled {
            return false;
        }
        let changed = self.recording.apply(action, payload);
        emit(&self.app, AGENT_HOOK_FIRED_EVENT);
        changed
    }
}

fn emit(app: &AppHandle, event: &str) {
    if let Err(e) = app.emit(event, ()) {
        log::error!("agent hooks: emit {event} failed: {e}");
    }
}
