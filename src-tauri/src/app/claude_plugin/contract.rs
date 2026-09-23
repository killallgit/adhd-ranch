//! The complete Ranch hook contract, checked against both the published source
//! and Claude's installed copy. Claude's schema validation alone cannot tell us
//! whether every event invokes the right Ranch action.

use std::fs;
use std::io;
use std::path::Path;

use adhd_ranch_domain::agents::claude_code::hooks::SESSION_HOOKS;
use serde_json::{json, Value};

const HOOK_COMMAND: &str = "${CLAUDE_PLUGIN_ROOT}/scripts/send-hook";
const WRAPPER: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../plugins/adhd-ranch-hooks/scripts/send-hook"
));

fn invalid(message: impl Into<String>) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message.into())
}

fn read_required(root: &Path, relative: &str) -> io::Result<Vec<u8>> {
    fs::read(root.join(relative)).map_err(|error| {
        io::Error::new(
            error.kind(),
            format!("Cannot read Ranch plugin {relative}: {error}"),
        )
    })
}

pub(super) fn validate(root: &Path) -> io::Result<()> {
    let manifest: Value =
        serde_json::from_slice(&read_required(root, ".claude-plugin/plugin.json")?)?;
    if manifest.get("name").and_then(Value::as_str) != Some("adhd-ranch-hooks") {
        return Err(invalid("Ranch plugin manifest has the wrong name"));
    }

    let document: Value = serde_json::from_slice(&read_required(root, "hooks/hooks.json")?)?;
    if let Some(unexpected) = document
        .as_object()
        .ok_or_else(|| invalid("Ranch hooks file is not an object"))?
        .keys()
        .find(|key| !matches!(key.as_str(), "hooks" | "description" | "$schema"))
    {
        return Err(invalid(format!(
            "Ranch hooks file has unexpected field {unexpected}"
        )));
    }
    let hooks = document
        .get("hooks")
        .and_then(Value::as_object)
        .ok_or_else(|| invalid("Ranch plugin is missing its hooks object"))?;
    for hook in SESSION_HOOKS {
        let expected = json!([{
            "hooks": [{
                "type": "command",
                "command": HOOK_COMMAND,
                "args": [hook.action.verb()],
                "timeout": 5
            }]
        }]);
        match hooks.get(hook.event) {
            None => return Err(invalid(format!("Ranch plugin is missing {}", hook.event))),
            Some(actual) if actual != &expected => {
                return Err(invalid(format!(
                    "Ranch plugin's {} handler does not invoke send-hook {}",
                    hook.event,
                    hook.action.verb()
                )))
            }
            Some(_) => {}
        }
    }
    if let Some(unexpected) = hooks.keys().find(|event| {
        !SESSION_HOOKS
            .iter()
            .any(|hook| hook.event == event.as_str())
    }) {
        return Err(invalid(format!(
            "Ranch plugin has unexpected hook event {unexpected}"
        )));
    }

    let script = root.join("scripts/send-hook");
    if read_required(root, "scripts/send-hook")? != WRAPPER {
        return Err(invalid(
            "Ranch plugin's send-hook wrapper differs from this app",
        ));
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if fs::metadata(&script)?.permissions().mode() & 0o111 == 0 {
            return Err(invalid(
                "Ranch plugin's send-hook wrapper is not executable",
            ));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture() -> tempfile::TempDir {
        let dir = tempfile::tempdir().unwrap();
        fs::create_dir_all(dir.path().join(".claude-plugin")).unwrap();
        fs::create_dir_all(dir.path().join("hooks")).unwrap();
        fs::create_dir_all(dir.path().join("scripts")).unwrap();
        fs::write(
            dir.path().join(".claude-plugin/plugin.json"),
            include_bytes!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/../plugins/adhd-ranch-hooks/.claude-plugin/plugin.json"
            )),
        )
        .unwrap();
        fs::write(
            dir.path().join("hooks/hooks.json"),
            include_bytes!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/../plugins/adhd-ranch-hooks/hooks/hooks.json"
            )),
        )
        .unwrap();
        let script = dir.path().join("scripts/send-hook");
        fs::write(&script, WRAPPER).unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&script, fs::Permissions::from_mode(0o755)).unwrap();
        }
        dir
    }

    #[test]
    fn published_plugin_has_the_complete_contract() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../plugins/adhd-ranch-hooks");
        validate(&root).unwrap();
        let manifest: Value =
            serde_json::from_slice(&fs::read(root.join(".claude-plugin/plugin.json")).unwrap())
                .unwrap();
        assert_eq!(manifest["version"], "0.1.1");
    }

    #[test]
    fn marketplace_points_at_the_validated_plugin() {
        let marketplace: Value = serde_json::from_slice(include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../.claude-plugin/marketplace.json"
        )))
        .unwrap();
        assert_eq!(marketplace["name"], "adhd-ranch");
        assert_eq!(marketplace["plugins"].as_array().unwrap().len(), 1);
        assert_eq!(marketplace["plugins"][0]["name"], "adhd-ranch-hooks");
        assert_eq!(
            marketplace["plugins"][0]["source"],
            "./plugins/adhd-ranch-hooks"
        );
    }

    #[test]
    fn an_existing_complete_install_is_valid() {
        let dir = fixture();
        validate(dir.path()).unwrap();
    }

    #[test]
    fn missing_hook_is_rejected() {
        let dir = fixture();
        let file = dir.path().join("hooks/hooks.json");
        let mut document: Value = serde_json::from_slice(&fs::read(&file).unwrap()).unwrap();
        document["hooks"]
            .as_object_mut()
            .unwrap()
            .remove("StopFailure");
        fs::write(file, serde_json::to_vec(&document).unwrap()).unwrap();
        assert!(validate(dir.path())
            .unwrap_err()
            .to_string()
            .contains("StopFailure"));
    }

    #[test]
    fn misspelled_event_or_action_is_rejected() {
        let dir = fixture();
        let file = dir.path().join("hooks/hooks.json");
        let mut document: Value = serde_json::from_slice(&fs::read(&file).unwrap()).unwrap();
        let stop = document["hooks"]
            .as_object_mut()
            .unwrap()
            .remove("Stop")
            .unwrap();
        document["hooks"]["Stopp"] = stop;
        fs::write(&file, serde_json::to_vec(&document).unwrap()).unwrap();
        assert!(validate(dir.path())
            .unwrap_err()
            .to_string()
            .contains("Stop"));

        let mut document: Value = serde_json::from_slice(include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../plugins/adhd-ranch-hooks/hooks/hooks.json"
        )))
        .unwrap();
        document["hooks"]["Stop"][0]["hooks"][0]["args"][0] = json!("idel");
        fs::write(file, serde_json::to_vec(&document).unwrap()).unwrap();
        assert!(validate(dir.path())
            .unwrap_err()
            .to_string()
            .contains("idle"));
    }

    #[test]
    fn misspelled_hooks_field_is_rejected() {
        let dir = fixture();
        let file = dir.path().join("hooks/hooks.json");
        let mut document: Value = serde_json::from_slice(&fs::read(&file).unwrap()).unwrap();
        document["hookz"] = json!({});
        fs::write(file, serde_json::to_vec(&document).unwrap()).unwrap();
        assert!(validate(dir.path())
            .unwrap_err()
            .to_string()
            .contains("hookz"));
    }

    #[test]
    fn misplaced_hooks_or_wrapper_are_rejected() {
        let dir = fixture();
        fs::rename(
            dir.path().join("hooks"),
            dir.path().join(".claude-plugin/hooks"),
        )
        .unwrap();
        assert!(validate(dir.path())
            .unwrap_err()
            .to_string()
            .contains("hooks/hooks.json"));

        let dir = fixture();
        fs::rename(
            dir.path().join("scripts/send-hook"),
            dir.path().join("hooks/send-hook"),
        )
        .unwrap();
        assert!(validate(dir.path()).is_err());
    }

    #[test]
    fn altered_or_nonexecutable_wrapper_is_rejected() {
        let dir = fixture();
        let script = dir.path().join("scripts/send-hook");
        fs::write(&script, b"#!/bin/sh\nexit 0\n").unwrap();
        assert!(validate(dir.path()).is_err());

        #[cfg(unix)]
        {
            fs::write(&script, WRAPPER).unwrap();
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&script, fs::Permissions::from_mode(0o644)).unwrap();
            assert!(validate(dir.path()).is_err());
        }
    }
}
