# 050 — Settings update workflow module

## Parent PRD

PRD.md §FR7 (configuration) + CLAUDE.md §Data/view separation

## What to build

Move settings update coordination out of the Tauri command handler and into a dedicated workflow module.

Today `ui_bridge::update_settings` mutates in-memory Settings, persists `settings.yaml`, applies always-on-top behavior, updates display state, reapplies overlays, and rebuilds the tray menu inline. That makes the command handler the place where ordering and side effects live.

There is also a runtime freshness bug hiding behind that shallow module: settings are copied into several long-lived adapters at startup. `SettingsState` updates when Preferences changes, but `Commands::settings`, `CapEvaluator::settings`, and tray rebuild handlers keep their original `Settings` value. That means caps, notification source toggles, and tray badge state can disagree with the newly persisted `settings.yaml` until restart.

### Target shape

Add an app-side workflow module, for example `src-tauri/src/app/settings_workflow.rs`.

The module should expose one high-leverage entry point:

```rust
pub struct SettingsWorkflow { /* injected adapters */ }

impl SettingsWorkflow {
    pub fn update(&self, next: Settings) -> Result<(), SettingsWorkflowError>;
}
```

Adapters should cover:

- in-memory Settings state
- settings file persistence
- overlay always-on-top application
- display config reapply
- cap evaluator / command settings freshness
- tray menu rebuild

The workflow owns ordering. A reasonable initial invariant: persist first, then update memory and apply runtime effects. If implementation chooses a different invariant, document it in tests.

The workflow should make one committed `Settings` value authoritative for runtime consumers. It is acceptable to replace startup-copied `Settings` fields with a shared settings provider/adapter, or to update those adapters as part of the workflow, as long as the result is that Preferences changes take effect immediately without requiring app restart.

## Completion promise

`ui_bridge::update_settings` delegates to a settings workflow module; settings persistence, runtime settings freshness, display effects, cap/notification behavior, and tray rebuild ordering are tested through one interface.

## Acceptance criteria

- [x] `src-tauri/src/app/settings_workflow.rs` exists
- [x] `ui_bridge::update_settings` delegates to the workflow and contains no inline display/tray/window coordination
- [x] The workflow uses injected adapters or small helper traits for side effects
- [x] Runtime consumers that currently receive startup-copied `Settings` observe the updated Settings without restart
- [x] `get_caps` reflects updated cap settings after `update_settings`
- [x] Cap and timer notification source toggles take effect on the next evaluation/tick without restart
- [x] Tray menu rebuild and tray badge calculations use the updated caps
- [x] Tests cover persistence failure, display-change behavior, non-display widget setting behavior, runtime settings freshness, and tray rebuild invocation
- [x] The chosen ordering invariant is documented in test names or code comments
- [x] `task check` green

## Blocked by

None

## User stories addressed

- "When settings change, the app applies one coherent workflow instead of scattering persistence and side effects through the command handler."
- "When I change caps or notification toggles in Preferences, the app behavior changes immediately without restarting."
