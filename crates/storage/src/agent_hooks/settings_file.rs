use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::Path;

use adhd_ranch_domain::agents::hooks::HookEdit;
use serde_json::{Map, Value};

use super::HookOutcome;
use crate::atomic::{atomic_write, tmp_path};

/// How many times to rebuild an edit that someone else invalidated before giving up.
/// A tool that is rewriting this file continuously is not one we can merge with.
const MAX_ATTEMPTS: usize = 3;

/// Apply a change to the settings document, abandoning the attempt if anybody else
/// writes to the file while we are deciding what to change.
///
/// The file belongs to the agent and to whatever else the user has pointed at it, so
/// a write built on a stale read would silently drop what they added in between.
/// This takes no lock: it narrows the window from however long parsing and editing
/// take down to the gap between the last check and the rename, which for a file
/// written twice a session is the proportionate trade.
pub fn edit(path: &Path, change: impl Fn(&Value) -> HookEdit) -> io::Result<HookOutcome> {
    for _ in 0..MAX_ATTEMPTS {
        let Some((before, settings)) = read(path)? else {
            return Ok(HookOutcome::SettingsNotUnderstood);
        };
        match change(&settings) {
            HookEdit::Unchanged => return Ok(HookOutcome::AlreadyDone),
            HookEdit::Unsupported => return Ok(HookOutcome::SettingsNotUnderstood),
            HookEdit::Updated(updated) => {
                if write_if_unchanged(path, &before, &updated)? {
                    return Ok(HookOutcome::Changed);
                }
            }
        }
    }
    Err(io::Error::other(
        "settings file kept changing while the ranch was editing it",
    ))
}

/// The document as it is on disk, with the exact text it was read from so a later
/// write can tell whether anyone has been in the file since.
///
/// `Ok(None)` means the file exists but is not JSON we can reason about; the caller
/// must then leave it alone rather than replace it with something we invented.
fn read(path: &Path) -> io::Result<Option<(String, Value)>> {
    match fs::read_to_string(path) {
        Ok(raw) => Ok(serde_json::from_str(&raw).ok().map(|value| (raw, value))),
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            Ok(Some((String::new(), Value::Object(Map::new()))))
        }
        Err(error) => Err(error),
    }
}

/// Reports whether the write happened. It does not when the file no longer holds
/// what the edit was built on.
fn write_if_unchanged(path: &Path, before: &str, settings: &Value) -> io::Result<bool> {
    match read(path)? {
        Some((current, _)) if current == before => {}
        _ => return Ok(false),
    }
    let mut content = serde_json::to_string_pretty(settings)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    content.push('\n');
    replace(path, content.as_bytes())?;
    Ok(true)
}

// An agent's settings file can hold secrets and is often a dotfiles symlink, so
// write through the link and give the new file the old file's permissions before any
// content lands in it.
fn replace(path: &Path, content: &[u8]) -> io::Result<()> {
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
    use std::sync::atomic::{AtomicUsize, Ordering};

    use serde_json::json;
    use tempfile::TempDir;

    use super::*;

    fn settings_file(dir: &TempDir, contents: &str) -> std::path::PathBuf {
        let file = dir.path().join("settings.json");
        fs::write(&file, contents).unwrap();
        file
    }

    /// The edit is rebuilt against whatever the file holds now, so a change that
    /// landed after the first read survives instead of being written over.
    #[test]
    fn an_edit_built_on_a_stale_read_is_retried() {
        let dir = TempDir::new().unwrap();
        let file = settings_file(&dir, "{}\n");
        let attempts = AtomicUsize::new(0);

        let outcome = edit(&file, |settings| {
            // Stands in for another tool writing between our read and our write.
            if attempts.fetch_add(1, Ordering::SeqCst) == 0 {
                fs::write(&file, r#"{"theirs":1}"#).unwrap();
            }
            let mut updated = settings.clone();
            updated["ours"] = json!(1);
            HookEdit::Updated(updated)
        })
        .unwrap();

        assert_eq!(outcome, HookOutcome::Changed);
        let written: Value = serde_json::from_str(&fs::read_to_string(&file).unwrap()).unwrap();
        assert_eq!(written["theirs"], json!(1));
        assert_eq!(written["ours"], json!(1));
    }

    #[test]
    fn a_file_that_never_settles_is_left_to_its_other_writer() {
        let dir = TempDir::new().unwrap();
        let file = settings_file(&dir, "{}\n");
        let writes = AtomicUsize::new(0);

        let refused = edit(&file, |settings| {
            fs::write(
                &file,
                format!(r#"{{"theirs":{}}}"#, writes.fetch_add(1, Ordering::SeqCst)),
            )
            .unwrap();
            let mut updated = settings.clone();
            updated["ours"] = json!(1);
            HookEdit::Updated(updated)
        });

        assert!(refused.is_err());
        let written: Value = serde_json::from_str(&fs::read_to_string(&file).unwrap()).unwrap();
        assert!(written.get("ours").is_none());
    }
}
