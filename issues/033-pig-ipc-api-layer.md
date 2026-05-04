# 033 — Extract pig/drag IPC into api/ layer

## What to build

Move the four direct `invoke`/`listen` calls that live in view-layer code into typed modules under `src/api/`, matching the pattern already established by `api/debugOverlay.ts` and `api/monitors.ts`.

Operations to extract:
- `invoke("set_pig_drag_active", { active })` — from `App.tsx:30`
- `listen("gather-pigs", ...)` — from `App.tsx:34`
- `listen<SpawnRegion>("display-region", ...)` — from `App.tsx:41`
- `invoke("update_pig_rects", { rects })` — from `usePigMovement.ts:219`

New module: `src/api/pig.ts`

```ts
export async function setPigDragActive(active: boolean): Promise<void>
export async function updatePigRects(rects: PigRect[]): Promise<void>
export async function subscribeGatherPigs(cb: () => void): Promise<Unsubscribe>
export async function subscribeDisplayRegion(cb: (region: SpawnRegion) => void): Promise<Unsubscribe>
```

`App.tsx` and `usePigMovement.ts` import from `src/api/pig.ts`; no `invoke`/`listen` remain in component or hook files.

## Completion promise

No `invoke` or `listen` calls remain in `src/components/` or `src/hooks/`; all pig/drag IPC lives in `src/api/pig.ts`.

## Acceptance criteria

- [ ] `src/api/pig.ts` exports all four typed wrappers
- [ ] `App.tsx` imports from `src/api/pig.ts`; no direct `invoke`/`listen`
- [ ] `usePigMovement.ts` imports from `src/api/pig.ts`; no direct `invoke`/`listen`
- [ ] `grep -r "invoke\|listen" src/components src/hooks` returns zero results
- [ ] `task check` green

## Blocked by

None — can start immediately.
