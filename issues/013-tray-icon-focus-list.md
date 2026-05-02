# 013 — Tray icon + live focus list

## Parent PRD

`PRD.md` §FR4 (Menu bar item)

## What to build

Reinstate a macOS menu bar tray icon (removed in v1.1 per D1, now repurposed as the ranch control panel). The icon opens a native NSMenu that lists current Focuses and provides a Quit item. The menu rebuilds live whenever focuses change.

- **Tray icon** (`src-tauri/src/app/tray.rs`, new):
  - Install via `TrayIconBuilder` in `app::run()` setup.
  - Left- or right-click opens the NSMenu (standard macOS tray behavior).
  - Normal icon: the app's pig icon (or a placeholder until real assets exist).
  - Over-cap icon: swapped to a red-badged variant via `TrayIcon::set_icon` on the `focuses-changed` + cap-check path.
- **NSMenu contents** (built with `tauri::menu::MenuBuilder`):

  ```text
  [Focus title 1]   ← MenuItemBuilder, disabled (no action yet — submenu comes in 014)
  [Focus title 2]
  …
  ─
  Quit              ← calls app.exit(0)
  ```

  Empty-state: show a single greyed-out "No focuses yet" item.
- **Live rebuild**:
  - Subscribe to the existing `FOCUSES_CHANGED_EVENT` from within setup.
  - On each event, re-read the focus store and call `tray.set_menu(Some(new_menu))`.
  - Over-cap check runs the existing `CapEvaluator` to decide whether to swap the icon.
- **No tray popover window** — the overlay window launched at startup is the pig canvas; the tray is menu-only.

## Completion promise

On `main`, clicking the macOS menu bar pig icon shows a native dropdown listing all current Focus titles (updating live within 1s of any focus file change), with a Quit item at the bottom; the icon shows a red badge while any cap is exceeded.

## Acceptance criteria

- [ ] Tray icon appears in menu bar at app launch.
- [ ] Clicking the icon opens a native macOS menu.
- [ ] Menu lists one item per Focus using `focus.title`.
- [ ] Empty state: single greyed-out "No focuses yet" item.
- [ ] After adding or deleting a focus (via file or HTTP), the menu updates within 1s without restarting the app.
- [ ] When over-cap, tray icon swaps to a red-badged variant.
- [ ] "Quit" item exits the app cleanly.
- [ ] No static mut, no global singletons, no module-level mutable state. TrayIcon handle managed via `app.manage()` or passed through setup closure.
- [ ] `task check` green.

## Blocked by

None — can start immediately.

## User stories addressed

- US3 (list of focuses accessible without opening anything else)
- US6 (over-cap badge)
