use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use adhd_ranch_domain::claude_hook::{
    register_session_hooks, session_hook_script, HookRegistration,
};
use serde_json::{Map, Value};

use crate::atomic::{atomic_write, tmp_path};

pub struct ClaudeHookPaths {
    pub settings_file: PathBuf,
    pub script_file: PathBuf,
    pub sessions_dir: PathBuf,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClaudeHookInstall {
    AlreadyRegistered,
    Registered,
    SettingsNotUnderstood,
}

pub fn install_claude_session_hook(paths: &ClaudeHookPaths) -> io::Result<ClaudeHookInstall> {
    fs::create_dir_all(&paths.sessions_dir)?;
    let script = session_hook_script(&paths.sessions_dir.to_string_lossy());
    atomic_write(&paths.script_file, script.as_bytes())?;

    let Some(settings) = read_settings(&paths.settings_file)? else {
        return Ok(ClaudeHookInstall::SettingsNotUnderstood);
    };
    match register_session_hooks(&settings, &paths.script_file.to_string_lossy()) {
        HookRegistration::Unchanged => Ok(ClaudeHookInstall::AlreadyRegistered),
        HookRegistration::UnsupportedSettings => Ok(ClaudeHookInstall::SettingsNotUnderstood),
        HookRegistration::Updated(updated) => {
            let mut content = serde_json::to_string_pretty(&updated)
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
            content.push('\n');
            replace_settings_file(&paths.settings_file, content.as_bytes())?;
            Ok(ClaudeHookInstall::Registered)
        }
    }
}

fn read_settings(path: &Path) -> io::Result<Option<Value>> {
    match fs::read_to_string(path) {
        Ok(raw) => Ok(serde_json::from_str(&raw).ok()),
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            Ok(Some(Value::Object(Map::new())))
        }
        Err(error) => Err(error),
    }
}

// Claude's settings file can hold secrets and is often a dotfiles symlink, so
// write through the link and give the new file the old file's permissions
// before any content lands in it.
fn replace_settings_file(path: &Path, content: &[u8]) -> io::Result<()> {
    let target = match fs::canonicalize(path) {
        Ok(real) => real,
        Err(error) if error.kind() == io::ErrorKind::NotFound => path.to_path_buf(),
        Err(error) => return Err(error),
    };
    let Ok(metadata) = fs::metadata(&target) else {
        return atomic_write(&target, content);
    };

    let tmp = tmp_path(&target);
    let result = (|| {
        let mut file = OpenOptions::new()
            .create(true)
            .truncate(true)
            .write(true)
            .open(&tmp)?;
        fs::set_permissions(&tmp, metadata.permissions())?;
        file.write_all(content)?;
        file.sync_data()?;
        fs::rename(&tmp, &target)
    })();
    if result.is_err() {
        let _ = fs::remove_file(&tmp);
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn paths(dir: &TempDir) -> ClaudeHookPaths {
        ClaudeHookPaths {
            settings_file: dir.path().join("claude").join("settings.json"),
            script_file: dir
                .path()
                .join("ranch")
                .join("hooks")
                .join("claude-session.sh"),
            sessions_dir: dir.path().join("ranch").join("sessions").join("claude"),
        }
    }

    fn session_start_commands(settings_file: &Path) -> Vec<String> {
        let settings: Value =
            serde_json::from_str(&fs::read_to_string(settings_file).unwrap()).unwrap();
        settings["hooks"]["SessionStart"]
            .as_array()
            .unwrap()
            .iter()
            .flat_map(|group| group["hooks"].as_array().unwrap().clone())
            .map(|hook| hook["command"].as_str().unwrap().to_string())
            .collect()
    }

    #[test]
    fn creates_settings_file_with_hooks_when_missing() {
        let dir = TempDir::new().unwrap();
        let paths = paths(&dir);

        let outcome = install_claude_session_hook(&paths).unwrap();

        assert_eq!(outcome, ClaudeHookInstall::Registered);
        assert_eq!(session_start_commands(&paths.settings_file).len(), 1);
    }

    #[test]
    fn second_install_leaves_settings_alone() {
        let dir = TempDir::new().unwrap();
        let paths = paths(&dir);
        install_claude_session_hook(&paths).unwrap();

        let outcome = install_claude_session_hook(&paths).unwrap();

        assert_eq!(outcome, ClaudeHookInstall::AlreadyRegistered);
        assert_eq!(session_start_commands(&paths.settings_file).len(), 1);
    }

    #[test]
    fn malformed_settings_are_not_rewritten() {
        let dir = TempDir::new().unwrap();
        let paths = paths(&dir);
        fs::create_dir_all(paths.settings_file.parent().unwrap()).unwrap();
        fs::write(&paths.settings_file, "{ oops").unwrap();

        let outcome = install_claude_session_hook(&paths).unwrap();

        assert_eq!(outcome, ClaudeHookInstall::SettingsNotUnderstood);
        assert_eq!(fs::read_to_string(&paths.settings_file).unwrap(), "{ oops");
    }

    #[test]
    fn writes_the_hook_script() {
        let dir = TempDir::new().unwrap();
        let paths = paths(&dir);

        install_claude_session_hook(&paths).unwrap();

        assert!(paths.script_file.is_file());
    }

    #[cfg(unix)]
    #[test]
    fn keeps_settings_file_permissions() {
        use std::os::unix::fs::PermissionsExt;

        let dir = TempDir::new().unwrap();
        let paths = paths(&dir);
        fs::create_dir_all(paths.settings_file.parent().unwrap()).unwrap();
        fs::write(&paths.settings_file, "{}").unwrap();
        fs::set_permissions(&paths.settings_file, fs::Permissions::from_mode(0o600)).unwrap();

        install_claude_session_hook(&paths).unwrap();

        let mode = fs::metadata(&paths.settings_file)
            .unwrap()
            .permissions()
            .mode();
        assert_eq!(mode & 0o777, 0o600);
    }

    #[cfg(unix)]
    #[test]
    fn writes_through_a_symlinked_settings_file() {
        let dir = TempDir::new().unwrap();
        let paths = paths(&dir);
        let real = dir.path().join("dotfiles-settings.json");
        fs::write(&real, "{}").unwrap();
        fs::create_dir_all(paths.settings_file.parent().unwrap()).unwrap();
        std::os::unix::fs::symlink(&real, &paths.settings_file).unwrap();

        install_claude_session_hook(&paths).unwrap();

        assert!(fs::symlink_metadata(&paths.settings_file)
            .unwrap()
            .file_type()
            .is_symlink());
        assert_eq!(session_start_commands(&real).len(), 1);
    }

    #[cfg(unix)]
    mod script {
        use super::*;
        use std::process::{Command, Stdio};

        fn run_script(paths: &ClaudeHookPaths, action: &str, payload: &str) {
            let mut child = Command::new("sh")
                .arg(&paths.script_file)
                .arg(action)
                .stdin(Stdio::piped())
                .spawn()
                .unwrap();
            child
                .stdin
                .take()
                .unwrap()
                .write_all(payload.as_bytes())
                .unwrap();
            assert!(child.wait().unwrap().success());
        }

        #[test]
        fn start_records_the_session_payload() {
            let dir = TempDir::new().unwrap();
            let paths = paths(&dir);
            install_claude_session_hook(&paths).unwrap();
            let payload = r#"{"session_id":"4af8005a-7a52","cwd":"/code/app","hook_event_name":"SessionStart"}"#;

            run_script(&paths, "start", payload);

            let recorded =
                fs::read_to_string(paths.sessions_dir.join("4af8005a-7a52.json")).unwrap();
            assert_eq!(recorded.trim_end(), payload);
        }

        #[test]
        fn end_removes_the_session_file() {
            let dir = TempDir::new().unwrap();
            let paths = paths(&dir);
            install_claude_session_hook(&paths).unwrap();
            let payload = r#"{"session_id":"4af8005a-7a52","cwd":"/code/app"}"#;
            run_script(&paths, "start", payload);

            run_script(&paths, "end", payload);

            assert!(!paths.sessions_dir.join("4af8005a-7a52.json").exists());
        }

        #[test]
        fn session_ids_with_path_characters_are_ignored() {
            let dir = TempDir::new().unwrap();
            let paths = paths(&dir);
            install_claude_session_hook(&paths).unwrap();

            run_script(
                &paths,
                "start",
                r#"{"session_id":"../escape","cwd":"/code"}"#,
            );

            assert_eq!(fs::read_dir(&paths.sessions_dir).unwrap().count(), 0);
            assert!(!dir
                .path()
                .join("ranch")
                .join("sessions")
                .join("escape.json")
                .exists());
        }
    }
}
