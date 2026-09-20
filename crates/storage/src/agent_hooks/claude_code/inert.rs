//! Claude Code's hooks where the client they name has nowhere to deliver.
//!
//! The entry the ranch would write names a client that speaks over a Unix socket.
//! Writing it on a platform with no such socket would leave a command in someone's
//! settings file that can only ever fail, so nothing goes in and nothing has to be
//! taken back out — which is what keeps the platform out of every caller.

use std::io;

use super::{ClaudeHookPaths, AGENT};
use crate::agent_hooks::{AgentHooks, HookOutcome};

/// Holds nothing: no settings file to edit, no command to match it against.
pub struct ClaudeCodeHooks;

impl ClaudeCodeHooks {
    pub fn new(_paths: ClaudeHookPaths) -> Self {
        Self
    }
}

impl AgentHooks for ClaudeCodeHooks {
    fn agent(&self) -> &'static str {
        AGENT
    }

    fn install(&self) -> io::Result<HookOutcome> {
        log::info!("{AGENT} hooks: no hook socket on this platform; nothing to install");
        Ok(HookOutcome::AlreadyDone)
    }

    fn uninstall(&self) -> io::Result<HookOutcome> {
        log::info!("{AGENT} hooks: no hook socket on this platform; nothing to uninstall");
        Ok(HookOutcome::AlreadyDone)
    }

    /// Nothing here ever installed anything, whatever a settings file synced from
    /// another machine may say about a socket this one cannot open.
    fn installed(&self) -> io::Result<bool> {
        Ok(false)
    }
}
