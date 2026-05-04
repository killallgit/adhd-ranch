# 021 — All monitors enabled by default on first launch

## Parent PRD

PRD.md §FR2 (overlay window) + Phase 3 polish

## Problem

`DisplayConfig` defaults to `enabled_indices = [0]` (primary only). On first launch with multiple monitors, the overlay covers only the primary. Users must manually enable additional monitors in Tray → Displays. Pigs never cross to other screens until the user discovers this option.

## What to build

On first launch (or when `settings.yaml` has no `displays` key), populate `enabled_indices` with all connected monitor indices instead of just `[0]`.

- In `app/mod.rs` (or wherever `DisplayConfig` is defaulted): after enumerating monitors, if the config has no explicit `enabled_indices`, set it to `(0..monitors.len()).collect()`.
- Existing toggle behaviour in tray unchanged — user can still deselect monitors.
- Subsequent launches restore from `settings.yaml` as today.

## Completion promise

On first launch with 2+ monitors connected, all monitors are enabled and pigs roam the full span without any manual configuration.

## Acceptance criteria

- [ ] First launch with 2 monitors: both enabled by default, overlay spans both
- [ ] Toggling a monitor off in the tray still persists correctly
- [ ] Single-monitor setup unaffected
- [ ] `task check` green

## Blocked by

024 (display subsystem coordinate fix) — enabling all monitors by default is only useful once the spanning window is correct.
