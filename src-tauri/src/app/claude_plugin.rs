//! Owns Claude's user-scoped Ranch plugin lifecycle. The UI asks for an install;
//! only this module knows the CLI commands, marketplace identity, and status format.

use std::io;
use std::path::PathBuf;
use std::process::{Command, Output, Stdio};

use serde::Deserialize;

const MARKETPLACE_NAME: &str = "adhd-ranch";
const MARKETPLACE_SOURCE: &str = "killallgit/adhd-ranch";
const PLUGIN_ID: &str = "adhd-ranch-hooks@adhd-ranch";

#[derive(Deserialize)]
struct ListedPlugin {
    id: String,
    enabled: bool,
    #[serde(default)]
    scope: String,
}

#[derive(Deserialize)]
struct ListedMarketplace {
    name: String,
    repo: Option<String>,
}

/// Read-only status for the Agent Hooks debug window.
pub fn plugin_enabled() -> io::Result<bool> {
    plugin_enabled_from(&run_claude(&["plugin", "list", "--json"])?)
}

fn plugin_enabled_from(raw: &[u8]) -> io::Result<bool> {
    Ok(listed_plugins(raw)?
        .iter()
        .any(|plugin| plugin.id == PLUGIN_ID && plugin.enabled))
}

fn user_plugin_state(raw: &[u8]) -> io::Result<Option<bool>> {
    Ok(listed_plugins(raw)?
        .into_iter()
        .find(|plugin| plugin.id == PLUGIN_ID && plugin.scope == "user")
        .map(|plugin| plugin.enabled))
}

fn listed_plugins(raw: &[u8]) -> io::Result<Vec<ListedPlugin>> {
    serde_json::from_slice(raw).map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))
}

fn marketplace_present(raw: &[u8]) -> io::Result<bool> {
    let marketplaces: Vec<ListedMarketplace> = serde_json::from_slice(raw)
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;
    match marketplaces
        .iter()
        .find(|marketplace| marketplace.name == MARKETPLACE_NAME)
    {
        Some(marketplace) if marketplace.repo.as_deref() == Some(MARKETPLACE_SOURCE) => Ok(true),
        Some(_) => Err(io::Error::new(
            io::ErrorKind::AlreadyExists,
            "Claude has an adhd-ranch marketplace from a different source",
        )),
        None => Ok(false),
    }
}

/// Idempotent: an enabled user install is left alone; a disabled one is enabled;
/// a missing one is installed through Claude's own marketplace commands.
fn install_user_with(mut run: impl FnMut(&[&str]) -> io::Result<Vec<u8>>) -> io::Result<()> {
    let user_state = user_plugin_state(&run(&["plugin", "list", "--json"])?)?;
    if !marketplace_present(&run(&["plugin", "marketplace", "list", "--json"])?)? {
        run(&[
            "plugin",
            "marketplace",
            "add",
            MARKETPLACE_SOURCE,
            "--scope",
            "user",
        ])?;
    }

    match user_state {
        Some(true) => return Ok(()),
        Some(false) => {
            run(&["plugin", "enable", PLUGIN_ID, "--scope", "user"])?;
        }
        None => {
            run(&["plugin", "install", PLUGIN_ID, "--scope", "user"])?;
        }
    }

    if user_plugin_state(&run(&["plugin", "list", "--json"])?)? != Some(true) {
        return Err(io::Error::other(
            "Claude did not report the user-scoped Ranch plugin as enabled after installation",
        ));
    }
    Ok(())
}

fn install_user() -> io::Result<()> {
    install_user_with(run_claude)
}

#[tauri::command]
pub async fn install_claude_hooks(window: tauri::Window) -> Result<(), String> {
    if window.label() != "install-hooks" {
        return Err("Hook installation is only available from the Install Hooks window".into());
    }
    tauri::async_runtime::spawn_blocking(install_user)
        .await
        .map_err(|error| format!("Hook installation could not finish: {error}"))?
        .map_err(|error| error.to_string())
}

fn run_claude(args: &[&str]) -> io::Result<Vec<u8>> {
    let output = match command_output("claude", args) {
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            // Finder-launched apps often omit the native installer's bin directory.
            let home = std::env::var_os("HOME")
                .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "Claude CLI not found"))?;
            command_output(PathBuf::from(home).join(".local/bin/claude"), args).map_err(
                |error| {
                    if error.kind() == io::ErrorKind::NotFound {
                        io::Error::new(io::ErrorKind::NotFound, "Claude CLI not found")
                    } else {
                        error
                    }
                },
            )?
        }
        result => result?,
    };
    if !output.status.success() {
        return Err(io::Error::other(format!(
            "claude {} failed ({})",
            args.join(" "),
            output.status,
        )));
    }
    Ok(output.stdout)
}

fn command_output(program: impl AsRef<std::ffi::OsStr>, args: &[&str]) -> io::Result<Output> {
    Command::new(program)
        .args(args)
        .stdin(Stdio::null())
        .output()
}

#[cfg(test)]
mod tests {
    use super::*;

    const ENABLED: &[u8] =
        br#"[{"id":"adhd-ranch-hooks@adhd-ranch","enabled":true,"scope":"user"}]"#;
    const DISABLED: &[u8] =
        br#"[{"id":"adhd-ranch-hooks@adhd-ranch","enabled":false,"scope":"user"}]"#;
    const MARKETPLACE: &[u8] = br#"[{"name":"adhd-ranch","repo":"killallgit/adhd-ranch"}]"#;

    #[test]
    fn status_requires_the_matching_enabled_plugin() {
        assert!(!plugin_enabled_from(b"[]").unwrap());
        assert!(!plugin_enabled_from(DISABLED).unwrap());
        assert!(plugin_enabled_from(ENABLED).unwrap());
        assert!(plugin_enabled_from(b"{}").is_err());
    }

    #[test]
    fn already_enabled_does_not_repeat_installation() {
        let mut calls = Vec::new();
        install_user_with(|args| {
            calls.push(args.iter().map(|arg| (*arg).to_owned()).collect::<Vec<_>>());
            Ok(if calls.len() == 1 {
                ENABLED.to_vec()
            } else {
                MARKETPLACE.to_vec()
            })
        })
        .unwrap();
        assert_eq!(
            calls,
            vec![
                vec!["plugin", "list", "--json"],
                vec!["plugin", "marketplace", "list", "--json"],
            ]
        );
    }

    #[test]
    fn missing_plugin_adds_marketplace_and_installs_at_user_scope() {
        let mut calls = Vec::new();
        install_user_with(|args| {
            calls.push(args.iter().map(|arg| (*arg).to_owned()).collect::<Vec<_>>());
            Ok(match calls.len() {
                1 => b"[]".to_vec(),
                2 => b"[]".to_vec(),
                5 => ENABLED.to_vec(),
                _ => Vec::new(),
            })
        })
        .unwrap();
        assert_eq!(
            calls,
            vec![
                vec!["plugin", "list", "--json"],
                vec!["plugin", "marketplace", "list", "--json"],
                vec![
                    "plugin",
                    "marketplace",
                    "add",
                    MARKETPLACE_SOURCE,
                    "--scope",
                    "user"
                ],
                vec!["plugin", "install", PLUGIN_ID, "--scope", "user"],
                vec!["plugin", "list", "--json"],
            ]
        );
    }

    #[test]
    fn disabled_user_plugin_is_enabled_without_reinstalling() {
        let mut calls = Vec::new();
        install_user_with(|args| {
            calls.push(args.iter().map(|arg| (*arg).to_owned()).collect::<Vec<_>>());
            Ok(match calls.len() {
                1 => DISABLED.to_vec(),
                2 => MARKETPLACE.to_vec(),
                4 => ENABLED.to_vec(),
                _ => Vec::new(),
            })
        })
        .unwrap();
        assert_eq!(
            calls,
            vec![
                vec!["plugin", "list", "--json"],
                vec!["plugin", "marketplace", "list", "--json"],
                vec!["plugin", "enable", PLUGIN_ID, "--scope", "user"],
                vec!["plugin", "list", "--json"],
            ]
        );
    }

    #[test]
    fn existing_marketplace_does_not_get_added_twice() {
        let mut calls = Vec::new();
        install_user_with(|args| {
            calls.push(args.iter().map(|arg| (*arg).to_owned()).collect::<Vec<_>>());
            Ok(match calls.len() {
                1 => b"[]".to_vec(),
                2 => MARKETPLACE.to_vec(),
                4 => ENABLED.to_vec(),
                _ => Vec::new(),
            })
        })
        .unwrap();
        assert_eq!(
            calls,
            vec![
                vec!["plugin", "list", "--json"],
                vec!["plugin", "marketplace", "list", "--json"],
                vec!["plugin", "install", PLUGIN_ID, "--scope", "user"],
                vec!["plugin", "list", "--json"],
            ]
        );
    }

    #[test]
    fn conflicting_marketplace_source_stops_before_installation() {
        let mut calls = Vec::new();
        let result = install_user_with(|args| {
            calls.push(args.iter().map(|arg| (*arg).to_owned()).collect::<Vec<_>>());
            Ok(if calls.len() == 1 {
                b"[]".to_vec()
            } else {
                br#"[{"name":"adhd-ranch","repo":"someone-else/adhd-ranch"}]"#.to_vec()
            })
        });
        assert_eq!(result.unwrap_err().kind(), io::ErrorKind::AlreadyExists);
        assert_eq!(calls.len(), 2);
    }

    #[test]
    fn install_is_not_reported_successful_until_claude_confirms_it() {
        let mut calls = Vec::new();
        let result = install_user_with(|args| {
            calls.push(args.iter().map(|arg| (*arg).to_owned()).collect::<Vec<_>>());
            Ok(if calls.len() == 2 {
                MARKETPLACE.to_vec()
            } else {
                b"[]".to_vec()
            })
        });
        assert!(result.is_err());
        assert_eq!(calls.len(), 4);
    }
}
