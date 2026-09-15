use std::path::Path;

use adhd_ranch_storage::{install_claude_session_hook, ClaudeHookInstall, ClaudeHookPaths};

use super::paths;

pub fn register(sessions_dir: &Path) {
    // The hook is a POSIX shell script; Claude Code on Windows can't run it.
    if cfg!(windows) {
        log::info!("claude hook: not registered on Windows");
        return;
    }

    let hook_paths = match (paths::claude_settings_file(), paths::claude_hook_script()) {
        (Ok(settings_file), Ok(script_file)) => ClaudeHookPaths {
            settings_file,
            script_file,
            sessions_dir: sessions_dir.to_path_buf(),
        },
        (Err(e), _) | (_, Err(e)) => {
            log::error!("claude hook: cannot resolve paths: {e}");
            return;
        }
    };

    match install_claude_session_hook(&hook_paths) {
        Ok(ClaudeHookInstall::Registered) => log::info!(
            "claude hook: registered in {}",
            hook_paths.settings_file.display()
        ),
        Ok(ClaudeHookInstall::AlreadyRegistered) => {
            log::info!("claude hook: already registered")
        }
        Ok(ClaudeHookInstall::SettingsNotUnderstood) => log::warn!(
            "claude hook: {} is not valid settings JSON; left untouched",
            hook_paths.settings_file.display()
        ),
        Err(e) => log::error!("claude hook: install failed: {e}"),
    }
}
