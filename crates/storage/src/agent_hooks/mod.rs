//! Installing and removing the ranch's hooks in a coding agent's configuration.
//!
//! These files belong to the agent and to whatever else the user has pointed at
//! them, so every implementation holds to the same three rules: touch only entries
//! the ranch wrote, leave a file alone entirely when its shape is not understood,
//! and be able to undo everything it did.

pub mod claude_code;
pub mod journal;
pub mod server;
mod settings_file;

use std::io;

pub use claude_code::{ClaudeCodeHooks, ClaudeHookPaths};
pub use journal::{HookHistory, HookJournal};
pub use server::{serve, HookServer};

/// What an install or uninstall did.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HookOutcome {
    Changed,
    /// The configuration already said what was wanted.
    AlreadyDone,
    /// The file could not be parsed, so it was left exactly as found.
    SettingsNotUnderstood,
}

/// One agent's hook lifecycle.
///
/// Agents differ in where their configuration lives, what an entry looks like in it
/// and which events they offer; what the ranch does with them does not differ, which
/// is what this trait is for.
pub trait AgentHooks {
    /// Names the agent in logs — the user needs to know whose settings changed.
    fn agent(&self) -> &'static str;

    /// Register any of this version's hooks the configuration does not already
    /// have. Safe to call on every launch.
    fn install(&self) -> io::Result<HookOutcome>;

    /// Remove the entries this version installs, matched exactly and never guessed
    /// at. Nothing else has to be undone: the hooks left nothing behind.
    fn uninstall(&self) -> io::Result<HookOutcome>;

    /// Whether the configuration already names this exact client and socket.
    ///
    /// Asked by the debug window, where "no hook has ever fired" is otherwise
    /// indistinguishable from "no agent is running".
    fn installed(&self) -> io::Result<bool>;
}
