# 010 — Regular Mac app skeleton (drop tray, drop accessory policy)

## Parent PRD

`PRD.md`, `docs/decisions/v1.1-floating-window.md` (D1, D4, D6, D7, D8, D10, D11, D12)

## What to build

Flip the app from menubar-tray (Accessory) to a regular Mac app. This slice is **plumbing only** — no UI changes are user-visible beyond the Dock icon appearing. The next slice wires the app menu and titlebar.

- `src-tauri/src/app/mod.rs`:
  - Remove `app.set_activation_policy(tauri::ActivationPolicy::Accessory)`.
  - Remove `tray::install(app.handle())?` and the `mod tray;` declaration.
  - Remove the `WindowEvent::Focused(false) → window.hide()` block.
  - Add `RunEvent::Reopen` handler on macOS that shows + focuses the main window.
- `src-tauri/src/app/tray.rs` — delete.
- `src-tauri/src/app/window_level.rs` — delete.
- `src-tauri/src/app/window_always_on_top.rs` (new) — pure FFI helper: `apply(window: &WebviewWindow, on: bool)`. macOS-only via objc2 `setLevel:`. `on=true` → `kCGFloatingWindowLevel` (3); `on=false` → `NSNormalWindowLevel` (0). No-op stub on non-macOS.
- `src-tauri/src/app/window_autosave.rs` (new) — `apply(window: &WebviewWindow, name: &str)` calls `setFrameAutosaveName:` on the underlying NSWindow. macOS-only.
- `tauri.conf.json`:
  - `width: 400`, `height: 600`.
  - `resizable: true`, `minWidth: 320`, `minHeight: 400`.
  - Remove `alwaysOnTop` (now runtime-controlled per-launch from settings).
  - Remove `skipTaskbar` (regular app belongs in Dock).
  - Keep `decorations: false`, `transparent: true`, `macOSPrivateApi: true`, `visible: false`.
- `crates/domain/src/settings.rs`:
  - Remove `WindowLevel`, `parse_window_level`, the four window-level tests.
  - `Widget` becomes `{ always_on_top: bool }` (default `false`).
  - Add yaml parser branch: `("widget", "always_on_top") → settings.widget.always_on_top = parse_bool(value)`.
  - Tests: defaults to `false`; `widget:\n  always_on_top: true\n` parses to `true`; invalid value falls back to default.
- `crates/commands/src/caps.rs` (and any other reader of `widget.window_level`) — update to `widget.always_on_top`.
- App setup: read `settings.widget.always_on_top`, call `window_always_on_top::apply(&window, on)` once at startup, then `window_autosave::apply(&window, "adhd-ranch-main")`, then `window.show()`.

## Completion promise

On `main`, launching Adhd Ranch shows a Dock icon and a 400×600 resizable window with `decorations: false`; closing the window via Cmd-W keeps the app running with a bright Dock icon, and clicking the Dock icon brings the window back; `widget.always_on_top: true` in `settings.yaml` is honored at launch.

## Acceptance criteria

- [ ] App launches with a Dock icon (Activity Monitor confirms `Regular` activation).
- [ ] No tray icon appears in the menubar.
- [ ] Window opens at 400×600, resizable, refusing to go below 320×400.
- [ ] Window position + size persist across launches via `setFrameAutosaveName:` (verify with `defaults read com.adhd-ranch.app | grep -i frame`).
- [ ] `settings.yaml` with `widget:\n  always_on_top: true\n` → window stays above other apps after launch.
- [ ] `settings.yaml` with `always_on_top: false` (or missing) → window behaves as a normal window.
- [ ] Cmd-W hides the window; Dock icon stays bright; clicking it re-shows the window.
- [ ] Cmd-Q quits the app.
- [ ] No `WindowLevel` enum remains in `crates/domain/`.
- [ ] No `tray.rs` remains in `src-tauri/src/app/`.
- [ ] No global mutable state introduced; all helpers operate on `&WebviewWindow`.
- [ ] `task check` green.

## Blocked by

- None. Supersedes the previous 010 (popover-Spaces) which was based on the now-obsolete tray-popover model.

## User stories addressed

- US1 (always-visible window — now via floating regular app rather than tray popover)
- US2 (zero-friction mid-session capture — Dock click brings window back, app menu offers `Show Ranch`)
