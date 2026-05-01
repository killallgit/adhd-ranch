# 011 — Nonactivating panel so popover doesn't steal focus

## Parent PRD

`PRD.md`

## What to build

The popover is a regular `NSWindow`. Clicking it activates the Adhd Ranch app: the Dock bounces, focus is stolen from whichever app the user is in (typically the terminal where they just ran `/checkpoint`). A real menubar widget should accept clicks without becoming the key window.

- Convert the main window's underlying `NSWindow` to behave as an `NSPanel` with `NSWindowStyleMaskNonactivatingPanel`. Two viable approaches:
  1. Toggle `setStyleMask:` on the existing NSWindow to include `NSWindowStyleMaskNonactivatingPanel` (simpler; `NSWindow` accepts the mask).
  2. Replace class via `object_setClass` to `NSPanel`, then set the mask (more invasive).
  Pick approach 1 unless empirical testing shows the underlying activation behavior persists.
- Drop `let _ = window.set_focus();` calls in `src-tauri/src/app/tray.rs` — the show path no longer needs to claim key-window status.
- Verify the existing focus-loss auto-hide (`mod.rs:118-124`) still fires for nonactivating panels (it should; `WindowEvent::Focused(false)` fires when key/main status leaves).
- New module `src-tauri/src/app/panel_style.rs` owns the FFI; `mod.rs` calls it before `tray::install`.

## Completion promise

On `main`, running `/checkpoint` in a terminal and clicking ✓ in the popover leaves the terminal as the active app: no Dock bounce, no menubar swap, terminal cursor still blinks; the popover still auto-hides when the user clicks anywhere outside it.

## Acceptance criteria

- [ ] `panel_style::apply` sets `NSWindowStyleMaskNonactivatingPanel` on the main NSWindow on macOS.
- [ ] No-op stub on non-macOS targets.
- [ ] `tray.rs` no longer calls `set_focus()` on show; `set_position` + `show` are sufficient.
- [ ] Manual: focus a terminal, click tray icon → popover appears, terminal remains the active app (menubar still shows terminal's name).
- [ ] Manual: with popover open, click into terminal → popover auto-hides via existing `WindowEvent::Focused(false)` path.
- [ ] Manual: with popover open, click anywhere on the desktop → popover auto-hides.
- [ ] No regression on slice 010 — popover still shows across Spaces and over fullscreen apps.
- [ ] `task check` green.

## Blocked by

- Blocked by `issues/010-popover-spaces-and-fullscreen.md` (both touch NSWindow internals; sequencing keeps each diff small and reviewable).

## User stories addressed

- User story 2
