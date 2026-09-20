//! The socket agents' hooks deliver to.
//!
//! Only this module knows that socket is a Unix one. Everything above it gets the
//! same `serve` and `HookServer` on every platform, so no caller has to gate itself
//! — the same bargain the hook client already makes with its own delivery.

use std::sync::Arc;

#[cfg_attr(unix, path = "unix.rs")]
#[cfg_attr(not(unix), path = "inert.rs")]
mod imp;

pub use imp::{serve, HookServer};

/// Called whenever a firing actually changed what the ranch would draw.
pub type OnChange = Arc<dyn Fn() + Send + Sync>;
