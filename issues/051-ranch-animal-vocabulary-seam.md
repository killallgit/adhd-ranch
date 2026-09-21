# 051 — Animal vocabulary for the movement layer

## Parent PRD

PRD.md §FR3 (Pig UI), the post-030 decision that timer-growth behavior is animal-neutral, and
ADR-0001, which settles **Animal** as a render-layer word.

## What to build

Give the movement layer one vocabulary — Animal — without changing user-visible behavior.

### Why this issue changed

The original 051 proposed these renames while CONTEXT.md still defined **Animal** as *"either a
Focus or an Agent Session."* Under that definition the renames would have stamped a domain union
across the one layer that is genuinely shared, teaching every future reader that the movement
layer traffics in domain objects.

ADR-0001 fixes the definition rather than the rename. **Animal** now means the sprite the ranch
draws and moves; Focus and Agent Session each project their own. Under that definition these
renames are not merely safe, they are the point: `useAnimalMovement` moving Animals is exactly
what that layer does.

Two things follow. The `Pig*` renames are unchanged from the original issue. But the scope was
too small — the movement layer carries **three** vocabularies today, not two.

### The third vocabulary

`lib/ranchAnimalMovement.ts` already speaks `RanchAnimalState`, `RanchAnimalDirection`,
`RanchAnimalInput`, `advanceRanchAnimal` and `restRanchAnimal`, while `hooks/usePigMovement.ts`
speaks `PigState`, `PigDirection` and `PigHitRect`, and both handle projected `Animal`s. CONTEXT.md
has listed `RanchAnimal` as a term to avoid the whole time. The original issue did not mention it.

### Target shape

- Rename the `Pig*` surface: `PIG_SIZE`, `PIG_SPEED`, `PigState`, `PigDirection`, `PigHitRect`,
  `usePigMovement`, `types/pig.ts` and the `api/pig.ts` client.
- Rename the `RanchAnimal*` surface: `RanchAnimalState`, `RanchAnimalDirection`,
  `RanchAnimalInput`, `advanceRanchAnimal`, `restRanchAnimal`, `lib/ranchAnimalMovement.ts`.
- Keep `PigSprite` and the pig sprite sheet. Those are the concrete Species asset and stay
  pig-specific until a second Species exists.
- The movement layer must name no domain type. After 057 it takes Animals and regions; this
  slice keeps it that way.
- Existing Tauri command names are either intentionally retained for compatibility or migrated
  with wrappers.
- Do not change storage format, Focus ids, tray labels, or user-visible copy.

### Out of scope

- Adding a second Species.
- The render contract and the deletion of the `Animal` union (057).
- Moving selection into the Focus half (058).
- Replacing the current pig sprite sheet.
- Rewriting historical issue files that correctly described pig-only behavior at the time.

## Completion promise

The movement layer speaks one vocabulary and names no domain type, while the concrete pig asset
stays pig-specific until a second Species exists.

## Acceptance criteria

- [ ] Movement state, direction, hit-rect and input types carry Animal names; no `Pig*` and no
      `RanchAnimal*` names remain in animal-neutral code
- [ ] No file in the movement layer imports or names `Focus`, `AgentSession`, `Timer` or `Pen`
- [ ] `PigSprite` and the sprite sheet keep their pig-specific names
- [ ] Current pig rendering, detail behavior, drag/toss, timer growth and tray behavior are
      unchanged
- [ ] Existing public Tauri command names are either intentionally retained for compatibility or
      migrated with compatibility wrappers
- [ ] Tests cover the renamed helpers or aliases
- [ ] `task check` green

## Blocked by

057 — the render contract from ADR-0001 action item 3, not yet filed.

Renaming first would churn the same files twice: 057 changes what the movement layer *takes*,
this changes what it is *called*. The original "blocked by 055" is resolved — 055 shipped and
`PigSubject` is gone.

## User stories addressed

- "When we add a second Species, the movement layer already speaks in Animal terms instead of
  assuming every Animal is forever a pig."
- "When I read the movement layer, nothing in it tells me whether a Focus or an agent put that
  Animal there."
