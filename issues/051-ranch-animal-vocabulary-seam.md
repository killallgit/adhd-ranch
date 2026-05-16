# 051 — RanchAnimal vocabulary seam for future animal types

## Parent PRD

PRD.md §FR3 (Pig UI) and the post-030 decision that timer-growth behavior is animal-neutral even though the only concrete animal today is a pig.

## What to build

Prepare the frontend and shared type names for future animal types without changing user-visible behavior.

Today several modules use `Pig` names for two different things:

- concrete pig assets or components, such as `PigSprite` and the pig sprite sheet
- animal-neutral overlay behavior, such as movement state, hit rectangles, scale, drag/toss, and focus-to-animal projection

The concrete pig names are fine while pigs are the only rendered animal. The shared behavior should move toward `RanchAnimal` vocabulary so future animal types can be added without a broad naming scramble.

### Target shape

- Keep `PigSprite` while pigs are the only rendered animal; the detail surface is already animal-neutral as `AnimalDetail`.
- Prefer `RanchAnimal` names for new shared movement, scaling, hit-test, and projection helpers.
- Introduce aliases/adapters where needed so the refactor can be incremental and reviewable.
- Do not change storage format, focus IDs, tray labels, or user-visible copy in this slice.

### Out of scope

- Adding a second animal.
- Animal selection UI.
- Replacing the current pig sprite sheet.
- Rewriting all historical issue files that correctly described pig-only behavior at the time.

## Completion promise

Shared overlay behavior uses animal-neutral vocabulary, while concrete pig asset/component names remain pig-specific until a second animal type exists.

## Acceptance criteria

- [ ] Existing shared movement state/types are renamed or wrapped so animal-neutral code does not introduce new pig-specific names
- [ ] Existing shared hit-test and scale helpers use animal-neutral names or explicit compatibility aliases
- [ ] Current pig rendering, detail behavior, drag/toss, timer growth, and tray behavior are unchanged
- [ ] Existing public Tauri command names are either intentionally retained for compatibility or migrated with compatibility wrappers
- [ ] Tests cover the renamed shared helpers or aliases
- [ ] `task check` green

## Blocked by

None. Best after 031 unless we decide to spend a refactor slice first.

## User stories addressed

- "When we add more animals later, shared ranch behavior already speaks in animal terms instead of assuming every Focus is forever represented by a pig."
