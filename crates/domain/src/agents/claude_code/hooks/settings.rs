use serde_json::{json, Map, Value};

use crate::agents::hooks::HookEdit;

use super::command::{HookCommand, HOOK_TIMEOUT_SECS};
use super::events::SESSION_HOOKS;

const HOOKS_KEY: &str = "hooks";

/// Register any of this version's hooks the file does not already have.
pub fn install(settings: &Value, command: &HookCommand) -> HookEdit {
    let mut updated = settings.clone();
    let Some(root) = updated.as_object_mut() else {
        return HookEdit::Unsupported;
    };
    let Some(hooks) = root
        .entry(HOOKS_KEY)
        .or_insert_with(|| json!({}))
        .as_object_mut()
    else {
        return HookEdit::Unsupported;
    };
    if add_missing(hooks, command).is_none() {
        return HookEdit::Unsupported;
    }
    settled(settings, updated)
}

/// Remove the commands this version installs, and nothing else.
pub fn uninstall(settings: &Value, command: &HookCommand) -> HookEdit {
    let mut updated = settings.clone();
    let Some(root) = updated.as_object_mut() else {
        return HookEdit::Unsupported;
    };
    if !root.contains_key(HOOKS_KEY) {
        return HookEdit::Unchanged;
    }

    let emptied = {
        let Some(hooks) = root.get_mut(HOOKS_KEY).and_then(Value::as_object_mut) else {
            return HookEdit::Unsupported;
        };
        if remove_ours(hooks, &our_commands(command)).is_none() {
            return HookEdit::Unsupported;
        }
        hooks.is_empty()
    };
    if emptied {
        root.remove(HOOKS_KEY);
    }
    settled(settings, updated)
}

/// The exact commands this version writes. An entry counts as the ranch's only if
/// its command is one of these, character for character.
fn our_commands(command: &HookCommand) -> Vec<String> {
    SESSION_HOOKS
        .iter()
        .map(|hook| command.for_action(hook.action))
        .collect()
}

fn add_missing(hooks: &mut Map<String, Value>, command: &HookCommand) -> Option<()> {
    for hook in SESSION_HOOKS {
        let command = command.for_action(hook.action);
        let groups = hooks
            .entry(hook.event)
            .or_insert_with(|| json!([]))
            .as_array_mut()?;
        if groups.iter().any(|group| has_command(group, &command)) {
            continue;
        }
        groups.push(json!({
            "hooks": [{
                "type": "command",
                "command": command,
                "timeout": HOOK_TIMEOUT_SECS,
            }]
        }));
    }
    Some(())
}

fn remove_ours(hooks: &mut Map<String, Value>, ours: &[String]) -> Option<()> {
    for event in hooks.keys().cloned().collect::<Vec<_>>() {
        let groups = hooks.get_mut(&event)?.as_array_mut()?;
        for group in groups.iter_mut() {
            if let Some(entries) = group.get_mut(HOOKS_KEY).and_then(Value::as_array_mut) {
                entries.retain(|entry| !is_ours(entry, ours));
            }
        }
        // A group we emptied is ours to drop; one that never had a `hooks` array is
        // not a shape we wrote, so it stays exactly as found.
        groups.retain(
            |group| match group.get(HOOKS_KEY).and_then(Value::as_array) {
                Some(entries) => !entries.is_empty(),
                None => true,
            },
        );
        if groups.is_empty() {
            hooks.remove(&event);
        }
    }
    Some(())
}

fn is_ours(entry: &Value, ours: &[String]) -> bool {
    entry
        .get("command")
        .and_then(Value::as_str)
        .is_some_and(|command| ours.iter().any(|ours| ours == command))
}

fn has_command(group: &Value, command: &str) -> bool {
    group
        .get(HOOKS_KEY)
        .and_then(Value::as_array)
        .is_some_and(|entries| {
            entries
                .iter()
                .any(|entry| entry.get("command").and_then(Value::as_str) == Some(command))
        })
}

fn settled(before: &Value, after: Value) -> HookEdit {
    if after == *before {
        HookEdit::Unchanged
    } else {
        HookEdit::Updated(after)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const COMMAND: HookCommand<'static> = HookCommand {
        client_bin: "/Applications/adhd-ranch.app/Contents/MacOS/adhd-ranch-hook",
        socket_path: "/home/ryan/.adhd-ranch/agent-hooks.sock",
    };

    /// Two other tools already own entries in this file; none of them may move.
    fn settings_with_other_tools() -> Value {
        json!({
            "theme": "dark",
            "hooks": {
                "PreToolUse": [{
                    "matcher": "Bash",
                    "hooks": [{ "type": "command", "command": "rtk hook claude" }]
                }],
                "Stop": [{
                    "hooks": [{ "type": "command", "command": "python3 \"/w/wiki-stop.py\"", "timeout": 5 }]
                }]
            }
        })
    }

    fn commands_for(settings: &Value, event: &str) -> Vec<String> {
        settings["hooks"][event]
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

    fn installed(settings: &Value) -> Value {
        match install(settings, &COMMAND) {
            HookEdit::Updated(updated) => updated,
            other => panic!("expected an update, got {other:?}"),
        }
    }

    #[test]
    fn install_registers_every_hook_in_the_table() {
        let updated = installed(&json!({}));

        for hook in SESSION_HOOKS {
            let expected = COMMAND.for_action(hook.action);
            assert!(commands_for(&updated, hook.event).contains(&expected));
        }
    }

    #[test]
    fn install_registers_no_per_tool_hook() {
        let updated = installed(&json!({}));

        let events = updated["hooks"].as_object().unwrap();
        assert!(!events.contains_key("PreToolUse"));
        assert!(!events.contains_key("PostToolUse"));
    }

    #[test]
    fn a_second_install_changes_nothing() {
        let updated = installed(&json!({}));

        assert_eq!(install(&updated, &COMMAND), HookEdit::Unchanged);
    }

    #[test]
    fn install_leaves_other_tools_untouched() {
        let updated = installed(&settings_with_other_tools());

        assert_eq!(
            commands_for(&updated, "PreToolUse"),
            vec!["rtk hook claude"]
        );
        assert!(commands_for(&updated, "Stop").contains(&"python3 \"/w/wiki-stop.py\"".to_string()));
        assert_eq!(updated["theme"], json!("dark"));
    }

    #[test]
    fn uninstall_restores_the_file_other_tools_were_using() {
        let before = settings_with_other_tools();
        let updated = installed(&before);

        let cleaned = match uninstall(&updated, &COMMAND) {
            HookEdit::Updated(cleaned) => cleaned,
            other => panic!("{other:?}"),
        };

        assert_eq!(cleaned, before);
    }

    #[test]
    fn uninstall_on_a_file_we_never_touched_changes_nothing() {
        let before = settings_with_other_tools();

        assert_eq!(uninstall(&before, &COMMAND), HookEdit::Unchanged);
    }

    /// Uninstall removes the commands this version writes, character for character.
    /// Anything else stays, including an entry naming the same client differently —
    /// there is no guessing about what some other install might have looked like.
    #[test]
    fn uninstall_leaves_a_command_it_did_not_write() {
        let near_miss = COMMAND
            .for_action(SESSION_HOOKS[0].action)
            .replace('\'', "");
        let before = json!({
            "hooks": {
                "SessionStart": [{
                    "hooks": [{ "type": "command", "command": near_miss }]
                }]
            }
        });

        assert_eq!(uninstall(&before, &COMMAND), HookEdit::Unchanged);
    }

    #[test]
    fn a_settings_document_that_is_not_an_object_is_refused() {
        assert_eq!(install(&json!([1, 2]), &COMMAND), HookEdit::Unsupported);
    }

    #[test]
    fn an_event_that_is_not_an_array_is_refused() {
        let odd = json!({ "hooks": { "Stop": "surprise" } });

        assert_eq!(install(&odd, &COMMAND), HookEdit::Unsupported);
    }

    #[test]
    fn a_reordered_event_is_still_recognised_as_installed() {
        let updated = installed(&settings_with_other_tools());
        let mut shuffled = updated.clone();
        shuffled["hooks"]["Stop"].as_array_mut().unwrap().reverse();

        assert_eq!(install(&shuffled, &COMMAND), HookEdit::Unchanged);
    }
}
