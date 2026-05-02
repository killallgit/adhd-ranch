# Changelog

All notable changes to adhd-ranch. Follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

---

## [Unreleased]

## [1.1.0] — 2026-05-01

Regular Mac app pivot. Replaces the tray-popover model with a draggable floating window, Dock icon, and standard app menu.

### Added
- Custom titlebar with drag region and close `×` button (hides window, keeps app running)
- App menu: `Adhd Ranch` / `File` / `Edit` / `Window` submenus
- `Window > Always on Top` checkbox — toggles window level and persists to `settings.yaml`
- `Window > Show Ranch` — re-shows and focuses the window
- Dock icon and `RunEvent::Reopen` handler (clicking Dock icon reopens window)
- `widget.always_on_top: bool` setting in `settings.yaml` (replaces `window_level` enum)
- Window position + size autosaved via `setFrameAutosaveName:` across launches
- `Settings::to_yaml()` + `storage::write_settings` for atomic settings persistence
- `File > Close` and `Cmd-W` hide the window; `Cmd-Q` quits

### Changed
- Window size: 400×600, resizable, minimum 320×400 (was 360×480, fixed)
- Activation policy: `Regular` (Dock icon) instead of `Accessory` (hidden from Dock)
- `WindowLevel` enum replaced with `always_on_top: bool`

### Removed
- Tray icon and `tray.rs`
- `window_level.rs` and `WindowLevel` / `parse_window_level` from domain
- `tauri-plugin-positioner` dependency
- `WindowEvent::Focused(false) → hide` auto-hide on focus loss
- `widget.window_level` settings key

---

## [1.0.0] — 2026-04 (tray-popover baseline)

### Added
- Tray icon toggles borderless popover window
- Focus CRUD: create, delete, task add/clear from widget
- Proposal queue: `/checkpoint` skill enqueues proposals; accept/reject from widget
- Decisions audit log (append-only `.jsonl`)
- Cap monitoring: max focuses + max tasks per focus with macOS notifications
- Edit proposal modal with focus-retarget support
- Localhost HTTP API (`/health`, `/focuses`, `/proposals`, `/caps`)
- Live file-watcher refresh (debounced, no polling)
- `settings.yaml`: caps, alerts, widget config
- `.app` + `.dmg` packaging; tag-driven GitHub releases
- CI: lint + typecheck + tests on push

