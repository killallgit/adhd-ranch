use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::Path;

use serde_json::{Map, Value};

use crate::atomic::{atomic_write, tmp_path};

/// Read a settings document, treating "not there yet" as an empty one.
///
/// `Ok(None)` means the file exists but is not JSON we can reason about; the caller
/// must then leave it alone rather than replace it with something we invented.
pub fn read(path: &Path) -> io::Result<Option<Value>> {
    match fs::read_to_string(path) {
        Ok(raw) => Ok(serde_json::from_str(&raw).ok()),
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            Ok(Some(Value::Object(Map::new())))
        }
        Err(error) => Err(error),
    }
}

pub fn write(path: &Path, settings: &Value) -> io::Result<()> {
    let mut content = serde_json::to_string_pretty(settings)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    content.push('\n');
    replace(path, content.as_bytes())
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
