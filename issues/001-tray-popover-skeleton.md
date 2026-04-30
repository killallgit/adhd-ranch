# 001 — Tauri tray + popover skeleton with hardcoded Focus

## Parent PRD

`PRD.md`

## What to build

End-to-end shell of the desktop app with no storage or HTTP layer yet. Boots, shows a menubar tray icon, opens a borderless always-on-top popover near the icon. Popover renders a hardcoded list of one or two Focuses with a few Tasks each, using the React component layout. Demonstrates the wiring all the way from `TrayIconBuilder` → window manager → React view layer.

See `PRD.md` FR2 (Tauri menubar app) and FR3 (widget UI) for layout intent. Inline `✗` and `…` controls render but are no-ops in this slice.

## Acceptance criteria

- [ ] `task run` launches the app; tray icon appears in macOS menubar.
- [ ] Clicking tray icon toggles a borderless popover positioned near the tray icon (`tauri-plugin-positioner`).
- [ ] Popover stays on top and dismisses on click-away.
- [ ] Hardcoded Focus list renders in React, sourced from a typed fixture in the frontend `data` layer (no fetch).
- [ ] Frontend data layer and view components live in separate folders; components do not import fixture data directly except via a typed adapter.
- [ ] `task check` green.

## Blocked by

- Blocked by `issues/000-scaffolding.md`

## User stories addressed

- User story 1
