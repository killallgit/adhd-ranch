# 014 — New Focus from tray

## Parent PRD

`PRD.md` §FR4 (Menu bar item — "+ New Focus")

## What to build

Add a "+ New Focus" menu item to the tray NSMenu that opens a small, dedicated Tauri webview window containing the existing `NewFocusForm` component. On save the focus is created and the window closes; the pig spawns automatically via the file watcher.

- **Tray menu change** (`src-tauri/src/app/tray.rs`):
  - Prepend a `MenuItem` labelled "+ New Focus" above the focus list, separated by a `Separator`.
  - On click: show the new-focus window (create if not yet created, show+focus if hidden).
- **New-focus window** (`src-tauri/tauri.conf.json` + setup):
  - Second Tauri webview window, label `"new-focus"`.
  - Small fixed size: `320 × 180`, not resizable, `decorations: false`, `transparent: true`.
  - Positioned near the tray icon (use `tauri-plugin-positioner` if already present, else center screen).
  - `visible: false` at launch; shown on demand, hidden (not destroyed) after save or cancel.
  - Renders a dedicated React route/component — reuse the existing `NewFocusForm` component with a submit handler that calls `create_focus` then hides the window.
- **Frontend** (`src/components/NewFocusWindow.tsx`, new):
  - Thin wrapper around `NewFocusForm` that calls `getCurrentWindow().hide()` after successful create.
  - Mounts at `src/new-focus.tsx` (separate entry point so the main overlay bundle stays lean).
  - Separate `new-focus.html` entry point in `tauri.conf.json`.
- **No duplicate logic** — `create_focus` command unchanged; window just calls it.

## Completion promise

On `main`, clicking "+ New Focus" in the tray menu opens a small form window; filling in a title and submitting creates the focus (pig spawns on the overlay within 1s) and closes the form.

## Acceptance criteria

- [ ] "+ New Focus" item appears at the top of the tray menu.
- [ ] Clicking it opens the new-focus window (small, no traffic lights).
- [ ] Submitting a title creates the focus via the existing `create_focus` command.
- [ ] The window closes (hides) after successful submit.
- [ ] Cancelling (Escape or a Cancel button) hides the window without creating anything.
- [ ] A new pig appears on the overlay within 1s of focus creation.
- [ ] Submitting with an empty title shows an inline validation error (reuses existing `NewFocusForm` behaviour).
- [ ] Second click on "+ New Focus" while window is already open brings it to front rather than creating a duplicate.
- [ ] `task check` green.

## Blocked by

- `issues/013-tray-icon-focus-list.md`

## User stories addressed

- US4 (add a Focus via the menu)
