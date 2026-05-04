# 026 — Settings / Preferences window

## Parent PRD

PRD.md §FR7 (configuration)

## Problem

Settings are accumulating across tray submenus and hardcoded constants (displays, always-on-top, future: keep-still, wrangle keybind, hitbox padding, friction, cap overrides). The tray submenu model doesn't scale. Each new setting either bloats the tray or lives in `settings.yaml` with no UI.

## What to build

### Option A — Settings submenu in tray (recommended first step)

Add a top-level **Settings** item to the tray menu. Move existing settings under it:

```
Settings
  ├── Displays       (existing submenu, moved here)
  └── Window
        └── Always on Top  (existing toggle, moved here)
```

Zero new windows. Ships fast. Existing behaviour unchanged.

### Option B — Native Preferences window (if Option A isn't enough)

`Cmd-,` / Tray → Preferences opens a small Tauri webview window:
- Sections: General / Displays / Pigs
- Form controls for cap values, friction, hitbox padding
- Saves to `settings.yaml` on change; triggers live reload of relevant subsystems

## Completion promise

All user-configurable settings are reachable from a single "Settings" entry in the tray, not scattered across top-level menu items.

## Acceptance criteria (Option A)

- [ ] Tray has a top-level **Settings** item
- [ ] Displays submenu lives under Settings
- [ ] Always on Top lives under Settings → Window
- [ ] All existing behaviour unchanged
- [ ] `task check` green

## Blocked by

None — but do Option A after 024 (display subsystem) lands so the Displays submenu is stable.
