//! The entries the ranch keeps in Claude Code's settings file.
//!
//! Only this module knows those entries are worth writing on some targets and not
//! others: the command they register runs a client that delivers over a Unix socket,
//! so where there is no such socket there is nothing worth registering. Everything
//! above it gets the same `ClaudeCodeHooks` on every platform, so no caller has to
//! gate itself — the same bargain the hook server already makes with its listener.

use std::path::PathBuf;

#[cfg_attr(unix, path = "unix.rs")]
#[cfg_attr(not(unix), path = "inert.rs")]
mod imp;

pub use imp::ClaudeCodeHooks;

/// What the user sees in logs and in the debug window when these hooks change.
const AGENT: &str = "Claude Code";

pub struct ClaudeHookPaths {
    /// Claude Code's own settings file, which it shares with every other tool the
    /// user has pointed at it.
    pub settings_file: PathBuf,
    /// The client Claude runs, which ships beside the app.
    pub client_bin: PathBuf,
    /// Where the running ranch is listening.
    pub socket_path: PathBuf,
}
