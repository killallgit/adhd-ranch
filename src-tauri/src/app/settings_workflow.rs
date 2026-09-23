use adhd_ranch_domain::Settings;
use adhd_ranch_storage::write_settings;
use tauri::{AppHandle, Emitter, Manager, Wry};

use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use super::{
    DisplayConfigState, MonitorsState, AGENT_SESSIONS_CHANGED_EVENT, SETTINGS_CHANGED_EVENT,
};

pub trait SettingsPersistence: Send + Sync {
    fn persist(&self, settings: &Settings) -> Result<(), SettingsWorkflowError>;
}

impl<T> SettingsPersistence for std::sync::Arc<T>
where
    T: SettingsPersistence + ?Sized,
{
    fn persist(&self, settings: &Settings) -> Result<(), SettingsWorkflowError> {
        (**self).persist(settings)
    }
}

pub trait SettingsRuntime: Send + Sync {
    fn current(&self) -> Result<Settings, SettingsWorkflowError>;
    fn commit(&self, settings: Settings) -> Result<(), SettingsWorkflowError>;
}

impl<T> SettingsRuntime for std::sync::Arc<T>
where
    T: SettingsRuntime + ?Sized,
{
    fn current(&self) -> Result<Settings, SettingsWorkflowError> {
        (**self).current()
    }

    fn commit(&self, settings: Settings) -> Result<(), SettingsWorkflowError> {
        (**self).commit(settings)
    }
}

pub trait SettingsEffects: Send + Sync {
    fn apply_widget(&self, settings: &Settings) -> Result<(), SettingsWorkflowError>;
    fn apply_displays(
        &self,
        previous: &Settings,
        next: &Settings,
    ) -> Result<(), SettingsWorkflowError>;
    fn apply_agents(
        &self,
        previous: &Settings,
        next: &Settings,
    ) -> Result<(), SettingsWorkflowError>;
    fn refresh_runtime_consumers(&self, settings: &Settings) -> Result<(), SettingsWorkflowError>;
    fn rebuild_tray(&self) -> Result<(), SettingsWorkflowError>;
}

impl<T> SettingsEffects for std::sync::Arc<T>
where
    T: SettingsEffects + ?Sized,
{
    fn apply_widget(&self, settings: &Settings) -> Result<(), SettingsWorkflowError> {
        (**self).apply_widget(settings)
    }

    fn apply_displays(
        &self,
        previous: &Settings,
        next: &Settings,
    ) -> Result<(), SettingsWorkflowError> {
        (**self).apply_displays(previous, next)
    }

    fn apply_agents(
        &self,
        previous: &Settings,
        next: &Settings,
    ) -> Result<(), SettingsWorkflowError> {
        (**self).apply_agents(previous, next)
    }

    fn refresh_runtime_consumers(&self, settings: &Settings) -> Result<(), SettingsWorkflowError> {
        (**self).refresh_runtime_consumers(settings)
    }

    fn rebuild_tray(&self) -> Result<(), SettingsWorkflowError> {
        (**self).rebuild_tray()
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum SettingsWorkflowError {
    Persist(String),
    Runtime(String),
    Effect(String),
}

pub struct SettingsWorkflow<P, R, E> {
    persistence: P,
    runtime: R,
    effects: E,
}

impl<P, R, E> SettingsWorkflow<P, R, E>
where
    P: SettingsPersistence,
    R: SettingsRuntime,
    E: SettingsEffects,
{
    pub fn new(persistence: P, runtime: R, effects: E) -> Self {
        Self {
            persistence,
            runtime,
            effects,
        }
    }

    pub fn update(&self, next: Settings) -> Result<(), SettingsWorkflowError> {
        let previous = self.runtime.current()?;
        self.persistence.persist(&next)?;
        self.runtime.commit(next.clone())?;
        self.effects.apply_widget(&next)?;
        if previous.displays != next.displays {
            self.effects.apply_displays(&previous, &next)?;
        }
        if previous.agents != next.agents {
            self.effects.apply_agents(&previous, &next)?;
        }
        self.effects.refresh_runtime_consumers(&next)?;
        self.effects.rebuild_tray()?;
        Ok(())
    }
}

/// Both directions matter: turning agents off has to take the hooks back out of
/// Claude Code's settings, not merely stop caring about them.
fn agents_setting_changed(previous: &Settings, next: &Settings) -> bool {
    previous.agents.enabled != next.agents.enabled
}

pub struct FileSettingsPersistence {
    path: PathBuf,
}

impl FileSettingsPersistence {
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }
}

impl SettingsPersistence for FileSettingsPersistence {
    fn persist(&self, settings: &Settings) -> Result<(), SettingsWorkflowError> {
        write_settings(&self.path, settings)
            .map_err(|e| SettingsWorkflowError::Persist(format!("persist settings: {e}")))
    }
}

pub struct SharedSettingsRuntime {
    state: Arc<Mutex<Settings>>,
}

impl SharedSettingsRuntime {
    pub fn new(state: Arc<Mutex<Settings>>) -> Self {
        Self { state }
    }
}

impl SettingsRuntime for SharedSettingsRuntime {
    fn current(&self) -> Result<Settings, SettingsWorkflowError> {
        self.state
            .lock()
            .map(|settings| settings.clone())
            .map_err(|_| SettingsWorkflowError::Runtime("settings lock poisoned".to_string()))
    }

    fn commit(&self, settings: Settings) -> Result<(), SettingsWorkflowError> {
        *self
            .state
            .lock()
            .map_err(|_| SettingsWorkflowError::Runtime("settings lock poisoned".to_string()))? =
            settings;
        Ok(())
    }
}

pub struct TauriSettingsEffects {
    app: AppHandle<Wry>,
}

impl TauriSettingsEffects {
    pub fn new(app: AppHandle<Wry>) -> Self {
        Self { app }
    }
}

impl TauriSettingsEffects {
    /// The hooks are about to be removed, so nothing will ever arrive to say the
    /// sessions the ranch is holding have ended. Anything kept would come back as an
    /// animal for a long-finished session the next time agents are switched on.
    fn forget_live_sessions(&self) {
        if let Some(sessions) = self.app.try_state::<super::LiveSessionsState>() {
            sessions.0.clear();
        }
    }
}

impl SettingsEffects for TauriSettingsEffects {
    fn apply_widget(&self, settings: &Settings) -> Result<(), SettingsWorkflowError> {
        let monitors_count = self
            .app
            .try_state::<MonitorsState>()
            .map(|s| s.0.len())
            .unwrap_or(1);
        for i in 0..monitors_count {
            if let Some(window) = self.app.get_webview_window(&format!("overlay-{i}")) {
                crate::app::window_always_on_top::apply(&window, settings.widget.always_on_top);
            }
        }
        Ok(())
    }

    fn apply_displays(
        &self,
        _previous: &Settings,
        next: &Settings,
    ) -> Result<(), SettingsWorkflowError> {
        if let Some(display_state) = self.app.try_state::<DisplayConfigState>() {
            match display_state.0.lock() {
                Ok(mut config) => *config = next.displays.clone(),
                Err(e) => {
                    return Err(SettingsWorkflowError::Effect(format!(
                        "display config lock poisoned: {e}"
                    )));
                }
            }
        }
        if let Some(display_state) = self.app.try_state::<crate::display::DisplayManagerState>() {
            if let Some(monitors_state) = self.app.try_state::<MonitorsState>() {
                let display_manager = Arc::clone(&display_state.0);
                let monitors = monitors_state.0.clone();
                let config = next.displays.clone();
                let app = self.app.clone();
                self.app
                    .run_on_main_thread(move || {
                        display_manager.apply(&app, &monitors, &config);
                    })
                    .map_err(|e| {
                        SettingsWorkflowError::Effect(format!("run_on_main_thread failed: {e}"))
                    })?;
            }
        }
        Ok(())
    }

    fn apply_agents(
        &self,
        previous: &Settings,
        next: &Settings,
    ) -> Result<(), SettingsWorkflowError> {
        if agents_setting_changed(previous, next) && !next.agents.enabled {
            self.forget_live_sessions();
        }
        // The overlay hears either way: what it is drawing has changed. Claude's
        // plugin remains installed regardless of this preference.
        self.app
            .emit(AGENT_SESSIONS_CHANGED_EVENT, ())
            .map_err(|e| SettingsWorkflowError::Effect(format!("emit agent-sessions-changed: {e}")))
    }

    fn refresh_runtime_consumers(&self, _settings: &Settings) -> Result<(), SettingsWorkflowError> {
        self.app
            .emit(SETTINGS_CHANGED_EVENT, ())
            .map_err(|e| SettingsWorkflowError::Effect(format!("emit settings-changed: {e}")))
    }

    fn rebuild_tray(&self) -> Result<(), SettingsWorkflowError> {
        crate::app::tray::rebuild_tray_menu(&self.app);
        Ok(())
    }
}

pub fn workflow_for_app(
    app: AppHandle<Wry>,
    settings_state: Arc<Mutex<Settings>>,
    settings_path: PathBuf,
) -> SettingsWorkflow<FileSettingsPersistence, SharedSettingsRuntime, TauriSettingsEffects> {
    SettingsWorkflow::new(
        FileSettingsPersistence::new(settings_path),
        SharedSettingsRuntime::new(settings_state),
        TauriSettingsEffects::new(app),
    )
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use adhd_ranch_domain::{AgentsConfig, Caps, Settings};

    use super::{
        agents_setting_changed, SettingsEffects, SettingsPersistence, SettingsRuntime,
        SettingsWorkflow, SettingsWorkflowError,
    };

    struct RecordingPersistence {
        should_fail: bool,
        persisted: Mutex<Vec<Settings>>,
    }

    impl RecordingPersistence {
        fn new() -> Self {
            Self {
                should_fail: false,
                persisted: Mutex::new(Vec::new()),
            }
        }

        fn failing() -> Self {
            Self {
                should_fail: true,
                persisted: Mutex::new(Vec::new()),
            }
        }

        fn persisted(&self) -> Vec<Settings> {
            self.persisted.lock().unwrap().clone()
        }
    }

    impl SettingsPersistence for RecordingPersistence {
        fn persist(&self, settings: &Settings) -> Result<(), SettingsWorkflowError> {
            if self.should_fail {
                return Err(SettingsWorkflowError::Persist("disk full".to_string()));
            }
            self.persisted.lock().unwrap().push(settings.clone());
            Ok(())
        }
    }

    struct RecordingRuntime {
        settings: Mutex<Settings>,
    }

    impl RecordingRuntime {
        fn new(settings: Settings) -> Self {
            Self {
                settings: Mutex::new(settings),
            }
        }

        fn settings(&self) -> Settings {
            self.settings.lock().unwrap().clone()
        }
    }

    impl SettingsRuntime for RecordingRuntime {
        fn current(&self) -> Result<Settings, SettingsWorkflowError> {
            Ok(self.settings())
        }

        fn commit(&self, settings: Settings) -> Result<(), SettingsWorkflowError> {
            *self.settings.lock().unwrap() = settings;
            Ok(())
        }
    }

    #[derive(Default)]
    struct RecordingEffects {
        calls: Mutex<Vec<&'static str>>,
    }

    impl RecordingEffects {
        fn calls(&self) -> Vec<&'static str> {
            self.calls.lock().unwrap().clone()
        }
    }

    impl SettingsEffects for RecordingEffects {
        fn apply_widget(&self, _settings: &Settings) -> Result<(), SettingsWorkflowError> {
            self.calls.lock().unwrap().push("widget");
            Ok(())
        }

        fn apply_displays(
            &self,
            _previous: &Settings,
            _next: &Settings,
        ) -> Result<(), SettingsWorkflowError> {
            self.calls.lock().unwrap().push("displays");
            Ok(())
        }

        fn apply_agents(
            &self,
            _previous: &Settings,
            _next: &Settings,
        ) -> Result<(), SettingsWorkflowError> {
            self.calls.lock().unwrap().push("agents");
            Ok(())
        }

        fn refresh_runtime_consumers(
            &self,
            _settings: &Settings,
        ) -> Result<(), SettingsWorkflowError> {
            self.calls.lock().unwrap().push("runtime");
            Ok(())
        }

        fn rebuild_tray(&self) -> Result<(), SettingsWorkflowError> {
            self.calls.lock().unwrap().push("tray");
            Ok(())
        }
    }

    fn settings_with_max_focuses(max_focuses: usize) -> Settings {
        Settings {
            caps: Caps {
                max_focuses,
                ..Caps::default()
            },
            ..Settings::default()
        }
    }

    fn settings_with_agents(enabled: bool) -> Settings {
        Settings {
            agents: AgentsConfig { enabled },
            ..Settings::default()
        }
    }

    fn settings_with_displays(enabled_indices: Vec<usize>) -> Settings {
        Settings {
            displays: adhd_ranch_domain::DisplayConfig { enabled_indices },
            ..Settings::default()
        }
    }

    #[test]
    fn persist_failure_does_not_commit_runtime_settings_or_apply_effects() {
        let original = settings_with_max_focuses(5);
        let next = settings_with_max_focuses(9);
        let runtime = Arc::new(RecordingRuntime::new(original.clone()));
        let effects = Arc::new(RecordingEffects::default());
        let workflow = SettingsWorkflow::new(
            Arc::new(RecordingPersistence::failing()),
            runtime.clone(),
            effects.clone(),
        );

        let err = workflow.update(next).unwrap_err();

        assert!(matches!(err, SettingsWorkflowError::Persist(_)));
        assert_eq!(runtime.settings(), original);
        assert!(effects.calls().is_empty());
    }

    #[test]
    fn non_display_update_persists_commits_runtime_and_skips_display_reapply() {
        let original = settings_with_max_focuses(5);
        let next = settings_with_max_focuses(9);
        let persistence = Arc::new(RecordingPersistence::new());
        let runtime = Arc::new(RecordingRuntime::new(original));
        let effects = Arc::new(RecordingEffects::default());
        let workflow = SettingsWorkflow::new(persistence.clone(), runtime.clone(), effects.clone());

        workflow.update(next.clone()).unwrap();

        assert_eq!(persistence.persisted(), vec![next.clone()]);
        assert_eq!(runtime.settings(), next);
        assert_eq!(effects.calls(), vec!["widget", "runtime", "tray"]);
    }

    #[test]
    fn display_update_reapplies_displays_between_widget_and_runtime_refresh() {
        let original = settings_with_displays(vec![0]);
        let next = settings_with_displays(vec![0, 1]);
        let persistence = Arc::new(RecordingPersistence::new());
        let runtime = Arc::new(RecordingRuntime::new(original));
        let effects = Arc::new(RecordingEffects::default());
        let workflow = SettingsWorkflow::new(persistence.clone(), runtime.clone(), effects.clone());

        workflow.update(next.clone()).unwrap();

        assert_eq!(persistence.persisted(), vec![next.clone()]);
        assert_eq!(runtime.settings(), next);
        assert_eq!(
            effects.calls(),
            vec!["widget", "displays", "runtime", "tray"]
        );
    }

    #[test]
    fn agents_update_applies_agents_effect_before_runtime_refresh() {
        let persistence = Arc::new(RecordingPersistence::new());
        let runtime = Arc::new(RecordingRuntime::new(settings_with_agents(false)));
        let effects = Arc::new(RecordingEffects::default());
        let workflow = SettingsWorkflow::new(persistence.clone(), runtime.clone(), effects.clone());

        workflow.update(settings_with_agents(true)).unwrap();

        assert_eq!(persistence.persisted(), vec![settings_with_agents(true)]);
        assert_eq!(runtime.settings(), settings_with_agents(true));
        assert_eq!(effects.calls(), vec!["widget", "agents", "runtime", "tray"]);
    }

    #[test]
    fn enabling_agents_changes_the_projection() {
        assert!(agents_setting_changed(
            &settings_with_agents(false),
            &settings_with_agents(true)
        ));
    }

    #[test]
    fn disabling_agents_changes_the_projection() {
        assert!(agents_setting_changed(
            &settings_with_agents(true),
            &settings_with_agents(false)
        ));
    }

    #[test]
    fn leaving_the_agents_setting_alone_needs_no_projection_change() {
        assert!(!agents_setting_changed(
            &settings_with_agents(true),
            &settings_with_agents(true)
        ));
    }
}
