# 051 — Animal vocabulary for the remaining hit-test adapter

## Owning specification

`specs/003-ranch-today/spec.md` — FR-004

## What to build

The Rust-owned render scene and generic Animal movement shipped in PR #88. This issue now covers
only the remaining TypeScript hit-test adapter names: `PigHitRect` in `src/types/pig.ts`, the
shared wrappers in `src/api/pig.ts`, and their callers in `src/hooks/useAnimalMovement.ts`.
These values describe any Animal, not Pig-specific assets or physical policy.

Rename those shared TypeScript types, modules, and wrappers to Animal vocabulary. Keep the
`update_pig_rects`, `set_pig_drag_active`, and `gather-pigs` IPC names as compatibility contracts,
as `specs/001-render-domain-seam/spec.md` FR-013 permits (supporting context). Keep `PigSprite`,
its assets, and the Rust Pig profile species-specific.
Do not repeat the scene, projection, region, or selection cutover from PR #88.

## Completion promise

The shared TypeScript hit-test adapter uses Animal vocabulary while existing IPC contracts and
visible behavior remain unchanged.

## Acceptance criteria

- [ ] Shared hit-rectangle types and adapter wrappers use Animal names; the generic movement hook
      no longer imports `PigHitRect` or Pig-named TypeScript wrappers
- [ ] Existing IPC command/event names remain unchanged
- [ ] Pig assets and the Rust Pig profile retain their species-specific names
- [ ] Hit-testing, selection, Gather, dragging, and tossing retain their current behavior
- [ ] Existing regression tests and `task check` pass

## Blocked by

None. The renderer contract and Focus-only selection shipped in PR #88.

## User stories addressed

- "When I read the movement adapter, its type names describe any Animal rather than a Pig."
