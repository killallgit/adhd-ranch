//! The hook socket where there is no such thing.
//!
//! Nothing installs hooks on these platforms, so nothing would ever connect. A
//! server that hears nothing is the honest shape of that, and it is what keeps the
//! platform out of every caller.

use std::io;
use std::path::PathBuf;
use std::sync::Arc;

use super::OnChange;
use crate::agent_session_store::HookEventSink;

/// Holds nothing: no socket to close, no thread to join.
pub struct HookServer;

pub fn serve(
    _socket_path: PathBuf,
    _sessions: Arc<dyn HookEventSink>,
    _on_change: OnChange,
) -> io::Result<HookServer> {
    log::info!("agent hooks: no hook socket on this platform");
    Ok(HookServer)
}
