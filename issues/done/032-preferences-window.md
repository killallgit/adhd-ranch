# 032 — Preferences window

## Parent PRD

PRD.md §FR7 (configuration)

## What to build

Native Tauri webview window opened via Tray "Settings…" item (no submenu) or `Cmd-,`.
Window is created on demand, destroyed on close, recreated on next open (not pre-configured in `tauri.conf.json`).

### Sections

| Section | Controls |
|---------|---------|
| General | Max focuses (1–10), Max tasks per focus (1–20) |
| Displays | Per-monitor toggle (moved from tray into Preferences window) |
| Widget | Always on Top, Confirm Before Delete |
| Alerts | System Notifications |
| Debug | Debug Overlay off by default; toggled here (ephemeral — emits `debug-overlay-toggle`, not persisted) |

### Backend

- `SettingsPathState(PathBuf)` in `app/mod.rs` — managed state for settings file path
- `get_settings` command — reads `SettingsState`, returns `Settings`
- `update_settings(settings: Settings)` command:
  1. Writes `SettingsState`
  2. Persists via `write_settings`
  3. Applies `always_on_top` to all `overlay-N` windows
  4. Rebuilds tray menu

### Frontend

- `settings.html` + `src/settings.tsx` — entry point (mirrors `new-focus` pattern)
- `src/types/settings.ts` — TypeScript mirror of `Settings` struct
- `src/api/settings.ts` — `getSettings()` + `updateSettings(settings)`
- `src/hooks/useSettingsWindow.ts` — loads settings, calls `updateSettings` on each change
- `src/components/SettingsWindow.tsx` — sections + row-toggle layout

### Tray + menu

- `tray.rs` — single "Settings…" item (no submenu) that opens Preferences window on demand
- `menu.rs` — "Preferences…" with `Cmd-,` in app menu
- `vite.config.ts` — `settings` Rollup input

## Completion promise

`Cmd-,` or Tray "Settings…" opens a Preferences window where all settings (including display toggles) are editable. Changes persist immediately. Window is ephemeral — destroyed on close.

## Acceptance criteria

- [ ] `Cmd-,` opens Preferences window
- [ ] Tray "Settings…" (single item, no submenu) opens Preferences window
- [ ] All sections render with correct current values, including Displays (per-monitor toggles)
- [ ] Changing caps, widget, or alerts persists to `settings.yaml` immediately
- [ ] `always_on_top` change takes effect on overlay windows without restart
- [ ] Debug overlay is off by default; toggle emits `debug-overlay-toggle` and is ephemeral
- [ ] Closing Preferences destroys the window; reopening recreates it
- [ ] `task check` green

## Blocked by

None
