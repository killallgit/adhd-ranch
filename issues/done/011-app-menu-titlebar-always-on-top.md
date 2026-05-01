# 011 — App menu + custom titlebar + Always-on-top toggle

## Parent PRD

`PRD.md`, `docs/decisions/v1.1-floating-window.md` (D2, D3, D5, D6, D9)

## What to build

Build the user-visible chrome on top of the regular-app skeleton from slice 010.

- **Custom titlebar** (`src/components/Titlebar.tsx`):
  - Thin bar (height ~28px) at top of main window.
  - Drag region: most of the bar gets `data-tauri-drag-region`; pointer events on inner controls do not.
  - Close button on the right: calls `window.hide()` via `@tauri-apps/api/window`.
  - No traffic lights, no title text.
  - Mounted as the first child of `App.tsx` above the existing focuses + tray UI.
- **App menu** (`src-tauri/src/app/menu.rs`):
  - `Adhd Ranch` submenu: `About Adhd Ranch`, separator, `Quit Cmd+Q`.
  - `File` submenu: `Close Cmd+W` (calls `window.hide()`).
  - `Edit` submenu: Tauri default (undo/redo/cut/copy/paste/select all).
  - `Window` submenu:
    - `Always on Top` checkbox item (no shortcut). Toggling flips `widget.always_on_top` in `settings.yaml` (atomic write, same pattern as focus.md), then calls `window_always_on_top::apply(&window, new_value)` immediately.
    - separator
    - `Show Ranch` item: calls `window.show().focus()`.
  - No Help menu.
- **Settings yaml writer** (`crates/storage/` or `crates/commands/`):
  - Pure helper: load current `Settings`, mutate `widget.always_on_top`, serialize back to yaml, atomic write to `~/.adhd-ranch/settings.yaml`.
  - Preserves any other keys (caps, alerts) the user has set. (Sketch: parse with current parser, keep extras in a side-map; or rewrite the whole file from a `Settings` value if extras are not yet preserved — call this out in the PR.)
- **Wiring**:
  - `tauri::menu::CheckMenuItem` for `Always on Top`. On startup, set its checked state from loaded `settings.widget.always_on_top`.
  - `on_menu_event` dispatches `always-on-top`, `show-ranch`, `close-window` to handlers.
  - `RunEvent::Reopen` from slice 010 already shows the window — `Show Ranch` reuses the same call.

## Completion promise

On `main`, launching Adhd Ranch presents a draggable 400×600 window with a thin custom titlebar (no traffic lights), a standard Mac app menu, and a `Window > Always on Top` checkbox that, when toggled, both pins the window and writes `widget.always_on_top` to `~/.adhd-ranch/settings.yaml`; closing via the titlebar `×`, Cmd-W, or `File > Close` hides the window, and `Window > Show Ranch` (or clicking the Dock icon) brings it back.

## Acceptance criteria

- [ ] Titlebar component renders at the top of the window with drag region and close `×`.
- [ ] Dragging the titlebar moves the window; clicking `×` hides it.
- [ ] App menu items appear in the macOS menu bar when the app is frontmost: `Adhd Ranch`, `File`, `Edit`, `Window`.
- [ ] `Window > Always on Top` is a checkbox; its state matches `widget.always_on_top` from `settings.yaml` at launch.
- [ ] Toggling `Always on Top` ON pins the window above other apps and writes `widget.always_on_top: true` to `~/.adhd-ranch/settings.yaml`; toggling OFF unpins and writes `false`.
- [ ] After toggling, opening `~/.adhd-ranch/settings.yaml` shows the change persisted (atomic write — no partial-file corruption).
- [ ] Other keys in `settings.yaml` (caps, alerts) are preserved across the toggle.
- [ ] `Window > Show Ranch` shows the window when hidden; brings it forward when already visible.
- [ ] `File > Close` and Cmd-W hide the window without quitting.
- [ ] `Adhd Ranch > Quit` and Cmd-Q quit the app.
- [ ] Edit menu (cut/copy/paste/undo) works in proposal-edit textareas.
- [ ] No global mutable state introduced; toggle handler reads/writes through injected services.
- [ ] `task check` green.

## Blocked by

- Blocked by `issues/010-...md` (regular-app skeleton must land first — this slice depends on `Regular` activation policy and the `window_always_on_top::apply` helper).

## User stories addressed

- US1 (always-visible widget — now controllable via `Always on Top`)
- US2 (zero-friction mid-session capture — `Show Ranch` and Dock click both reopen)
