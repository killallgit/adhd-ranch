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

pub fn settings_file() -> io::Result<PathBuf> {
    Ok(data_root()?.join("settings.yaml"))
}

/// Where the ranch listens for hook firings.
///
/// Kept directly under the data root because a Unix socket path is limited to about
/// a hundred bytes, and nesting spends that budget for nothing.
pub fn agent_hook_socket() -> io::Result<PathBuf> {
    Ok(data_root()?.join("agent-hooks.sock"))
}

/// The client an agent runs, which ships beside the app's own binary.
pub fn hook_client_bin() -> io::Result<PathBuf> {
    let exe = std::env::current_exe()?;
    let dir = exe.parent().ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::NotFound,
            "executable has no parent directory",
        )
    })?;
    Ok(dir.join("adhd-ranch-hook"))
}

pub fn claude_settings_file() -> io::Result<PathBuf> {
    claude_settings_file_from_env(|key| std::env::var_os(key))
}

fn claude_settings_file_from_env(
    mut var: impl FnMut(&str) -> Option<OsString>,
) -> io::Result<PathBuf> {
    if let Some(config_dir) = var("CLAUDE_CONFIG_DIR") {
        return Ok(PathBuf::from(config_dir).join("settings.json"));
    }
    let home = var("HOME")
        .or_else(|| var("USERPROFILE"))
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "home directory not found"))?;
    Ok(PathBuf::from(home).join(".claude").join("settings.json"))
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

    #[test]
    fn claude_settings_default_to_home_dot_claude() {
        let vars: HashMap<&str, OsString> =
            HashMap::from([("HOME", OsString::from("/Users/ryan"))]);
        assert_eq!(
            claude_settings_file_from_env(|key| vars.get(key).cloned()).unwrap(),
            PathBuf::from("/Users/ryan/.claude/settings.json")
        );
    }

    #[test]
    fn claude_config_dir_overrides_home_for_claude_settings() {
        let vars: HashMap<&str, OsString> = HashMap::from([
            ("CLAUDE_CONFIG_DIR", OsString::from("/tmp/claude")),
            ("HOME", OsString::from("/Users/ryan")),
        ]);
        assert_eq!(
            claude_settings_file_from_env(|key| vars.get(key).cloned()).unwrap(),
            PathBuf::from("/tmp/claude/settings.json")
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
