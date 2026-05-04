# 030 — Pig scale growth + expired visual + tray expired list

## Parent PRD

PRD.md §FR3 (pig UI)

## What to build

Visual feedback for timer state: pigs grow as their timer runs down, show a distinct expired style, and expired focuses are listed in the tray.

### Pig scale growth

- `src/hooks/usePigScale.ts` — pure function hook:

  ```ts
  export function usePigScale(startedAt: number | null, durationSecs: number | null): number
  // Returns 1.0 if no timer. Otherwise pig_scale(elapsed, duration) clamped 1.0–3.0.
  ```

- Pig component multiplies `PIG_SIZE` by scale each render frame
- Scale recomputed from `Date.now()` each rAF tick — no new `PigState` fields

### Expired pig visual

- When `timer.status === 'Expired'`: pig renders with red tint (CSS `filter: hue-rotate` or overlay)
- Subtle pulse/shake animation on expiry (CSS keyframe, one-shot on status change)
- `PigDetail` shows timer status + remaining time (or "Expired")

### Tray expired list

- Tray menu section **"Expired"** lists focus titles whose timer status is `Expired`
- Shown below active focuses, separated by a menu divider
- Each item is read-only (clicking opens PigDetail or does nothing — designer choice)
- Section hidden when no expired focuses

## Completion promise

Pigs with timers visually grow over their timer window; expired pigs are visually distinct; expired focuses appear in a dedicated tray section.

## Acceptance criteria

- [ ] `usePigScale` returns 1.0 for no-timer focuses
- [ ] Pig renders larger as elapsed time increases toward `duration_secs`
- [ ] Pig reaches ~3× base size at or after timer end
- [ ] Expired pig has distinct visual style (red tint)
- [ ] Expiry animation plays once on status change
- [ ] `PigDetail` shows "Expired" or remaining `mm:ss`
- [ ] Tray lists expired focuses under a divider; section absent when none
- [ ] `task check` green

## Blocked by

028 (FocusTimer + pig_scale domain types)
029 (timer-expired Tauri event for expiry animation trigger)
