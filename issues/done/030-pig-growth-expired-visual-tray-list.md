# 030 — Animal timer growth + expired visual + tray expired list

Completed by PR #59. GitHub #30 is closed.

## Parent PRD

PRD.md §FR3 (pig UI)

## What to build

Visual feedback for timer state: the animal projection for a Focus grows as its timer runs down, shows a distinct expired style, and expired focuses are listed in the tray.

The only concrete animal today is `Pig`, but this behavior is not pig-specific. New pure helpers and shared state should prefer `RanchAnimal` / animal-neutral naming. Keep `PigSprite` and `PigDetail` names only where the current UI component or sprite asset is specifically pig-shaped.

### Animal scale growth

- `src/hooks/useRanchAnimalScale.ts` — pure function hook:

  ```ts
  export function useRanchAnimalScale(startedAt: number | null, durationSecs: number | null): number
  // Returns 1.0 if no timer. Otherwise pig_scale(elapsed, duration) clamped 1.0–3.0.
  ```

- Current `PigSprite` component multiplies the base animal size by scale each render frame
- Hit rects use the scaled visual bounds, not the base animal size, so large animals remain clickable and click-through remains precise
- `PigDetail` popover positioning uses the scaled animal size for offset/clamping
- Scale recomputed from `Date.now()` each rAF tick — no new `PigState` fields

### Expired animal visual

- When `timer.status === 'Expired'`: animal renders with a ghostly transparent style
- Subtle pulse/shake animation on expiry (CSS keyframe, one-shot on status change)
- `PigDetail` shows timer status + remaining time (or "Expired")

### Tray expired list

- Tray menu section **"Expired"** lists focus titles whose timer status is `Expired`
- Shown below active focuses, separated by a menu divider
- Each item is read-only (clicking opens PigDetail or does nothing — designer choice)
- Section hidden when no expired focuses

## Completion promise

Focus animals with timers visually grow over their timer window; expired animals are visually distinct; expired focuses appear in a dedicated tray section.

## Acceptance criteria

- [x] New timer-scale helper uses animal-neutral naming and returns 1.0 for no-timer focuses
- [x] Current pig sprite renders larger as elapsed time increases toward `duration_secs`
- [x] Current pig sprite reaches ~3× base size at or after timer end
- [x] Hit testing and `PigDetail` positioning account for scaled animal size
- [x] Expired animal has distinct visual style (ghostly transparency)
- [x] Expiry animation plays once on status change
- [x] `PigDetail` shows "Expired" or remaining `mm:ss`
- [x] Tray lists expired focuses under a divider; section absent when none
- [x] `task check` green

## Blocked by

None. 028 and 029 are done.
