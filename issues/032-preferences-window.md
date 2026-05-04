# 032 — Preferences window

## Parent PRD

PRD.md §FR7 (configuration)

## What to build

Native Tauri webview window opened via Tray → Settings → Open Preferences… and `Cmd-,`.

### Sections

| Section | Controls |
|---------|---------|
| General | Max focuses (1–10), Max tasks per focus (1–20) |
| Widget | Always on Top, Confirm Before Delete |
| Alerts | System Notifications |
| Debug | Debug Overlay (ephemeral — emits `debug-overlay-toggle`, not persisted) |

Displays stay in the tray (complex, per-monitor).

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

- `tray.rs` — "Open Preferences…" item at top of Settings submenu
- `menu.rs` — "Preferences…" with `Cmd-,` in app menu
- `tauri.conf.json` — `settings` window entry
- `vite.config.ts` — `settings` Rollup input

## Completion promise

`Cmd-,` or Tray → Settings → Open Preferences opens a window where all non-display settings are editable. Changes persist immediately.

## Acceptance criteria

- [ ] `Cmd-,` opens Preferences window
- [ ] Tray → Settings → Open Preferences… opens Preferences window
- [ ] All four sections render with correct current values
- [ ] Changing caps, widget, or alerts persists to `settings.yaml` immediately
- [ ] `always_on_top` change takes effect on overlay windows without restart
- [ ] Debug overlay toggle emits `debug-overlay-toggle` and is ephemeral
- [ ] `task check` green

## Blocked by

None
