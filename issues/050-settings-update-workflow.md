# 050 — Settings update workflow module

## Parent PRD

PRD.md §FR7 (configuration) + CLAUDE.md §Data/view separation

## What to build

Move settings update coordination out of the Tauri command handler and into a dedicated workflow module.

Today `ui_bridge::update_settings` mutates in-memory Settings, persists `settings.yaml`, applies always-on-top behavior, updates display state, reapplies overlays, and rebuilds the tray menu inline. That makes the command handler the place where ordering and side effects live.

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
- tray menu rebuild

The workflow owns ordering. A reasonable initial invariant: persist first, then update memory and apply runtime effects. If implementation chooses a different invariant, document it in tests.

## Completion promise

`ui_bridge::update_settings` delegates to a settings workflow module; settings persistence, in-memory update, display effects, and tray rebuild ordering are tested through one interface.

## Acceptance criteria

- [ ] `src-tauri/src/app/settings_workflow.rs` exists
- [ ] `ui_bridge::update_settings` delegates to the workflow and contains no inline display/tray/window coordination
- [ ] The workflow uses injected adapters or small helper traits for side effects
- [ ] Tests cover persistence failure, display-change behavior, non-display widget setting behavior, and tray rebuild invocation
- [ ] The chosen ordering invariant is documented in test names or code comments
- [ ] `task check` green

## Blocked by

None

## User stories addressed

- "When settings change, the app applies one coherent workflow instead of scattering persistence and side effects through the command handler."
