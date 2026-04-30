use std::fs::{File, OpenOptions};
use std::io::{self, BufRead, BufReader, Write};
use std::marker::PhantomData;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use serde::{de::DeserializeOwned, Serialize};

use crate::atomic::atomic_write;

#[derive(Debug)]
pub enum JsonlError {
    Io(io::Error),
    Serde(serde_json::Error),
}

impl std::fmt::Display for JsonlError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(e) => write!(f, "jsonl io: {e}"),
            Self::Serde(e) => write!(f, "jsonl serde: {e}"),
        }
    }
}

impl std::error::Error for JsonlError {}

impl From<io::Error> for JsonlError {
    fn from(e: io::Error) -> Self {
        Self::Io(e)
    }
}

impl From<serde_json::Error> for JsonlError {
    fn from(e: serde_json::Error) -> Self {
        Self::Serde(e)
    }
}

/// Append-only JSONL file with crash-safe rewrite.
///
/// Concurrency model: a process-local `Mutex` serializes writers within the
/// process; `atomic_write` (tmpfile + flock + rename) protects against partial
/// reads by readers in the same or other processes.
pub struct JsonlLog<T> {
    path: PathBuf,
    write_lock: Mutex<()>,
    _marker: PhantomData<T>,
}

impl<T> JsonlLog<T>
where
    T: Serialize + DeserializeOwned,
{
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self {
            path: path.into(),
            write_lock: Mutex::new(()),
            _marker: PhantomData,
        }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn append(&self, item: &T) -> Result<(), JsonlError> {
        let _guard = self.write_lock.lock().unwrap_or_else(|e| e.into_inner());
        if let Some(parent) = self.path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let mut line = serde_json::to_string(item)?;
        line.push('\n');
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)?;
        file.write_all(line.as_bytes())?;
        file.sync_data()?;
        Ok(())
    }

    pub fn read_all(&self) -> Result<Vec<T>, JsonlError> {
        let file = match File::open(&self.path) {
            Ok(f) => f,
            Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(Vec::new()),
            Err(e) => return Err(e.into()),
        };
        let reader = BufReader::new(file);
        let mut out = Vec::new();
        for line in reader.lines() {
            let line = line?;
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }
            out.push(serde_json::from_str(trimmed)?);
        }
        Ok(out)
    }

    /// Read all items, hand them to `mutate`, and atomically rewrite the file.
    /// Holds the write lock across read + write, so concurrent appends in this
    /// process cannot interleave.
    ///
    /// Empty result removes the file (so "no items" = "no file").
    pub fn modify<F, R>(&self, mutate: F) -> Result<R, JsonlError>
    where
        F: FnOnce(&mut Vec<T>) -> R,
    {
        let _guard = self.write_lock.lock().unwrap_or_else(|e| e.into_inner());
        let mut items = self.read_all()?;
        let result = mutate(&mut items);
        if items.is_empty() {
            let _ = std::fs::remove_file(&self.path);
        } else {
            let mut buf = String::new();
            for item in &items {
                buf.push_str(&serde_json::to_string(item)?);
                buf.push('\n');
            }
            atomic_write(&self.path, buf.as_bytes())?;
        }
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;
    use tempfile::TempDir;

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    struct Item {
        id: String,
        value: u32,
    }

    fn item(id: &str, value: u32) -> Item {
        Item {
            id: id.into(),
            value,
        }
    }

    #[test]
    fn read_all_on_missing_file_returns_empty() {
        let dir = TempDir::new().unwrap();
        let log: JsonlLog<Item> = JsonlLog::new(dir.path().join("a.jsonl"));
        assert!(log.read_all().unwrap().is_empty());
    }

    #[test]
    fn append_creates_file_and_persists_in_order() {
        let dir = TempDir::new().unwrap();
        let log: JsonlLog<Item> = JsonlLog::new(dir.path().join("nested/a.jsonl"));
        log.append(&item("a", 1)).unwrap();
        log.append(&item("b", 2)).unwrap();
        let listed = log.read_all().unwrap();
        assert_eq!(listed, vec![item("a", 1), item("b", 2)]);
    }

    #[test]
    fn read_all_skips_blank_lines() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("a.jsonl");
        std::fs::write(&path, "{\"id\":\"x\",\"value\":7}\n\n\n").unwrap();
        let log: JsonlLog<Item> = JsonlLog::new(&path);
        assert_eq!(log.read_all().unwrap(), vec![item("x", 7)]);
    }

    #[test]
    fn modify_can_drop_items_and_rewrite() {
        let dir = TempDir::new().unwrap();
        let log: JsonlLog<Item> = JsonlLog::new(dir.path().join("a.jsonl"));
        log.append(&item("a", 1)).unwrap();
        log.append(&item("b", 2)).unwrap();
        let removed = log
            .modify(|items| {
                let before = items.len();
                items.retain(|i| i.id != "a");
                items.len() != before
            })
            .unwrap();
        assert!(removed);
        assert_eq!(log.read_all().unwrap(), vec![item("b", 2)]);
    }

    #[test]
    fn modify_to_empty_removes_file() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("a.jsonl");
        let log: JsonlLog<Item> = JsonlLog::new(&path);
        log.append(&item("a", 1)).unwrap();
        log.modify(|items| items.clear()).unwrap();
        assert!(!path.exists());
    }
}
