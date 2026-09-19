use std::sync::Arc;

use adhd_ranch_domain::agents::hooks::{HookFiring, HookWiring};
use adhd_ranch_storage::{AgentHooks, HookHistory};

use crate::SettingsProvider;

/// Where the ranch and an agent were told to meet. Settled once, at startup.
pub struct HookPaths {
    pub settings_file: String,
    pub client_bin: String,
    pub socket_path: String,
}

/// Answers "is any of this working?" for one agent.
///
/// Every part of the hook path fails silently by design — the client says nothing
/// when it cannot deliver, and a wrong path in a settings file looks exactly like an
/// idle machine. This is the one place that makes those distinguishable.
pub struct AgentDebug {
    hooks: Arc<dyn AgentHooks + Send + Sync>,
    history: Arc<dyn HookHistory>,
    settings: SettingsProvider,
    paths: HookPaths,
}

impl AgentDebug {
    pub fn new(
        hooks: Arc<dyn AgentHooks + Send + Sync>,
        history: Arc<dyn HookHistory>,
        settings: SettingsProvider,
        paths: HookPaths,
    ) -> Self {
        Self {
            hooks,
            history,
            settings,
            paths,
        }
    }

    pub fn wiring(&self) -> HookWiring {
        HookWiring {
            agent: self.hooks.agent().to_string(),
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

    /// An unreadable settings file is reported as "not installed", which is what it
    /// means for the ranch: nothing there is going to call us. The log carries the
    /// reason, because the window has no room for one.
    fn installed(&self) -> bool {
        match self.hooks.installed() {
            Ok(installed) => installed,
            Err(e) => {
                log::warn!("{} hooks: cannot read settings: {e}", self.hooks.agent());
                false
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::io;

    use adhd_ranch_domain::agents::hooks::HookAction;
    use adhd_ranch_domain::{AgentsConfig, Settings};
    use adhd_ranch_storage::HookOutcome;

    use super::*;

    struct StubHooks(io::Result<bool>);

    impl AgentHooks for StubHooks {
        fn agent(&self) -> &'static str {
            "Stub"
        }
        fn install(&self) -> io::Result<HookOutcome> {
            Ok(HookOutcome::AlreadyDone)
        }
        fn uninstall(&self) -> io::Result<HookOutcome> {
            Ok(HookOutcome::AlreadyDone)
        }
        fn installed(&self) -> io::Result<bool> {
            match &self.0 {
                Ok(installed) => Ok(*installed),
                Err(e) => Err(io::Error::new(e.kind(), "unreadable")),
            }
        }
    }

    struct StubHistory(Vec<HookFiring>);

    impl HookHistory for StubHistory {
        fn recent(&self) -> Vec<HookFiring> {
            self.0.clone()
        }
    }

    fn debug_with(installed: io::Result<bool>, enabled: bool) -> AgentDebug {
        let settings = Settings {
            agents: AgentsConfig { enabled },
            ..Settings::default()
        };
        AgentDebug::new(
            Arc::new(StubHooks(installed)),
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
    fn wiring_reports_the_paths_the_hooks_were_installed_with() {
        let wiring = debug_with(Ok(true), true).wiring();

        assert_eq!(wiring.client_bin, "/app/adhd-ranch-hook");
        assert_eq!(wiring.socket_path, "/home/agent-hooks.sock");
        assert!(wiring.installed);
        assert!(wiring.enabled);
    }

    #[test]
    fn settings_that_cannot_be_read_mean_nothing_is_installed() {
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
