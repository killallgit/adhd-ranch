use std::io;
use std::sync::Arc;

use adhd_ranch_domain::agents::hooks::{HookFiring, HookWiring};
use adhd_ranch_storage::HookHistory;

use crate::SettingsProvider;

/// Where the ranch and Claude's plugin meet. Settled once, at startup.
pub struct HookPaths {
    pub settings_file: String,
    pub client_bin: String,
    pub socket_path: String,
}

/// Answers "is any of this working?" for one agent.
///
/// Every part of the hook path fails silently by design — the client says nothing
/// when it cannot deliver, and an absent or disabled plugin looks like an idle
/// machine. This window exposes both Claude's plugin listing and observed firings.
pub struct AgentDebug {
    plugin_enabled: Arc<dyn Fn() -> io::Result<bool> + Send + Sync>,
    history: Arc<dyn HookHistory>,
    settings: SettingsProvider,
    paths: HookPaths,
}

impl AgentDebug {
    pub fn new(
        plugin_enabled: Arc<dyn Fn() -> io::Result<bool> + Send + Sync>,
        history: Arc<dyn HookHistory>,
        settings: SettingsProvider,
        paths: HookPaths,
    ) -> Self {
        Self {
            plugin_enabled,
            history,
            settings,
            paths,
        }
    }

    pub fn wiring(&self) -> HookWiring {
        HookWiring {
            agent: "Claude Code".to_string(),
            settings_file: self.paths.settings_file.clone(),
            client_bin: self.paths.client_bin.clone(),
            socket_path: self.paths.socket_path.clone(),
            installed: self.installed(),
            enabled: self.settings.get().agents.enabled,
        }
    }

    pub fn recent_firings(&self) -> Vec<HookFiring> {
        self.history.recent()
    }

    /// A failed plugin-list read is not proof of installation. Log the reason;
    /// the install instructions remain available from the tray either way.
    fn installed(&self) -> bool {
        match (self.plugin_enabled)() {
            Ok(installed) => installed,
            Err(e) => {
                log::warn!("Claude Code plugin: cannot read status: {e}");
                false
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::io;

    use super::*;
    use adhd_ranch_domain::agents::hooks::HookAction;
    use adhd_ranch_domain::{AgentsConfig, Settings};

    struct StubHistory(Vec<HookFiring>);

    impl HookHistory for StubHistory {
        fn recent(&self) -> Vec<HookFiring> {
            self.0.clone()
        }
    }

    fn debug_with(installed: io::Result<bool>, enabled: bool) -> AgentDebug {
        let installed = installed.map_err(|error| error.kind());
        let settings = Settings {
            agents: AgentsConfig { enabled },
            ..Settings::default()
        };
        AgentDebug::new(
            Arc::new(move || match installed {
                Ok(value) => Ok(value),
                Err(kind) => Err(io::Error::new(kind, "unreadable")),
            }),
            Arc::new(StubHistory(vec![HookFiring {
                at: "2026-01-01T00:00:00Z".into(),
                action: HookAction::Working,
                session_id: Some("abc".into()),
                pen: Some("app".into()),
                changed: true,
            }])),
            Arc::new(move || settings.clone()),
            HookPaths {
                settings_file: "/home/settings.json".into(),
                client_bin: "/app/adhd-ranch-hook".into(),
                socket_path: "/home/agent-hooks.sock".into(),
            },
        )
    }

    #[test]
    fn wiring_reports_the_stable_client_and_socket_paths() {
        let wiring = debug_with(Ok(true), true).wiring();

        assert_eq!(wiring.client_bin, "/app/adhd-ranch-hook");
        assert_eq!(wiring.socket_path, "/home/agent-hooks.sock");
        assert!(wiring.installed);
        assert!(wiring.enabled);
    }

    #[test]
    fn plugin_status_that_cannot_be_read_does_not_claim_installation() {
        assert!(
            !debug_with(Err(io::Error::other("boom")), true)
                .wiring()
                .installed
        );
    }

    #[test]
    fn disabled_agents_are_reported_as_disabled() {
        assert!(!debug_with(Ok(true), false).wiring().enabled);
    }

    #[test]
    fn firings_are_passed_through_as_the_journal_kept_them() {
        let firings = debug_with(Ok(true), true).recent_firings();

        assert_eq!(firings.len(), 1);
        assert_eq!(firings[0].session_id.as_deref(), Some("abc"));
    }
}
