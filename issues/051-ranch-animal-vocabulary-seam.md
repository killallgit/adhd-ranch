# 051 — Animal vocabulary for shared movement code

## Parent PRD

PRD.md §FR3 (Pig UI) and the post-030 decision that timer-growth behavior is animal-neutral even though the only concrete animal today is a pig.

## What to build

Prepare the frontend and shared type names for future Species without changing user-visible behavior.

Today several modules use `Pig` names for two different things:

- concrete pig assets or components, such as `PigSprite` and the pig sprite sheet
- animal-neutral overlay behavior, such as movement state, hit rectangles, and drag/toss

The concrete pig names are fine while pigs are the only rendered Animal. The shared behavior should speak in **Animal** terms so a second Species can be added without a broad naming scramble.

`CONTEXT.md` names the general concept **Animal** and says to avoid `RanchAnimal`, so this issue no longer proposes a `RanchAnimal` prefix. The projection half of the original issue — turning Focuses and Agent Sessions into Animals — moved to 055 and is not repeated here.

### Target shape

- Keep `PigSprite` while pigs are the only rendered Animal; the detail surface is already `AnimalDetail`.
- Rename the shared movement surface to Animal vocabulary: `PIG_SIZE`, `PIG_SPEED`, `PigState`, `PigDirection`, `PigHitRect`, `usePigMovement`, and the `api/pig.ts` client.
- Existing Tauri command names are either intentionally retained for compatibility or migrated with wrappers.
- Do not change storage format, Focus ids, tray labels, or user-visible copy in this slice.

### Out of scope

- Adding a second Species.
- Animal selection UI (055).
- Replacing the current pig sprite sheet.
- Rewriting historical issue files that correctly described pig-only behavior at the time.

## Completion promise

Shared overlay behavior uses Animal vocabulary, while concrete pig asset/component names remain pig-specific until a second Species exists.

## Acceptance criteria

- [ ] Shared movement state and types carry Animal names, with no pig-specific names left in animal-neutral code
- [ ] Shared hit-test helpers use Animal names or explicit compatibility aliases
- [ ] Current pig rendering, detail behavior, drag/toss, timer growth, and tray behavior are unchanged
- [ ] Existing public Tauri command names are either intentionally retained for compatibility or migrated with compatibility wrappers
- [ ] Tests cover the renamed shared helpers or aliases
- [ ] `task check` green

## Blocked by

055, which removes `PigSubject` and the scale map this issue would otherwise have to rename.

## User stories addressed

- "When we add more Species later, shared ranch behavior already speaks in Animal terms instead of assuming every Focus is forever represented by a pig."
