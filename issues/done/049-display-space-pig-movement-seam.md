# 049 — Display-space seam for RanchAnimal movement

## Parent PRD

CONTEXT.md §Core interaction loop step 8 (Configure displays)

## What to build

Deepen the display and RanchAnimal movement seam so React movement code no longer infers monitor behavior from raw viewport dimensions.

Today Rust computes monitor span and a primary region, then React still owns display policy inside `usePigMovement`: spawn region, primary-display y ceiling, hard boundary clamping, hit-rect widening during drag, and the difference between overlay span and visible monitor regions. This makes cross-monitor behavior hard to reason about, especially with negative coordinates and portrait monitors. Pig is the only implemented RanchAnimal today, but this seam should avoid Pig-specific names where the behavior applies to future animal projections. Per-animal movement profiles are explicitly out of scope for this slice; all RanchAnimals use one shared movement rule set for now.

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

Rust should own monitor geometry: enabled monitor filtering, logical-coordinate span, primary spawn region, and movement regions. Movement regions are the actual visible enabled-monitor rectangles normalized into overlay coordinates; they are not the full rectangular span. React should own motion over that display-space model through a deep movement-step module: given a RanchAnimal, DisplaySpace, elapsed time, current time, randomness Adapter, and shared movement options, it returns the next RanchAnimal state. The hook should stay a thin Adapter for React state, animation frames, Tauri subscriptions, and hit rect synchronization.

The exact type names can change. The important seam is that `tickPig` should not need to know "if x is in the primary display, use a different y ceiling." It should delegate movement sequencing to the movement-step module rather than stitching together raw geometry helpers. If a RanchAnimal's current position becomes invalid after a display change, that module should move it to the nearest valid point in any movement region rather than respawning it.

Movement edge behavior is hybrid: soft-steer near movement-region edges so wandering still feels natural, then hard-clamp and reflect velocity if a RanchAnimal still crosses outside the allowed regions.

## Completion promise

RanchAnimal movement consumes a display-space interface instead of raw viewport dimensions and ad hoc primary-region rules; multi-monitor geometry policy is local to the display-space module.

## Acceptance criteria

- [x] Rust emits a display-space payload that includes span, spawn region, and allowed movement regions
- [x] Allowed movement regions are normalized visible monitor rectangles, so RanchAnimals cannot enter invisible gaps inside the overlay span
- [x] React has a pure movement-step module for roam/freeze/friction/turn/soft-steer/hard-clamp decisions with unit tests
- [x] The movement-step interface accepts randomness as an Adapter so turn timing and direction are deterministic in tests
- [x] Invalid RanchAnimal positions after display changes are clamped to the nearest valid point in any movement region
- [x] Movement uses hybrid edge behavior: soft steering near region edges, hard clamp/reflect after crossing
- [x] `tickPig` no longer hard-codes primary-display y ceiling logic
- [x] Tests cover single monitor, side-by-side monitors, negative-x monitor, portrait monitor, and monitor-above-primary geometry
- [x] Drag/toss still keeps the overlay interactive during active drag
- [x] Existing hit-rect tests remain green or are moved to the new helper
- [x] `task check` green

## Blocked by

None

## User stories addressed

- "When I add monitors in odd layouts, RanchAnimals stay in visible display regions and the movement rules are testable without opening Tauri windows."
