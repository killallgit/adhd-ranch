use std::ffi::OsString;
use std::io;
use std::path::PathBuf;

pub fn data_root() -> io::Result<PathBuf> {
    data_root_from_env(|key| std::env::var_os(key))
}

fn data_root_from_env(mut var: impl FnMut(&str) -> Option<OsString>) -> io::Result<PathBuf> {
    if let Some(root) = var("ADHD_RANCH_HOME") {
        return Ok(PathBuf::from(root));
    }

    #[cfg(target_os = "windows")]
    {
        if let Some(appdata) = var("APPDATA") {
            return Ok(PathBuf::from(appdata).join("adhd-ranch"));
        }
        if let Some(profile) = var("USERPROFILE") {
            return Ok(PathBuf::from(profile).join(".adhd-ranch"));
        }
    }

    let home = var("HOME")
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "home directory not found"))?;
    Ok(PathBuf::from(home).join(".adhd-ranch"))
}

pub fn focuses_root() -> io::Result<PathBuf> {
    Ok(data_root()?.join("focuses"))
}

pub fn port_file() -> io::Result<PathBuf> {
    Ok(data_root()?.join("run/port"))
}

pub fn proposals_file() -> io::Result<PathBuf> {
    Ok(data_root()?.join("proposals.jsonl"))
}

pub fn decisions_file() -> io::Result<PathBuf> {
    Ok(data_root()?.join("decisions.jsonl"))
}

pub fn settings_file() -> io::Result<PathBuf> {
    Ok(data_root()?.join("settings.yaml"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn root(vars: &[(&str, &str)]) -> io::Result<PathBuf> {
        let vars: HashMap<&str, OsString> =
            vars.iter().map(|(k, v)| (*k, OsString::from(v))).collect();
        data_root_from_env(|key| vars.get(key).cloned())
    }

    #[test]
    fn explicit_root_overrides_platform_defaults() {
        assert_eq!(
            root(&[("ADHD_RANCH_HOME", "/tmp/ranch"), ("HOME", "/Users/ryan")]).unwrap(),
            PathBuf::from("/tmp/ranch")
        );
    }

    #[test]
    fn unix_default_uses_home_dot_dir() {
        assert_eq!(
            root(&[("HOME", "/Users/ryan")]).unwrap(),
            PathBuf::from("/Users/ryan").join(".adhd-ranch")
        );
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn windows_default_uses_appdata() {
        assert_eq!(
            root(&[("APPDATA", "C:\\Users\\ryan\\AppData\\Roaming")]).unwrap(),
            PathBuf::from("C:\\Users\\ryan\\AppData\\Roaming").join("adhd-ranch")
        );
    }
}
