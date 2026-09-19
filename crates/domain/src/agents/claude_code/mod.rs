//! What the ranch asks of Claude Code, and how it is asked.
//!
//! The only thing the ranch installs is a set of [`hooks`] in Claude's settings file.
//! Claude keeps its own account of what it is running, under `~/.claude/sessions/`,
//! and the ranch deliberately does not read it: a session is drawn from what it told
//! us, so there is no second source of truth to reconcile.

pub mod hooks;
