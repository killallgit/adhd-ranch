# 057 — Animal as a render contract

## Parent PRD

PRD.md §FR3 (Pig UI) and ADR-0001, which moves the seam between the ranch's two halves from the
domain down to the renderer.

## What to build

Delete the `Animal` union. Both halves project into a flat, render-only Animal that names no
domain type.

Today `Animal` is `FocusAnimal | SessionAnimal` over a shared `AnimalBase`, and only one of that
base's four fields is real for both halves: `scale` is hardcoded `1` for every agent Animal,
`resting` collapses two unrelated causes into one boolean, and `id` carries an `agent:` prefix to
keep two id spaces from colliding. Every consumer then discriminates straight back apart —
`useAnimalSelection` accepts `Animal[]` and returns `FocusAnimal | null`, `App.tsx` re-narrows
again through `selected?.focus ?? null` behind a triple guard, and `AnimalDetail` never accepted
an Animal at all.

CONTEXT.md now defines **Animal** as the sprite the ranch draws and moves, and **Motion** as the
renderer's own vocabulary. This slice makes the types say that.

### Target shape

`src/types/animal.ts`:

```ts
export type Motion = "walking" | "resting";

export interface Animal {
  readonly id: string;
  readonly label: string;
  readonly size: number;             // absolute px — the renderer stops knowing about timers
  readonly regionId: string | null;  // null = the run of the DisplaySpace
  readonly motion: Motion;
}
```

- `FocusAnimal`, `SessionAnimal`, `AnimalBase` and `animalPen()` are deleted.
- `lib/focus/animals.ts` — `size` is `PIG_SIZE * growth`, `regionId` is `null`, `motion` is
  `"resting"` when the Timer is Expired.
- `lib/session/animals.ts` — `size` is `PIG_SIZE`, `regionId` is the Pen id, `motion` is
  `"resting"` when the activity is Idle.
- `lib/animals.ts` — `projectAnimals` still concatenates, and `nowMs` still reaches only the
  Focus half.
- `PigSprite` takes `size` rather than `scale`; the `animalScales` map in `usePigMovement` goes
  with it.

### Selection, for this slice only

Selection still has to work, and moving it properly is 058. Here the Focus half looks a selected
id back up against the `focuses` it already holds. Behavior does not change: `AnimalDetail` keeps
opening for a Focus Animal and keeps never opening for an agent Animal.

### Out of scope

- Renaming the movement layer to Animal vocabulary (051).
- Moving selection into the Focus half properly (058).
- Richer Motions for agents. This slice ships `"walking" | "resting"` and nothing more — the
  point is that adding to it later touches one half only.
- Anything under `crates/` or `src-tauri/`.
- Adding a second Species.

## Completion promise

`Animal` is a render type that names no domain concept, and the two halves project into it
independently, so a Motion added for one half cannot touch the other.

## Acceptance criteria

- [ ] `src/types/animal.ts` exports a flat `Animal` and a `Motion`; `FocusAnimal`,
      `SessionAnimal` and `AnimalBase` are gone
- [ ] No `kind` discriminant, and no `focus` or `session` payload, anywhere on an Animal
- [ ] `animalPen()` is deleted; the movement layer takes `regionId`
- [ ] Neither `hooks/usePigMovement.ts`, `lib/ranchAnimalMovement.ts` nor `components/PigSprite.tsx`
      imports or names `Focus`, `AgentSession`, `Timer` or `Pen`
- [ ] Agent Animals still group by Pen; Focus Animals still have the run of the DisplaySpace
- [ ] Timer growth, drag/toss, resting opacity, Gather and the detail card are unchanged
- [ ] `AnimalDetail` opens for a Focus Animal and never for an agent Animal, with 055's
      stale-selection regression still covered
- [ ] `task check` green

## Blocked by

None. ADR-0001 is Accepted.

## Blocks

051 (which renames what this slice reshapes) and 058.

## User stories addressed

- "When an agent Animal needs a new way to move, nothing about my Focuses changes."
- "When I read the movement layer, nothing in it tells me whether a Focus or an agent put that
  Animal there."
