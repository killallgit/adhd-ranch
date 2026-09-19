use adhd_ranch_storage::{AgentHooks, ClaudeCodeHooks, ClaudeHookPaths, HookOutcome};

use super::paths;

/// Bring Claude Code's hooks in line with the `agents.enabled` setting.
///
/// Turning agents off actively undoes the install rather than just ignoring it. The
/// entries live in a settings file the ranch does not own and shares with whatever
/// else the user has pointed at it, so leaving them behind would keep firing the
/// client on every session on the machine with nothing left listening for it.
pub fn reconcile(enabled: bool) {
    // The client delivers over a Unix socket, which Windows has no equivalent of here.
    if cfg!(windows) {
        log::info!("claude hook: not supported on Windows");
        return;
    }

    let resolved = paths::claude_settings_file()
        .and_then(|settings_file| Ok((settings_file, paths::hook_client_bin()?)))
        .and_then(|(settings_file, client_bin)| {
            Ok(ClaudeHookPaths {
                settings_file,
                client_bin,
                socket_path: paths::agent_hook_socket()?,
            })
        });
    let hooks = match resolved {
        Ok(paths) => ClaudeCodeHooks::new(paths),
        Err(e) => {
            log::error!("claude hook: cannot resolve paths: {e}");
            return;
        }
    };

    let verb = if enabled { "install" } else { "uninstall" };
    let result = if enabled {
        hooks.install()
    } else {
        hooks.uninstall()
    };

    match result {
        Ok(HookOutcome::Changed) => log::info!("{} hooks: {verb}ed", hooks.agent()),
        Ok(HookOutcome::AlreadyDone) => {
            log::info!("{} hooks: nothing to {verb}", hooks.agent())
        }
        Ok(HookOutcome::SettingsNotUnderstood) => log::warn!(
            "{} hooks: settings file is not JSON we understand; left untouched",
            hooks.agent()
        ),
        Err(e) => log::error!("{} hooks: {verb} failed: {e}", hooks.agent()),
    }
}
