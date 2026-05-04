# 022 — Wrangle pig + wrangle all

## Parent PRD

PRD.md §FR3 (pig UI)

## Problem

Pigs can drift to edges or off-screen (especially in multi-monitor setups). No way to round them up without restarting.

## What to build

### Wrangle (per-pig)
Tray → [Focus name] submenu → **Wrangle** — moves the pig to the centre of the primary screen. Pig resumes wandering from that position.

### Wrangle All
Top-level tray item: **Wrangle All** — moves all pigs to a visible cluster on the primary screen. Fan them out slightly from centre (e.g. spread by index × 80px) so they're not stacked.

Both actions update the pig's `(x, y)` state in the frontend via a Tauri event or command.

## Completion promise

User can summon any or all pigs back to the primary screen with a single menu click.

## Acceptance criteria

- [ ] Tray → Focus submenu has a "Wrangle" item
- [ ] Clicking "Wrangle" moves that pig to primary screen centre; pig resumes wandering
- [ ] Top-level "Wrangle All" clusters all pigs visibly on primary screen
- [ ] Pigs don't stack on top of each other after wrangle all
- [ ] `task check` green

## Blocked by

024 (display subsystem) — needs correct screen bounds to compute primary-screen centre.
