# 027 — Confirm-delete setting for focuses

## Context

Confirm dialog was removed from the "Delete" tray menu item (was blocking and disruptive).
Direct delete is now in place as of the 024 display-subsystem work.

## What to build

Add a boolean setting `confirm_delete` (default: `true`) to `Settings` / `settings.yaml`.

When `confirm_delete: true`, show the native dialog before deleting.
When `confirm_delete: false`, delete immediately.

Wire the setting through:
- `adhd_ranch_domain::Settings` — add field
- `app/tray.rs::handle_delete` — branch on `settings.confirm_delete`
- `issues/026-settings-preferences.md` — expose toggle in the settings UI

## Acceptance criteria

- [ ] `Settings::confirm_delete` field with default `true`
- [ ] `confirm_delete: false` in settings.yaml skips the dialog
- [ ] Setting surfaced in preferences UI (see issue 026)
- [ ] `task check` green

## Blocked by

026 (settings preferences UI)
