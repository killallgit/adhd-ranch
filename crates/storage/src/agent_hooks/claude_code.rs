use std::io;
use std::path::PathBuf;

use adhd_ranch_domain::agents::claude_code::hooks::{self, HookCommand};
use adhd_ranch_domain::agents::hooks::HookEdit;

use super::settings_file;
use super::{AgentHooks, HookOutcome};

pub struct ClaudeHookPaths {
    /// Claude Code's own settings file, which it shares with every other tool the
    /// user has pointed at it.
    pub settings_file: PathBuf,
    /// The client Claude runs, which ships beside the app.
    pub client_bin: PathBuf,
    /// Where the running ranch is listening.
    pub socket_path: PathBuf,
}

pub struct ClaudeCodeHooks {
    settings_file: PathBuf,
    // Held as the strings they become in the settings file: that text is the whole
    // contract, and uninstall matches it character for character.
    client_bin: String,
    socket_path: String,
}

impl ClaudeCodeHooks {
    pub fn new(paths: ClaudeHookPaths) -> Self {
        Self {
            client_bin: paths.client_bin.to_string_lossy().into_owned(),
            socket_path: paths.socket_path.to_string_lossy().into_owned(),
            settings_file: paths.settings_file,
        }
    }

    fn command(&self) -> HookCommand<'_> {
        HookCommand {
            client_bin: &self.client_bin,
            socket_path: &self.socket_path,
        }
    }

    fn apply(&self, edit: HookEdit) -> io::Result<HookOutcome> {
        match edit {
            HookEdit::Unchanged => Ok(HookOutcome::AlreadyDone),
            HookEdit::Unsupported => Ok(HookOutcome::SettingsNotUnderstood),
            HookEdit::Updated(updated) => {
                settings_file::write(&self.settings_file, &updated)?;
                Ok(HookOutcome::Changed)
            }
        }
    }
}

impl AgentHooks for ClaudeCodeHooks {
    fn agent(&self) -> &'static str {
        "Claude Code"
    }

    fn install(&self) -> io::Result<HookOutcome> {
        let Some(settings) = settings_file::read(&self.settings_file)? else {
            return Ok(HookOutcome::SettingsNotUnderstood);
        };
        self.apply(hooks::install(&settings, &self.command()))
    }

    fn uninstall(&self) -> io::Result<HookOutcome> {
        let Some(settings) = settings_file::read(&self.settings_file)? else {
            return Ok(HookOutcome::SettingsNotUnderstood);
        };
        self.apply(hooks::uninstall(&settings, &self.command()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use adhd_ranch_domain::agents::claude_code::hooks::SESSION_HOOKS;
    use serde_json::Value;
    use std::fs;
    use tempfile::TempDir;

    fn hooks_for(dir: &TempDir) -> ClaudeCodeHooks {
        ClaudeCodeHooks::new(ClaudeHookPaths {
            settings_file: dir.path().join("claude").join("settings.json"),
            client_bin: dir.path().join("bin").join("adhd-ranch-hook"),
            socket_path: dir.path().join("agent-hooks.sock"),
        })
    }

    fn registered_events(hooks: &ClaudeCodeHooks) -> Vec<String> {
        let raw = fs::read_to_string(&hooks.settings_file).unwrap();
        serde_json::from_str::<Value>(&raw).unwrap()["hooks"]
            .as_object()
            .map(|events| events.keys().cloned().collect())
            .unwrap_or_default()
    }

    #[test]
    fn install_registers_every_hook() {
        let dir = TempDir::new().unwrap();
        let hooks = hooks_for(&dir);

        assert_eq!(hooks.install().unwrap(), HookOutcome::Changed);

        let registered = registered_events(&hooks);
        for hook in SESSION_HOOKS {
            assert!(registered.contains(&hook.event.to_string()));
        }
    }

    /// Installing is an edit to one file and nothing else. The socket and the client
    /// belong to the running app, and an installer that made them would be making
    /// promises about a ranch that may not even be running.
    #[test]
    fn install_writes_nothing_but_the_settings_file() {
        let dir = TempDir::new().unwrap();
        let hooks = hooks_for(&dir);

        hooks.install().unwrap();

        assert!(!dir.path().join("agent-hooks.sock").exists());
        assert!(!dir.path().join("bin").exists());
    }

    #[test]
    fn a_second_install_leaves_the_file_alone() {
        let dir = TempDir::new().unwrap();
        let hooks = hooks_for(&dir);
        hooks.install().unwrap();

        assert_eq!(hooks.install().unwrap(), HookOutcome::AlreadyDone);
    }

    #[test]
    fn uninstall_leaves_no_trace() {
        let dir = TempDir::new().unwrap();
        let hooks = hooks_for(&dir);
        hooks.install().unwrap();

        assert_eq!(hooks.uninstall().unwrap(), HookOutcome::Changed);

        assert!(registered_events(&hooks).is_empty());
    }

    #[test]
    fn uninstall_before_install_is_not_an_error() {
        let dir = TempDir::new().unwrap();
        let hooks = hooks_for(&dir);

        assert_eq!(hooks.uninstall().unwrap(), HookOutcome::AlreadyDone);
    }

    #[test]
    fn malformed_settings_are_never_rewritten() {
        let dir = TempDir::new().unwrap();
        let hooks = hooks_for(&dir);
        fs::create_dir_all(hooks.settings_file.parent().unwrap()).unwrap();
        fs::write(&hooks.settings_file, "{ oops").unwrap();

        assert_eq!(hooks.install().unwrap(), HookOutcome::SettingsNotUnderstood);

        assert_eq!(fs::read_to_string(&hooks.settings_file).unwrap(), "{ oops");
    }
}
