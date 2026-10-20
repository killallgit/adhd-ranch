# 031 — Notification source settings registry

## Owning specification

`specs/003-ranch-today/spec.md` — FR-008

## What to build

Make notification settings use the registered `NotificationSource` list instead of hardcoded UI rows.

Issue 029 introduced `NotificationSource`, `NotificationSettings`, and `all_sources()` in Rust. The current Preferences window already exposes notification toggles, but the source list is duplicated in TypeScript as `NOTIFICATION_SOURCES`. That means adding a new source still requires remembering to update frontend UI by hand.

This slice should make Preferences render notification toggles from one source registry exported by the app.

### Target shape

- Add a Tauri command such as `list_notification_sources`.
- Return stable source descriptors:

  ```ts
  interface NotificationSourceDescriptor {
    readonly key: string;
    readonly label: string;
  }
  ```

- The command iterates `adhd_ranch_domain::all_sources()`.
- `SettingsWindow` reads descriptors through an injected reader or hook, then renders one toggle per descriptor.
- Toggle state still reads/writes `settings.notifications.sources[key]`.
- Existing settings persistence remains unchanged: flat YAML under `notifications:`.

### Out of scope

- Reintroducing tray submenus. The current app uses `Settings...` to open Preferences.
- New notification sources.
- User-visible write-failure notifications.

## Completion promise

Notification toggles in Preferences are driven by the Rust `NotificationSource` registry, so adding a new source does not require editing the settings UI source list.

## Acceptance criteria

- [x] A command/API returns notification source descriptors from `all_sources()`
- [x] `SettingsWindow` no longer contains a hardcoded `NOTIFICATION_SOURCES` list
- [x] "Timer expired", "Too many focuses", and "Too many tasks in a focus" toggles still render with current settings values
- [x] Toggling any source updates `settings.yaml` and takes effect immediately (no restart needed)
- [x] Adding a second or later `NotificationSource` requires no changes to `SettingsWindow`
- [x] Existing Settings sections (General, Widget, Displays, Debug) are unaffected
- [x] `task check` green

## Blocked by

050 — settings update workflow and runtime freshness. 029 and 032 are done.

## Validation

Automated acceptance passed: the registry projection, API/hook wiring, current labels and values, and arbitrary-descriptor rendering/toggling are covered by focused tests. Existing settings workflow, flat YAML round-trip, and latest-settings notification tests passed, covering persistence and immediate runtime application without changing the settings schema.

`task check` and `task check:windows` passed. Implementation review returned PASS; the rules review was clean.

Native Preferences interaction, actual settings-file changes from native toggles, and OS notification delivery were not observed. These checks establish automated behavior, not native GUI verification.
