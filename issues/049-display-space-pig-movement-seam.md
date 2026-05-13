# 049 — Display-space seam for Pig movement

## Parent PRD

CONTEXT.md §Core interaction loop step 8 (Configure displays)

## What to build

Deepen the display and Pig movement seam so React movement code no longer infers monitor behavior from raw viewport dimensions.

Today Rust computes monitor span and a primary region, then React still owns display policy inside `usePigMovement`: spawn region, primary-display y ceiling, hard boundary clamping, hit-rect widening during drag, and the difference between overlay span and visible monitor regions. This makes cross-monitor behavior hard to reason about, especially with negative coordinates and portrait monitors.

### Target shape

Create a compact display-space model emitted from Rust and consumed by React:

```ts
interface DisplaySpace {
  readonly span: { readonly w: number; readonly h: number };
  readonly spawnRegion: Rect;
  readonly movementRegions: readonly Rect[];
  readonly hitTestScale: number;
}
```

Rust should own monitor geometry: enabled monitor filtering, logical-coordinate span, primary spawn region, and movement regions. React should own motion over that display-space model: ticking velocity, random turns, drag/toss, and hit rect synchronization.

The exact type names can change. The important seam is that `tickPig` should not need to know "if x is in the primary display, use a different y ceiling." It should ask a display-space helper to clamp or steer within allowed regions.

## Completion promise

Pig movement consumes a display-space interface instead of raw viewport dimensions and ad hoc primary-region rules; multi-monitor geometry policy is local to the display-space module.

## Acceptance criteria

- [ ] Rust emits a display-space payload that includes span, spawn region, and allowed movement regions
- [ ] React has a pure display-space helper for clamp/steer decisions with unit tests
- [ ] `tickPig` no longer hard-codes primary-display y ceiling logic
- [ ] Tests cover single monitor, side-by-side monitors, negative-x monitor, portrait monitor, and monitor-above-primary geometry
- [ ] Drag/toss still keeps the overlay interactive during active drag
- [ ] Existing hit-rect tests remain green or are moved to the new helper
- [ ] `task check` green

## Blocked by

None

## User stories addressed

- "When I add monitors in odd layouts, Pigs stay in visible display regions and the movement rules are testable without opening Tauri windows."
