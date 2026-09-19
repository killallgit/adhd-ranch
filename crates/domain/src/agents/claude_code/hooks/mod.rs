//! Asking Claude Code to tell the ranch what its sessions are doing.
//!
//! Claude runs a command of ours when something happens. Four pieces, deliberately
//! kept apart:
//!
//! - `events` — which of Claude's events we register, and why not the others.
//! - `command` — what each one runs: a client that hands the firing straight to the
//!   running ranch.
//! - `settings` — putting those entries into Claude's settings file and taking them
//!   back out, without disturbing what anyone else put there.
//! - `payload` — reading what a firing delivered.
//!
//! All of it is pure. Nothing here opens a file; see `adhd-ranch-storage`.

mod command;
mod events;
mod payload;
mod settings;

pub use command::HookCommand;
pub use events::{SessionHook, SESSION_HOOKS};
pub use payload::{agent_session_from_payload, session_id_from_payload};
pub use settings::{install, uninstall};
