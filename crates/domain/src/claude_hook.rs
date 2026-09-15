use std::path::Path;

use serde_json::{json, Value};

use crate::animal::Animal;

const HOOK_TIMEOUT_SECS: u64 = 5;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionHookAction {
    Start,
    End,
}

impl SessionHookAction {
    const ALL: [Self; 2] = [Self::Start, Self::End];

    fn event_name(self) -> &'static str {
        match self {
            Self::Start => "SessionStart",
            Self::End => "SessionEnd",
        }
    }

    fn arg(self) -> &'static str {
        match self {
            Self::Start => "start",
            Self::End => "end",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum HookRegistration {
    Unchanged,
    Updated(Value),
    UnsupportedSettings,
}

pub fn animal_from_session_start(payload: &str) -> Option<Animal> {
    let value: Value = serde_json::from_str(payload).ok()?;
    let id = value.get("session_id")?.as_str()?.trim();
    if id.is_empty() {
        return None;
    }
    let name = value
        .get("cwd")
        .and_then(Value::as_str)
        .and_then(|cwd| Path::new(cwd).file_name())
        .and_then(|name| name.to_str())
        .unwrap_or(id);
    Some(Animal {
        id: id.to_string(),
        name: name.to_string(),
    })
}

pub fn session_hook_command(script_path: &str, action: SessionHookAction) -> String {
    format!("sh {} {}", shell_quote(script_path), action.arg())
}

// The session id becomes a file name, so the script only accepts ids made of
// letters, digits and dashes; anything else could escape the sessions dir.
pub fn session_hook_script(sessions_dir: &str) -> String {
    format!(
        r#"#!/bin/sh
# Installed by adhd-ranch. The app rewrites this file on every launch.
# Keeps one file per running Claude Code session so the ranch can show an animal for it.

sessions_dir={dir}
payload=$(cat)
session_id=$(printf '%s\n' "$payload" | sed -n 's/.*"session_id"[[:space:]]*:[[:space:]]*"\([0-9A-Za-z-]\{{1,\}}\)".*/\1/p' | head -n 1)
[ -n "$session_id" ] || exit 0

case "${{1:-}}" in
  start)
    mkdir -p "$sessions_dir" 2>/dev/null || exit 0
    tmp="$sessions_dir/.$session_id.json.tmp"
    printf '%s\n' "$payload" > "$tmp" 2>/dev/null && mv -f "$tmp" "$sessions_dir/$session_id.json" 2>/dev/null
    ;;
  end)
    rm -f "$sessions_dir/$session_id.json" 2>/dev/null
    ;;
esac
exit 0
"#,
        dir = shell_quote(sessions_dir)
    )
}

pub fn register_session_hooks(settings: &Value, script_path: &str) -> HookRegistration {
    let mut updated = settings.clone();
    let Some(root) = updated.as_object_mut() else {
        return HookRegistration::UnsupportedSettings;
    };
    let Some(hooks) = root
        .entry("hooks")
        .or_insert_with(|| json!({}))
        .as_object_mut()
    else {
        return HookRegistration::UnsupportedSettings;
    };

    let mut changed = false;
    for action in SessionHookAction::ALL {
        let command = session_hook_command(script_path, action);
        let Some(groups) = hooks
            .entry(action.event_name())
            .or_insert_with(|| json!([]))
            .as_array_mut()
        else {
            return HookRegistration::UnsupportedSettings;
        };
        if !groups_contain_command(groups, &command) {
            groups.push(json!({
                "hooks": [{
                    "type": "command",
                    "command": command,
                    "timeout": HOOK_TIMEOUT_SECS,
                }]
            }));
            changed = true;
        }
    }

    if changed {
        HookRegistration::Updated(updated)
    } else {
        HookRegistration::Unchanged
    }
}

fn groups_contain_command(groups: &[Value], command: &str) -> bool {
    groups
        .iter()
        .filter_map(|group| group.get("hooks").and_then(Value::as_array))
        .flatten()
        .any(|hook| hook.get("command").and_then(Value::as_str) == Some(command))
}

fn shell_quote(raw: &str) -> String {
    format!("'{}'", raw.replace('\'', r"'\''"))
}

#[cfg(test)]
mod tests {
    use super::*;

    const SCRIPT: &str = "/Users/me/.adhd-ranch/hooks/claude-session.sh";

    fn commands_for(settings: &Value, event: &str) -> Vec<String> {
        settings["hooks"][event]
            .as_array()
            .unwrap()
            .iter()
            .flat_map(|group| group["hooks"].as_array().unwrap().clone())
            .map(|hook| hook["command"].as_str().unwrap().to_string())
            .collect()
    }

    fn updated(registration: HookRegistration) -> Value {
        match registration {
            HookRegistration::Updated(value) => value,
            other => panic!("expected Updated, got {other:?}"),
        }
    }

    #[test]
    fn animal_is_named_after_the_session_working_directory() {
        let animal =
            animal_from_session_start(r#"{"session_id":"abc-123","cwd":"/code/adhd-ranch"}"#)
                .unwrap();

        assert_eq!(
            animal,
            Animal {
                id: "abc-123".into(),
                name: "adhd-ranch".into()
            }
        );
    }

    #[test]
    fn animal_without_cwd_is_named_after_its_session_id() {
        let animal = animal_from_session_start(r#"{"session_id":"abc-123"}"#).unwrap();

        assert_eq!(animal.name, "abc-123");
    }

    #[test]
    fn payload_without_session_id_has_no_animal() {
        assert_eq!(animal_from_session_start(r#"{"cwd":"/code"}"#), None);
    }

    #[test]
    fn malformed_payload_has_no_animal() {
        assert_eq!(animal_from_session_start("{not json"), None);
    }

    #[test]
    fn registers_start_and_end_hooks_in_empty_settings() {
        let settings = updated(register_session_hooks(&json!({}), SCRIPT));

        assert_eq!(
            commands_for(&settings, "SessionStart"),
            vec![session_hook_command(SCRIPT, SessionHookAction::Start)]
        );
        assert_eq!(
            commands_for(&settings, "SessionEnd"),
            vec![session_hook_command(SCRIPT, SessionHookAction::End)]
        );
    }

    #[test]
    fn registration_keeps_existing_hooks_for_the_same_event() {
        let existing = json!({
            "hooks": {"SessionStart": [{"hooks": [{"type": "command", "command": "other"}]}]}
        });

        let settings = updated(register_session_hooks(&existing, SCRIPT));

        assert_eq!(
            commands_for(&settings, "SessionStart"),
            vec![
                "other".to_string(),
                session_hook_command(SCRIPT, SessionHookAction::Start)
            ]
        );
    }

    #[test]
    fn registration_keeps_unrelated_settings() {
        let existing = json!({"theme": "dark", "permissions": {"allow": ["Bash"]}});

        let settings = updated(register_session_hooks(&existing, SCRIPT));

        assert_eq!(settings["theme"], "dark");
        assert_eq!(settings["permissions"], existing["permissions"]);
    }

    #[test]
    fn registering_twice_leaves_settings_unchanged() {
        let once = updated(register_session_hooks(&json!({}), SCRIPT));

        assert_eq!(
            register_session_hooks(&once, SCRIPT),
            HookRegistration::Unchanged
        );
    }

    #[test]
    fn settings_with_non_object_hooks_are_unsupported() {
        assert_eq!(
            register_session_hooks(&json!({"hooks": []}), SCRIPT),
            HookRegistration::UnsupportedSettings
        );
    }

    #[test]
    fn hook_command_quotes_the_script_path() {
        assert_eq!(
            session_hook_command("/a b/it's.sh", SessionHookAction::End),
            r"sh '/a b/it'\''s.sh' end"
        );
    }
}
