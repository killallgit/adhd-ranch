//! Ranch places its local transport client; Claude's plugin lifecycle lives in
//! `claude_plugin`. Neither module edits Claude's hook settings directly.

use std::io;

#[cfg(unix)]
use std::fs;
#[cfg(unix)]
use std::path::Path;
use std::path::PathBuf;

use super::paths;

/// Make the bundled binary available at a path that survives an app upgrade.
/// Copy to a sibling first so a failed update leaves the working client intact.
#[cfg(unix)]
pub fn place_client() -> io::Result<PathBuf> {
    let bundled = paths::hook_client_bin()?;
    let installed = paths::stable_hook_client_bin()?;
    place_client_from(&bundled, &installed)?;
    Ok(installed)
}

/// Windows has no hook listener, so the client needs no placement there.
#[cfg(not(unix))]
pub fn place_client() -> io::Result<PathBuf> {
    paths::stable_hook_client_bin()
}

#[cfg(unix)]
fn place_client_from(bundled: &Path, installed: &Path) -> io::Result<()> {
    let source = fs::read(bundled)?;
    if fs::symlink_metadata(installed).is_ok_and(|metadata| metadata.file_type().is_file())
        && fs::read(installed).is_ok_and(|current| current == source)
    {
        return Ok(());
    }
    let parent = installed
        .parent()
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "client has no parent"))?;
    fs::create_dir_all(parent)?;
    let temporary = installed.with_extension(format!("tmp-{}", uuid::Uuid::now_v7()));
    let result = (|| {
        fs::copy(bundled, &temporary)?;
        fs::rename(&temporary, installed)
    })();
    if result.is_err() {
        let _ = fs::remove_file(&temporary);
    }
    result
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;

    #[test]
    fn placement_updates_contents() {
        let dir = tempfile::tempdir().unwrap();
        let bundled = dir.path().join("bundled");
        let installed = dir.path().join("data").join("adhd-ranch-hook");
        fs::write(&bundled, b"v1").unwrap();
        place_client_from(&bundled, &installed).unwrap();
        assert_eq!(fs::read(&installed).unwrap(), b"v1");

        fs::write(&bundled, b"v2").unwrap();
        place_client_from(&bundled, &installed).unwrap();
        assert_eq!(fs::read(&installed).unwrap(), b"v2");
    }

    #[test]
    fn failed_placement_keeps_existing_client() {
        let dir = tempfile::tempdir().unwrap();
        let bundled = dir.path().join("missing");
        let installed = dir.path().join("adhd-ranch-hook");
        fs::write(&installed, b"working").unwrap();

        assert!(place_client_from(&bundled, &installed).is_err());
        assert_eq!(fs::read(&installed).unwrap(), b"working");
    }
}
