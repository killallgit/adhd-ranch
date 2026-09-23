//! Claude owns plugin registration. Ranch only places the local transport client
//! and reads plugin status; neither action edits Claude's settings.

use std::io;
use std::process::Command;

#[cfg(unix)]
use std::fs;
#[cfg(unix)]
use std::path::Path;
use std::path::PathBuf;

use serde::Deserialize;

use super::paths;

const PLUGIN_ID: &str = "adhd-ranch-hooks@adhd-ranch";

#[derive(Deserialize)]
struct ListedPlugin {
    id: String,
    enabled: bool,
}

pub fn plugin_enabled() -> io::Result<bool> {
    let output = match Command::new("claude")
        .args(["plugin", "list", "--json"])
        .output()
    {
        Ok(output) => output,
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            // Finder-launched macOS apps often lack ~/.local/bin in PATH, where
            // Claude's native installer places the CLI.
            let home = std::env::var_os("HOME").ok_or(error)?;
            Command::new(PathBuf::from(home).join(".local/bin/claude"))
                .args(["plugin", "list", "--json"])
                .output()?
        }
        Err(error) => return Err(error),
    };
    if !output.status.success() {
        return Err(io::Error::other(format!(
            "claude plugin list exited with {}",
            output.status
        )));
    }
    plugin_enabled_from(&output.stdout)
}

fn plugin_enabled_from(raw: &[u8]) -> io::Result<bool> {
    let listed: Vec<ListedPlugin> = serde_json::from_slice(raw)
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;
    Ok(listed
        .iter()
        .any(|plugin| plugin.id == PLUGIN_ID && plugin.enabled))
}

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

    #[test]
    fn plugin_status_requires_the_matching_enabled_plugin() {
        assert!(!plugin_enabled_from(b"[]").unwrap());
        assert!(
            !plugin_enabled_from(br#"[{"id":"adhd-ranch-hooks@adhd-ranch","enabled":false}]"#)
                .unwrap()
        );
        assert!(plugin_enabled_from(
            br#"[{"id":"other@adhd-ranch","enabled":true},{"id":"adhd-ranch-hooks@adhd-ranch","enabled":true}]"#
        )
        .unwrap());
        assert!(plugin_enabled_from(b"{}").is_err());
    }
}
