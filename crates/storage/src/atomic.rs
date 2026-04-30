use std::fs::{self, File, OpenOptions};
use std::io::{self, Write};
use std::path::Path;

use fs2::FileExt;

const TMP_SUFFIX: &str = ".tmp";

pub fn atomic_write(target: &Path, content: &[u8]) -> io::Result<()> {
    let parent = target
        .parent()
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "target has no parent"))?;
    fs::create_dir_all(parent)?;

    let lock_path = target.with_extension(format!(
        "{}.lock",
        target
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or("focus")
    ));
    let lock = File::create(&lock_path)?;
    FileExt::lock_exclusive(&lock)?;

    let result = (|| {
        let tmp = tmp_path(target);
        {
            let mut f = OpenOptions::new()
                .create(true)
                .truncate(true)
                .write(true)
                .open(&tmp)?;
            f.write_all(content)?;
            f.sync_data()?;
        }
        fs::rename(&tmp, target)?;
        Ok::<(), io::Error>(())
    })();

    let _ = FileExt::unlock(&lock);
    let _ = fs::remove_file(&lock_path);
    result
}

pub fn tmp_path(target: &Path) -> std::path::PathBuf {
    let mut s = target.as_os_str().to_os_string();
    s.push(TMP_SUFFIX);
    std::path::PathBuf::from(s)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn writes_then_reads_back() {
        let dir = TempDir::new().unwrap();
        let target = dir.path().join("a.md");
        atomic_write(&target, b"hello").unwrap();
        assert_eq!(fs::read_to_string(&target).unwrap(), "hello");
    }

    #[test]
    fn replaces_existing_content_atomically() {
        let dir = TempDir::new().unwrap();
        let target = dir.path().join("a.md");
        fs::write(&target, "old").unwrap();
        atomic_write(&target, b"new").unwrap();
        assert_eq!(fs::read_to_string(&target).unwrap(), "new");
    }

    #[test]
    fn stale_tmp_does_not_pollute_reader() {
        let dir = TempDir::new().unwrap();
        let target = dir.path().join("a.md");
        fs::write(&target, "real").unwrap();
        fs::write(tmp_path(&target), "garbage").unwrap();
        assert_eq!(fs::read_to_string(&target).unwrap(), "real");
    }

    #[test]
    fn atomic_write_overwrites_stale_tmp() {
        let dir = TempDir::new().unwrap();
        let target = dir.path().join("a.md");
        fs::write(&target, "v1").unwrap();
        fs::write(tmp_path(&target), "stale junk that crashed").unwrap();
        atomic_write(&target, b"v2").unwrap();
        assert_eq!(fs::read_to_string(&target).unwrap(), "v2");
    }

    #[test]
    fn simulated_crash_before_rename_leaves_target_intact() {
        let dir = TempDir::new().unwrap();
        let target = dir.path().join("a.md");
        fs::write(&target, "stable").unwrap();

        // Simulate the in-flight write: a partial tmp file exists, but the
        // process crashed before fs::rename ran. Readers must still see the
        // pre-crash target content.
        let tmp = tmp_path(&target);
        {
            let mut f = OpenOptions::new()
                .create(true)
                .truncate(true)
                .write(true)
                .open(&tmp)
                .unwrap();
            f.write_all(b"partial junk").unwrap();
            f.sync_data().unwrap();
        }

        assert_eq!(fs::read_to_string(&target).unwrap(), "stable");
        assert!(tmp.exists());
        assert_ne!(fs::read_to_string(&tmp).unwrap(), "stable");
    }
}
