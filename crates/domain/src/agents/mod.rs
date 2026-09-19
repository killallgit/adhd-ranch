//! What each coding agent tells the ranch, and how to ask for it.
//!
//! One module per agent. Each owns the exact shape of that agent's own hook
//! configuration — we do not control these formats, so each module is a
//! transcription of an external contract — and maps that agent's events onto the
//! shared vocabulary in [`hooks`].

pub mod claude_code;
pub mod hooks;
