use adhd_ranch_storage::{AgentHooks, ClaudeCodeHooks, ClaudeHookPaths, HookOutcome};

use super::paths;

/// Bring Claude Code's hooks in line with the `agents.enabled` setting.
///
/// Turning agents off actively undoes the install rather than just ignoring it. The
/// entries live in a settings file the ranch does not own and shares with whatever
/// else the user has pointed at it, so leaving them behind would keep firing the
/// client on every session on the machine with nothing left listening for it.
///
/// Failures come back rather than only reaching the log: switching agents on is a
/// request that can fail, and calling it done when the hooks are not installed
/// leaves the user waiting for animals that will never arrive.
pub fn reconcile(enabled: bool) -> Result<(), String> {
    let hooks = ClaudeCodeHooks::new(
        hook_paths().map_err(|e| format!("claude hook: cannot resolve paths: {e}"))?,
    );
    let verb = if enabled { "install" } else { "uninstall" };
    let outcome = if enabled {
        hooks.install()
    } else {
        hooks.uninstall()
    }
    .map_err(|e| format!("{} hooks: {verb} failed: {e}", hooks.agent()))?;

    match outcome {
        HookOutcome::Changed => {
            log::info!("{} hooks: {verb}ed", hooks.agent());
            Ok(())
        }
        HookOutcome::AlreadyDone => {
            log::info!("{} hooks: nothing to {verb}", hooks.agent());
            Ok(())
        }
        HookOutcome::SettingsNotUnderstood => Err(format!(
            "{} hooks: settings file is not JSON we understand; left untouched",
            hooks.agent()
        )),
    }
}

pub(super) fn hook_paths() -> std::io::Result<ClaudeHookPaths> {
    Ok(ClaudeHookPaths {
        settings_file: paths::claude_settings_file()?,
        client_bin: paths::hook_client_bin()?,
        socket_path: paths::agent_hook_socket()?,
    })
}
