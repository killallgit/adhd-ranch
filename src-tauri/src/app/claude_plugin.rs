//! Owns Claude's user-scoped Ranch plugin lifecycle. The UI asks for an install;
//! only this module knows the CLI commands, marketplace identity, and status format.

mod contract;

use std::io;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};

use serde::Deserialize;

const MARKETPLACE_NAME: &str = "adhd-ranch";
const MARKETPLACE_SOURCE: &str = "killallgit/adhd-ranch";
const PLUGIN_ID: &str = "adhd-ranch-hooks@adhd-ranch";

#[derive(Clone, Deserialize)]
struct ListedPlugin {
    id: String,
    enabled: bool,
    #[serde(default)]
    scope: String,
    #[serde(rename = "installPath")]
    install_path: Option<PathBuf>,
    #[serde(default)]
    errors: Vec<String>,
    #[serde(default)]
    notes: Vec<String>,
}

#[derive(Deserialize)]
struct ListedMarketplace {
    name: String,
    repo: Option<String>,
    url: Option<String>,
    #[serde(rename = "installLocation")]
    install_location: Option<PathBuf>,
}

/// Read-only status for the tray and Agent Hooks window. A listed, enabled
/// plugin with missing or altered hooks is not a working installation.
pub fn plugin_enabled() -> io::Result<bool> {
    plugin_enabled_from(&run_claude(&["plugin", "list", "--json"])?)
}

fn plugin_enabled_from(raw: &[u8]) -> io::Result<bool> {
    let Some(plugin) = user_plugin(raw)? else {
        return Ok(false);
    };
    if !plugin.enabled {
        return Ok(false);
    }
    match validate_listed_plugin(&plugin, contract::validate) {
        Ok(()) => Ok(true),
        Err(error) => {
            log::warn!("Claude Ranch plugin is enabled but incomplete: {error}");
            Ok(false)
        }
    }
}

fn user_plugin(raw: &[u8]) -> io::Result<Option<ListedPlugin>> {
    Ok(listed_plugins(raw)?
        .into_iter()
        .find(|plugin| plugin.id == PLUGIN_ID && plugin.scope == "user"))
}

fn listed_plugins(raw: &[u8]) -> io::Result<Vec<ListedPlugin>> {
    serde_json::from_slice(raw).map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))
}

fn marketplace_location(raw: &[u8]) -> io::Result<Option<PathBuf>> {
    let marketplaces: Vec<ListedMarketplace> = serde_json::from_slice(raw)
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;
    match marketplaces
        .iter()
        .find(|marketplace| marketplace.name == MARKETPLACE_NAME)
    {
        Some(marketplace) => {
            if !matches_ranch_marketplace_source(marketplace)? {
                return Err(io::Error::new(
                    io::ErrorKind::AlreadyExists,
                    "Claude has an adhd-ranch marketplace from a different source",
                ));
            }
            marketplace
                .install_location
                .clone()
                .map(Some)
                .ok_or_else(|| {
                    io::Error::new(
                        io::ErrorKind::InvalidData,
                        "Claude did not report the Ranch marketplace location",
                    )
                })
        }
        None => Ok(None),
    }
}

fn matches_ranch_marketplace_source(marketplace: &ListedMarketplace) -> io::Result<bool> {
    if let Some(repo) = marketplace.repo.as_deref() {
        return Ok(repo == MARKETPLACE_SOURCE);
    }
    match marketplace.url.as_deref() {
        Some(
            "https://github.com/killallgit/adhd-ranch.git"
            | "https://github.com/killallgit/adhd-ranch"
            | "ssh://git@github.com/killallgit/adhd-ranch.git"
            | "ssh://git@github.com/killallgit/adhd-ranch",
        ) => Ok(true),
        Some(url) => Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("Claude has an adhd-ranch marketplace with an unsupported source URL: {url}"),
        )),
        None => Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "Claude has an adhd-ranch marketplace with an unsupported source",
        )),
    }
}

fn validate_listed_plugin(
    plugin: &ListedPlugin,
    mut inspect: impl FnMut(&Path) -> io::Result<()>,
) -> io::Result<()> {
    if !plugin.errors.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!(
                "Claude could not load Ranch plugin: {}",
                plugin.errors.join("; ")
            ),
        ));
    }
    if !plugin.notes.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!(
                "Claude reported Ranch plugin warnings: {}",
                plugin.notes.join("; ")
            ),
        ));
    }
    let root = plugin.install_path.as_deref().ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            "Claude did not report the Ranch plugin location",
        )
    })?;
    inspect(root)
}

fn validate_with_claude(root: &Path) -> io::Result<()> {
    contract::validate(root)?;
    let path = root.to_str().ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            "Claude plugin path is not UTF-8",
        )
    })?;
    run_claude(&["plugin", "validate", path, "--strict"])?;
    Ok(())
}

/// Only the explicit button may make changes. A valid install is untouched;
/// an incomplete one is updated first and reinstalled only if update cannot
/// restore the full contract. Claude owns every plugin write.
fn install_user_with(
    mut run: impl FnMut(&[&str]) -> io::Result<Vec<u8>>,
    mut inspect: impl FnMut(&Path) -> io::Result<()>,
) -> io::Result<()> {
    let previous = user_plugin(&run(&["plugin", "list", "--json"])?)?;
    let mut marketplace =
        marketplace_location(&run(&["plugin", "marketplace", "list", "--json"])?)?;
    if marketplace.is_none() {
        run(&[
            "plugin",
            "marketplace",
            "add",
            MARKETPLACE_SOURCE,
            "--scope",
            "user",
        ])?;
        marketplace = marketplace_location(&run(&["plugin", "marketplace", "list", "--json"])?)?;
    }
    let marketplace = marketplace.ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::NotFound,
            "Claude did not add the Ranch marketplace",
        )
    })?;

    match previous {
        Some(plugin) if validate_listed_plugin(&plugin, &mut inspect).is_ok() => {
            if plugin.enabled {
                return Ok(());
            }
            run(&["plugin", "enable", PLUGIN_ID, "--scope", "user"])?;
        }
        Some(_) => {
            run(&["plugin", "marketplace", "update", MARKETPLACE_NAME])?;
            let refreshed =
                marketplace_location(&run(&["plugin", "marketplace", "list", "--json"])?)?
                    .ok_or_else(|| {
                        io::Error::new(
                            io::ErrorKind::NotFound,
                            "Ranch marketplace disappeared during update",
                        )
                    })?;
            inspect(&refreshed.join("plugins/adhd-ranch-hooks"))?;
            run(&["plugin", "update", PLUGIN_ID, "--scope", "user"])?;
            let updated = user_plugin(&run(&["plugin", "list", "--json"])?)?;
            if updated.as_ref().map_or(true, |plugin| {
                validate_listed_plugin(plugin, &mut inspect).is_err()
            }) {
                // Claude can skip an update when its version key is unchanged.
                // Reinstall only the user-scoped Ranch plugin on this explicit
                // request; never touch project plugins or settings hooks.
                run(&["plugin", "uninstall", PLUGIN_ID, "--scope", "user"])?;
                run(&["plugin", "install", PLUGIN_ID, "--scope", "user"])?;
            } else if updated.as_ref().is_some_and(|plugin| !plugin.enabled) {
                run(&["plugin", "enable", PLUGIN_ID, "--scope", "user"])?;
            }
        }
        None => {
            inspect(&marketplace.join("plugins/adhd-ranch-hooks"))?;
            run(&["plugin", "install", PLUGIN_ID, "--scope", "user"])?;
        }
    }

    let installed = user_plugin(&run(&["plugin", "list", "--json"])?)?.ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::NotFound,
            "Claude did not list the user-scoped Ranch plugin",
        )
    })?;
    if !installed.enabled {
        return Err(io::Error::other(
            "Claude did not report the user-scoped Ranch plugin as enabled after installation",
        ));
    }
    validate_listed_plugin(&installed, inspect)?;
    Ok(())
}

fn install_user() -> io::Result<()> {
    install_user_with(run_claude, validate_with_claude)
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
        let details = if output.stderr.is_empty() {
            String::from_utf8_lossy(&output.stdout)
        } else {
            String::from_utf8_lossy(&output.stderr)
        };
        let details = details.trim();
        return Err(io::Error::other(format!(
            "claude {} failed ({}): {}",
            args.join(" "),
            output.status,
            if details.is_empty() {
                "no details from Claude"
            } else {
                details
            },
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

    const MARKETPLACE: &[u8] = br#"[{"name":"adhd-ranch","repo":"killallgit/adhd-ranch","installLocation":"/marketplace"}]"#;

    fn installed(enabled: bool) -> Vec<u8> {
        serde_json::to_vec(&serde_json::json!([{
            "id": PLUGIN_ID,
            "scope": "user",
            "enabled": enabled,
            "installPath": "/installed"
        }]))
        .unwrap()
    }

    fn source_path() -> PathBuf {
        PathBuf::from("/marketplace/plugins/adhd-ranch-hooks")
    }

    #[test]
    fn existing_marketplace_added_by_git_url_is_recognized() {
        for url in [
            "https://github.com/killallgit/adhd-ranch.git",
            "https://github.com/killallgit/adhd-ranch",
        ] {
            let listing = serde_json::to_vec(&serde_json::json!([{
                "name": MARKETPLACE_NAME,
                "source": "git",
                "url": url,
                "installLocation": "/marketplace"
            }]))
            .unwrap();
            assert_eq!(
                marketplace_location(&listing).unwrap(),
                Some(PathBuf::from("/marketplace"))
            );
        }
    }

    #[test]
    fn unsupported_marketplace_source_is_reported_as_unsupported() {
        let listing = br#"[{"name":"adhd-ranch","source":"git","url":"https://example.com/adhd-ranch.git","installLocation":"/marketplace"}]"#;
        let error = marketplace_location(listing).unwrap_err();
        assert_eq!(error.kind(), io::ErrorKind::InvalidData);
        assert!(error.to_string().contains("unsupported"));
    }

    #[test]
    fn status_requires_the_matching_enabled_and_complete_user_plugin() {
        assert!(!plugin_enabled_from(b"[]").unwrap());
        assert!(!plugin_enabled_from(&installed(false)).unwrap());
        assert!(!plugin_enabled_from(&installed(true)).unwrap());
        let source = Path::new(env!("CARGO_MANIFEST_DIR")).join("../plugins/adhd-ranch-hooks");
        let valid = serde_json::to_vec(&serde_json::json!([{
            "id": PLUGIN_ID,
            "scope": "user",
            "enabled": true,
            "installPath": source
        }]))
        .unwrap();
        assert!(plugin_enabled_from(&valid).unwrap());
        let warning = serde_json::to_vec(&serde_json::json!([{
            "id": PLUGIN_ID,
            "scope": "user",
            "enabled": true,
            "installPath": source,
            "notes": ["unknown hook field"]
        }]))
        .unwrap();
        assert!(!plugin_enabled_from(&warning).unwrap());
        assert!(plugin_enabled_from(b"{}").is_err());
    }

    #[test]
    fn already_enabled_and_valid_does_not_repeat_installation() {
        let mut calls = Vec::new();
        install_user_with(
            |args| {
                calls.push(args.iter().map(|arg| (*arg).to_owned()).collect::<Vec<_>>());
                Ok(if calls.len() == 1 {
                    installed(true)
                } else {
                    MARKETPLACE.to_vec()
                })
            },
            |_| Ok(()),
        )
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
        let mut inspected = Vec::new();
        install_user_with(
            |args| {
                calls.push(args.iter().map(|arg| (*arg).to_owned()).collect::<Vec<_>>());
                Ok(match calls.len() {
                    1 | 2 | 3 | 5 => b"[]".to_vec(),
                    4 => MARKETPLACE.to_vec(),
                    6 => installed(true),
                    _ => panic!("unexpected Claude command"),
                })
            },
            |path| {
                inspected.push(path.to_path_buf());
                Ok(())
            },
        )
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
                vec!["plugin", "marketplace", "list", "--json"],
                vec!["plugin", "install", PLUGIN_ID, "--scope", "user"],
                vec!["plugin", "list", "--json"],
            ]
        );
        assert_eq!(inspected, vec![source_path(), PathBuf::from("/installed")]);
    }

    #[test]
    fn disabled_user_plugin_is_enabled_without_reinstalling() {
        let mut calls = Vec::new();
        install_user_with(
            |args| {
                calls.push(args.iter().map(|arg| (*arg).to_owned()).collect::<Vec<_>>());
                Ok(match calls.len() {
                    1 => installed(false),
                    2 => MARKETPLACE.to_vec(),
                    4 => installed(true),
                    _ => Vec::new(),
                })
            },
            |_| Ok(()),
        )
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
        install_user_with(
            |args| {
                calls.push(args.iter().map(|arg| (*arg).to_owned()).collect::<Vec<_>>());
                Ok(match calls.len() {
                    1 => b"[]".to_vec(),
                    2 => MARKETPLACE.to_vec(),
                    4 => installed(true),
                    _ => Vec::new(),
                })
            },
            |_| Ok(()),
        )
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
        let result = install_user_with(
            |args| {
                calls.push(args.iter().map(|arg| (*arg).to_owned()).collect::<Vec<_>>());
                Ok(if calls.len() == 1 {
                    b"[]".to_vec()
                } else {
                    br#"[{"name":"adhd-ranch","repo":"someone-else/adhd-ranch"}]"#.to_vec()
                })
            },
            |_| Ok(()),
        );
        assert_eq!(result.unwrap_err().kind(), io::ErrorKind::AlreadyExists);
        assert_eq!(calls.len(), 2);
    }

    #[test]
    fn install_is_not_reported_successful_until_claude_confirms_it() {
        let mut calls = Vec::new();
        let result = install_user_with(
            |args| {
                calls.push(args.iter().map(|arg| (*arg).to_owned()).collect::<Vec<_>>());
                Ok(if calls.len() == 2 {
                    MARKETPLACE.to_vec()
                } else {
                    b"[]".to_vec()
                })
            },
            |_| Ok(()),
        );
        assert!(result.is_err());
        assert_eq!(calls.len(), 4);
    }

    #[test]
    fn incomplete_plugin_is_updated_and_verified_without_reinstallation() {
        let mut calls = Vec::new();
        let mut inspected_install = 0;
        install_user_with(
            |args| {
                calls.push(args.iter().map(|arg| (*arg).to_owned()).collect::<Vec<_>>());
                Ok(match calls.len() {
                    1 | 6 | 7 => installed(true),
                    2 | 4 => MARKETPLACE.to_vec(),
                    _ => Vec::new(),
                })
            },
            |path| {
                if path == Path::new("/installed") {
                    inspected_install += 1;
                    if inspected_install == 1 {
                        return Err(io::Error::new(io::ErrorKind::InvalidData, "missing Stop"));
                    }
                }
                Ok(())
            },
        )
        .unwrap();
        assert!(calls.contains(&vec![
            "plugin".into(),
            "marketplace".into(),
            "update".into(),
            MARKETPLACE_NAME.into()
        ]));
        assert!(calls.contains(&vec![
            "plugin".into(),
            "update".into(),
            PLUGIN_ID.into(),
            "--scope".into(),
            "user".into()
        ]));
        assert!(!calls
            .iter()
            .any(|args| args.contains(&"uninstall".to_owned())));
    }

    #[test]
    fn incomplete_plugin_is_reinstalled_only_when_update_does_not_repair_it() {
        let mut calls = Vec::new();
        let mut inspected_install = 0;
        install_user_with(
            |args| {
                calls.push(args.iter().map(|arg| (*arg).to_owned()).collect::<Vec<_>>());
                Ok(match calls.len() {
                    1 | 6 | 9 => installed(true),
                    2 | 4 => MARKETPLACE.to_vec(),
                    _ => Vec::new(),
                })
            },
            |path| {
                if path == Path::new("/installed") {
                    inspected_install += 1;
                    if inspected_install < 3 {
                        return Err(io::Error::new(io::ErrorKind::InvalidData, "missing Stop"));
                    }
                }
                Ok(())
            },
        )
        .unwrap();
        assert!(calls.contains(&vec![
            "plugin".into(),
            "uninstall".into(),
            PLUGIN_ID.into(),
            "--scope".into(),
            "user".into()
        ]));
        assert!(calls.contains(&vec![
            "plugin".into(),
            "install".into(),
            PLUGIN_ID.into(),
            "--scope".into(),
            "user".into()
        ]));
    }

    #[test]
    fn invalid_marketplace_source_does_not_remove_an_existing_plugin() {
        let mut calls = Vec::new();
        let result = install_user_with(
            |args| {
                calls.push(args.iter().map(|arg| (*arg).to_owned()).collect::<Vec<_>>());
                Ok(match calls.len() {
                    1 => installed(true),
                    2 | 4 => MARKETPLACE.to_vec(),
                    _ => Vec::new(),
                })
            },
            |_| Err(io::Error::new(io::ErrorKind::InvalidData, "missing Stop")),
        );
        assert!(result.is_err());
        assert!(!calls
            .iter()
            .any(|args| args.contains(&"uninstall".to_owned())));
    }
}
