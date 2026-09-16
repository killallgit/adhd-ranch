# 055 — Animal projection and selection

## Parent PRD

PRD.md §FR3 (Pig UI) and the agents-as-animals slice. Architecture review 2026-09-15, candidate 2. Supersedes the projection half of 051.

## What to build

`App.tsx` decides in six separate places what an Animal is and whether it can be selected: `agentPigId`, `pigSubjects`, the `agentPigIds` set, the `animalScales` map, `expired` computed twice, and a raw `selectedId` that nothing prunes.

That last one is a live bug. When the selected Focus disappears — deleted from the tray, or removed on disk — `selectedId` keeps its value. `AnimalDetail` stops rendering because `selectedFocus` is undefined, but `usePigMovement` still reports the wide hit rect (`wide = selectedIdRef.current !== null`), so the overlay swallows every click until something else is selected.

Also fix the name. `CONTEXT.md` defines **Animal** as a Focus or an Agent Session and says to avoid `RanchAnimal`, but the Rust `Animal` type describes only an Agent Session.

### Target shape

- Rename in Rust and in the generated types: `Animal` → `AgentSession`, `AnimalStore` → `AgentSessionStore`, `list_animals` → `list_agent_sessions`, `animals-changed` → `agent-sessions-changed`. `Commands::Animals` follows. This frees **Animal** for the projection.
- `lib/animals.ts`: `projectAnimals(focuses, sessions, nowMs): readonly Animal[]`, a discriminated union:
  - `{ kind: "focus", id, name, expired, scale, focus }`
  - `{ kind: "agent", id, name, session }` with `id` namespaced `agent:<session id>`
  - Selectability follows `kind`. No boolean flag.
- `hooks/useAnimalSelection.ts`: holds the raw id, derives `selected` by looking it up in the current Animals. An Animal that is gone yields `null`, so the wide hit rect turns off in the same render. `useOpenFocusDetailRequest` folds in.
- `usePigMovement(animals, selectedId)`. `PigSubject` and the `animalScales` map are deleted; scale comes off the Animal.
- App renders Animals and passes `selected` to `AnimalDetail`.

### Out of scope

- The `optimisticFocuses` append path in App, including its copy of the Revive rule (candidate 6).
- Splitting the rAF loop out of `usePigMovement` (candidate 5), and the scaled-bounds bug it covers.
- Renaming `PIG_SIZE`, `PIG_SPEED`, `PigState`, `PigDirection`, `PigSprite` (051).
- Adding a second Species.

## Completion promise

One pure projection decides what every Animal is, and a selection that no longer exists resolves to none, so a deleted Focus can never leave the overlay swallowing clicks.

## Acceptance criteria

- [x] Rust `Animal` is `AgentSession` end to end, including the command, the event name and `src/types/generated/`
- [x] `projectAnimals` is pure, takes `nowMs`, and owns id namespacing, `expired` and `scale`
- [x] `App.tsx` contains no `agent:` prefixing, no `agentPigIds` set, no `animalScales` map and no duplicate `expired` lookup
- [x] `useAnimalSelection` derives the selected Animal and forgets one that is gone; selecting an agent Animal is not possible
- [x] Test: a selection whose Focus has vanished resolves to none and the hit rect is narrow again
- [x] `usePigMovement` takes projected Animals; `PigSubject` is gone
- [x] Projection rules (id namespacing, expired, scale, ordering) are covered by `projectAnimals` tests; the App tests keep their rendering assertions plus the stale-selection regression
- [x] `task check` green

## Blocked by

None. Ships before 054.

## User stories addressed

- "When a Focus I had open disappears, the overlay goes back to letting my clicks through."
- "When we add a second Species, one module decides what an Animal is."
