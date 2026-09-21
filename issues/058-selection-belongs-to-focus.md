# 058 — Selection belongs to the Focus half

## Parent PRD

PRD.md §FR3 (Pig UI) and ADR-0001.

## What to build

`useAnimalSelection` takes every Animal and throws half of them away: it returns
`FocusAnimal | null` via `requested?.kind === "focus" ? requested : null`, and `App.tsx` then
re-narrows a second time through `selected?.focus ?? null` behind a triple guard. After 057 there
is no `kind` left to test, so this has to move.

Selection is a Focus-half concern. The renderer should report that a sprite was clicked, and take
an id to freeze, without knowing what either means.

### Target shape

- `hooks/useFocusSelection.ts` replaces `useAnimalSelection`: it takes the `focuses` and the
  requested id, returns `Focus | null`, and forgets an id that no longer resolves — 055's rule,
  carried over unchanged.
- The movement layer takes `frozenId: string | null` and emits click-by-id. It learns nothing
  about what a click means.
- `App.tsx` routes clicks to the Focus half. A click on an agent Animal resolves to nothing, as
  it does today.
- The wide hit rect still follows "something is selected", and still turns off in the same render
  when the selection stops resolving.

### Out of scope

- Making agent Animals selectable, or giving them a detail card.
- The movement-layer rename (051).
- The `optimisticFocuses` append path in `App.tsx`.

## Completion promise

The renderer reports clicks and freezes by id, and only the Focus half decides what a click
means — so a deleted Focus still cannot leave the overlay swallowing clicks.

## Acceptance criteria

- [ ] `useAnimalSelection` is gone; `useFocusSelection` returns `Focus | null`
- [ ] No `kind` test anywhere in selection
- [ ] The movement layer takes `frozenId` and emits click-by-id, naming no domain type
- [ ] Clicking an agent Animal opens nothing
- [ ] Test: a selection whose Focus has vanished resolves to none and the hit rect is narrow
      again — 055's regression, kept
- [ ] `task check` green

## Blocked by

057.

## User stories addressed

- "When a Focus I had open disappears, the overlay goes back to letting my clicks through."
- "When I read the movement layer, it has no opinion about what a click means."
