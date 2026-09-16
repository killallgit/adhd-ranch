pub mod agent_session_store;
pub mod atomic;
pub mod claude_hook_installer;
mod focus_document;
pub mod focus_store;
pub mod settings_writer;
pub mod timer_store;
pub mod watcher;

pub use agent_session_store::{AgentSessionStore, ClaudeSessionStore};
pub use atomic::atomic_write;
pub use claude_hook_installer::{install_claude_session_hook, ClaudeHookInstall, ClaudeHookPaths};
pub use focus_store::{FocusStore, FocusStoreError, MarkdownFocusStore};
pub use settings_writer::write_settings;
pub use timer_store::{InMemoryTimerStore, TimerStore};
pub use watcher::{watch_path, FocusWatcher, WatcherError};
