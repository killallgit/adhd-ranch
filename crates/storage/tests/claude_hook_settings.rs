//! A settings file the ranch shares with other tools, taken through a real install
//! and uninstall on disk. The unit tests work on parsed documents; this is the whole
//! round trip, because the file that got damaged was a real one with other people's
//! hooks in it.
#![cfg(unix)]

use std::fs;
use std::path::PathBuf;

use adhd_ranch_storage::{AgentHooks, ClaudeCodeHooks, ClaudeHookPaths, HookOutcome};
use serde_json::{json, Value};
use tempfile::TempDir;

/// Shaped after a real `~/.claude/settings.json`: two unrelated tools already own
/// entries, one of them on an event the ranch also wants.
fn other_tools() -> Value {
    json!({
        "hooks": {
            "PreToolUse": [{
                "matcher": "Bash",
                "hooks": [{ "type": "command", "command": "rtk hook claude" }]
            }],
            "SessionStart": [{
                "hooks": [{ "type": "command", "command": "\"/w/hooks/wiki-session-start.sh\"", "timeout": 5 }]
            }],
            "Stop": [{
                "hooks": [{ "type": "command", "command": "python3 \"/w/hooks/wiki-stop.py\"", "timeout": 5 }]
            }]
        },
        "theme": "dark",
        "autoCompactWindow": 500000
    })
}

struct Ranch {
    _root: TempDir,
    hooks: ClaudeCodeHooks,
    settings_file: PathBuf,
}

impl Ranch {
    fn new(initial: &Value) -> Self {
        let root = TempDir::new().unwrap();
        let settings_file = root.path().join("claude").join("settings.json");
        fs::create_dir_all(settings_file.parent().unwrap()).unwrap();
        fs::write(
            &settings_file,
            serde_json::to_string_pretty(initial).unwrap(),
        )
        .unwrap();
        let hooks = ClaudeCodeHooks::new(ClaudeHookPaths {
            settings_file: settings_file.clone(),
            client_bin: root.path().join("bin/adhd-ranch-hook"),
            socket_path: root.path().join("agent-hooks.sock"),
        });
        Self {
            _root: root,
            hooks,
            settings_file,
        }
    }

    fn settings(&self) -> Value {
        serde_json::from_str(&fs::read_to_string(&self.settings_file).unwrap()).unwrap()
    }

    fn commands(&self, event: &str) -> Vec<String> {
        self.settings()["hooks"][event]
            .as_array()
            .map(|groups| {
                groups
                    .iter()
                    .filter_map(|g| g.get("hooks").and_then(Value::as_array))
                    .flatten()
                    .filter_map(|h| h.get("command").and_then(Value::as_str))
                    .map(str::to_string)
                    .collect()
            })
            .unwrap_or_default()
    }
}

#[test]
fn install_adds_the_ranch_beside_other_tools_on_a_shared_event() {
    let ranch = Ranch::new(&other_tools());

    assert_eq!(ranch.hooks.install().unwrap(), HookOutcome::Changed);

    let session_start = ranch.commands("SessionStart");
    assert!(session_start.contains(&"\"/w/hooks/wiki-session-start.sh\"".to_string()));
    assert_eq!(session_start.len(), 2);
}

#[test]
fn install_never_touches_a_tool_it_does_not_own() {
    let ranch = Ranch::new(&other_tools());

    ranch.hooks.install().unwrap();

    assert_eq!(ranch.commands("PreToolUse"), vec!["rtk hook claude"]);
    assert_eq!(ranch.settings()["theme"], json!("dark"));
    assert_eq!(ranch.settings()["autoCompactWindow"], json!(500_000));
}

#[test]
fn uninstall_returns_the_file_to_exactly_what_it_was() {
    let before = other_tools();
    let ranch = Ranch::new(&before);
    ranch.hooks.install().unwrap();

    ranch.hooks.uninstall().unwrap();

    assert_eq!(ranch.settings(), before);
}

#[test]
fn relaunching_the_app_rewrites_nothing() {
    let ranch = Ranch::new(&other_tools());
    ranch.hooks.install().unwrap();
    let after_first = fs::read_to_string(&ranch.settings_file).unwrap();

    ranch.hooks.install().unwrap();

    assert_eq!(
        fs::read_to_string(&ranch.settings_file).unwrap(),
        after_first
    );
}
