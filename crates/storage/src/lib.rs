pub mod agent_hooks;
pub mod agent_session_store;
pub mod atomic;
mod focus_document;
pub mod focus_store;
pub mod settings_writer;
pub mod timer_store;
pub mod watcher;

pub use agent_hooks::{
    serve, AgentHooks, ClaudeCodeHooks, ClaudeHookPaths, HookHistory, HookJournal, HookOutcome,
    HookServer,
};
pub use agent_session_store::{AgentSessionStore, HookEventSink, LiveSessions};
pub use atomic::atomic_write;
pub use focus_store::{FocusStore, FocusStoreError, MarkdownFocusStore};
pub use settings_writer::write_settings;
pub use timer_store::{InMemoryTimerStore, TimerStore};
pub use watcher::{watch_path, FocusWatcher, WatcherError};
