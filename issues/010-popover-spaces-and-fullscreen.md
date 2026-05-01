# 010 — Popover follows user across Spaces + fullscreen apps

## Parent PRD

`PRD.md`

## What to build

The tray popover already floats above normal windows via `setLevel`, but it does not follow the user across macOS Spaces and is hidden when a fullscreen app is frontmost. A real menubar widget must be visible on whichever Space the user is on, and must layer over fullscreen apps.

- Set `NSWindowCollectionBehavior` on the main webview window: `canJoinAllSpaces | fullScreenAuxiliary | stationary`.
- Apply alongside `window_level::apply` in `src-tauri/src/app/mod.rs` setup, before the window is shown.
- macOS-only via `objc2 msg_send`; no-op on other targets, matching `window_level.rs`.
- New module `src-tauri/src/app/window_collection.rs` owns the FFI; `mod.rs` calls it.

No new settings keys. Behavior is unconditional — every popover instance gets it.

## Completion promise

On `main`, opening the popover on Space A and switching to Space B keeps the popover available on B; clicking the tray icon while a fullscreen app is frontmost shows the popover above that fullscreen app.

## Acceptance criteria

- [ ] `window_collection::apply` sets `NSWindowCollectionBehavior` flags `canJoinAllSpaces | fullScreenAuxiliary | stationary` on macOS.
- [ ] No-op stub compiles on non-macOS targets.
- [ ] Called from `app/mod.rs` setup, alongside `window_level::apply`, behind `if let Some(window) = app.get_webview_window("main")`.
- [ ] Manual: open popover on Space A, switch to Space B via Mission Control, click tray — popover shows on B.
- [ ] Manual: enter a fullscreen app (e.g. Safari fullscreen), click tray — popover overlays fullscreen.
- [ ] No regression on auto-hide-on-blur (`mod.rs:118-124`); popover still hides when focus leaves it.
- [ ] No global mutable state introduced; helper is a pure function over `&WebviewWindow`.
- [ ] `task check` green.

## Blocked by

- None

## User stories addressed

- User story 1
