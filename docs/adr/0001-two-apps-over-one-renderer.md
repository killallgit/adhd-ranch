# ADR-0001: Split Focus and Agent Session into two applications over one renderer

**Status:** Accepted
**Date:** 2026-09-20 (proposed)
**Accepted:** 2026-09-21
**Deciders:** Ryan (project owner)

## Context

The ranch draws two kinds of Animal. They were built as one concept and they are not one
concept. Every axis on which software can differ, these differ:

| | Focus half | Agent half |
|---|---|---|
| Origin | the user creates it | observed; the user can never create one |
| Storage | markdown files on disk | in-memory `BTreeMap`, nothing persisted |
| Change transport | fs watcher, 200 ms debounce | unix socket push |
| Identity | slug — stable, reusable, comes back | harness session id — opaque, never returns |
| Lifecycle | create / duplicate / delete | start / end, observed |
| Time | timers, expiry, countdown, growth | explicitly no clock, no expiry |
| Grouping | none | one pen per repo checkout |
| Limits | counts toward caps | excluded by design |
| Interaction | selectable, detail card, editable | never selectable, no card |
| Direction | read-write | read-only |
| Gate | always on | `agents.enabled`, off by default |

They share one thing: a sprite that walks around a region of the screen.

### The union is three-quarters coincidence

`src/types/animal.ts` gives both halves a common supertype:

```ts
interface AnimalBase {
  readonly id: string;       // two id spaces, hence the `agent:` prefix
  readonly name: string;     // genuinely shared — a label to draw
  readonly scale: number;    // Focus grows with its timer; Session is hardcoded 1
  readonly resting: boolean; // two unrelated causes collapsed into one bool
}
```

One of those four fields is real. The comment above it already concedes the point: *"a Focus
rests because its timer ran out, a Session rests because it is between turns. Neither knows
why the other one does."* That is two concepts sharing a boolean, not a shared concept.

### The union pays for nothing

Every consumer discriminates straight back apart:

- `useAnimalSelection` accepts `readonly Animal[]` and returns `FocusAnimal | null`, discarding
  half its input via `requested?.kind === "focus" ? requested : null`
- `App.tsx` re-narrows again — `selected?.focus ?? null`, guarded by
  `selectedPig && selected && selectedFocus &&`
- `AnimalDetail` never accepted an `Animal`; it takes `focus: Focus` directly
- `animalPen()` returns `Pen | null`, permanently null for focus animals, threaded through
  shared code as `animalPen(animal)?.id ?? ""`
- the projection signature is the two apps concatenated, with a clock that reaches only one:
  `projectAnimals(focuses, sessions, nowMs) => [...projectFocusAnimals(focuses, nowMs), ...projectSessionAnimals(sessions)]`

A union whose every consumer immediately re-discriminates is a tuple with extra steps.

### Forces

1. **The glossary generates the code.** CONTEXT.md defines **Animal** as *"either a Focus or an
   Agent Session."* The union is a faithful implementation of the domain model. Changing the
   code without changing the glossary will regenerate the union.
2. **It blocks the agent animation work.** `resting: boolean` is the shared vocabulary, so the
   agent half cannot gain a single motion state without changing the Focus half's type. The
   expressive ceiling of the most dynamic half is welded to the most static one.
3. **Issue 051 is queued to widen it.** 051 renames the shared movement surface (`PigState`,
   `usePigMovement`, `PIG_SIZE`) into *Animal* vocabulary — stamping the conflated word across
   the one layer that is genuinely shared.
4. **The Rust side already has the seam and then erases it.** The module tree is clean
   (`domain::focus`/`timer`/`caps` vs `domain::session`/`agents`; same split in `commands` and
   `storage`), but `domain/lib.rs` flattens everything into one prelude, so `Focus` and
   `AgentSession` sit side by side at the crate root and nothing would notice the boundary
   being crossed.
5. **No big-bang rewrite.** Solo project, `task check` must stay green, and each slice has to
   be reviewable in under fifteen minutes.

### What 055 got right

Issue 055 introduced this union, and it was good work: it killed a live bug (a stale selection
leaving the overlay swallowing clicks) and collapsed six ad-hoc decisions in `App.tsx` into one
pure projection. The consolidation was correct. The shape it consolidated *into* is what this
ADR revisits.

## Decision

**Move the seam from the domain to the renderer.** Delete the `Animal` union. Let each half
project independently into a render contract that knows nothing about Focus, Session, timer,
pen or selection.

```
focus/   Focus → timers  → sprites ──┐
                                     ├──> ranch renderer ──> screen
agent/   hooks → sessions → sprites ─┘
```

The render contract carries only what a walking sprite needs:

```ts
interface RanchSprite {
  readonly id: string;
  readonly label: string;
  readonly size: number;             // absolute px — the renderer stops knowing about timers
  readonly regionId: string | null;  // null = whole DisplaySpace; the renderer never learns "pen"
  readonly motion: Motion;           // starts as "walking" | "resting"
}
```

Three consequences of that shape are the point of the whole decision:

- **`scale` becomes `size`.** Focus computes `PIG_SIZE * growth`; Agent computes `PIG_SIZE`.
  Timer growth stops being a field every animal carries.
- **`pen` becomes `regionId`.** A pen is how the agent half computes its region. The renderer
  takes regions.
- **`Motion` is open for extension, and that is legitimate.** It is a *rendering* vocabulary,
  not a domain identity. Adding `"settling"` for agents does not force the Focus half to change
  — Focus simply never produces it. This is exactly the property `resting: boolean` lacks today.

Selection stays a Focus-half concern. The renderer reports "sprite `id` was clicked" and takes a
`frozenId` without knowing why it is frozen; the Focus half decides what a click means and the
agent half ignores clicks entirely.

**Animal** is redefined in CONTEXT.md as a render-layer word — the sprite on screen — and stops
being a domain supertype over Focus and Agent Session.

## Options Considered

### Option A: Status quo — widen `AnimalBase` as needs arrive

| Dimension | Assessment |
|---|---|
| Effort now | None |
| Blast radius | None now; grows with every field |
| Unblocks agent animation | No |
| Drift resistance | None — the glossary keeps regenerating the union |
| Reversibility | Decreasing over time |

**Pros:** zero cost today; the union is only mildly annoying at current size.
**Cons:** every agent motion state becomes a change to the Focus half's type; `AnimalBase`
accretes fields dead for one side (`scale` already is); 051 will spread the conflated vocabulary
into the render layer, after which this is materially more expensive to undo.

### Option B: Render-layer contract (recommended)

| Dimension | Assessment |
|---|---|
| Effort now | Medium — frontend only, 2–3 slices |
| Blast radius | `types/animal.ts`, `lib/animals.ts`, `useAnimals`, `useAnimalSelection`, `usePigMovement`, `App.tsx` |
| Unblocks agent animation | Yes — directly |
| Drift resistance | Convention + glossary; no compiler enforcement |
| Reversibility | High — pure types and projections, no storage or IPC change |

**Pros:** smallest change that makes the boundary real; no Rust, storage, or IPC churn; each
half becomes independently testable; 051 can then rename honestly toward render vocabulary.
**Cons:** two places compute sprites instead of one; nothing at build time stops a future import
from crossing back.

### Option C: Option B plus enforced boundaries

Option B, then split the halves into separate crates (or lint-enforced folder isolation on the
TS side) so a cross-import fails the build rather than a review.

| Dimension | Assessment |
|---|---|
| Effort now | High — touches crate graph, `domain/lib.rs` prelude, `Cargo.toml`, knip config |
| Blast radius | Both Rust and TS, plus the `task check` tooling |
| Unblocks agent animation | Yes, but not sooner than B |
| Drift resistance | Strong — the compiler holds the line |
| Reversibility | Medium |

**Pros:** the boundary stops depending on discipline; makes the second harness (Codex) and the
second Species genuinely additive.
**Cons:** large diff for zero user-visible change; premature while the render contract's shape
is still being learned. The enforcement is worth more once B has proven the contract.

### Option D: Two applications

Separate binaries or processes, each owning its own overlay.

| Dimension | Assessment |
|---|---|
| Effort now | Very high |
| Blast radius | Everything — windowing, tray, settings, display management |
| Unblocks agent animation | Yes, at absurd cost |
| Drift resistance | Total |
| Reversibility | Low |

**Pros:** the boundary becomes physically impossible to cross.
**Cons:** the two halves genuinely *do* share the overlay window, the DisplaySpace, hit-testing,
the tray and the settings file. Splitting the process duplicates all of it to separate two
projections. Rejected — the sharing is real at the shell layer, just not at the domain layer.

## Trade-off Analysis

The decision is really **where to draw the line**, and the candidates are three layers apart:

- **A** draws it at the domain — two things are one thing. Cheapest, and already failing: it is
  what makes agent motion states a cross-cutting change.
- **B** draws it at the renderer — two things that look alike on screen. This matches the one
  honest sentence already in the codebase: they *"draw and move the same way,"* and nothing more.
- **D** draws it at the process — two things that share nothing. False, because the shell layer
  (window, displays, tray, settings) genuinely is shared.

B is the only line that matches the actual sharing. C is not a different line — it is B plus
enforcement, and enforcement is worth buying only after the contract has survived contact with
the agent motion work. Doing C first would mean hardening a boundary whose exact shape is still
a guess.

The real cost of B is that sprite construction lives in two places. That is not duplication;
it is two different computations that currently pretend to be one, and the pretending is what
costs us.

## Consequences

**Easier**
- Agent motion states, timings and transitions can grow without touching Focus
- Each half is testable without the other; `projectAnimals` stops needing a clock it half-ignores
- A second harness (Codex) and a second Species become additive rather than union-widening
- 051 can rename the movement layer toward render vocabulary without cementing the conflation

**Harder**
- One more indirection between domain state and pixels
- Two sprite projections to keep visually consistent
- The generated TS types no longer line up 1:1 with a single `Animal`

**To revisit**
- Whether `Pen` stays agent-only, if Focus ever gains grouping
- Whether `Motion` stays one shared enum or becomes per-half once the agent vocabulary grows
- Whether Option C's enforcement is worth buying, once the contract has settled (revisit after
  the first agent motion slice ships)

## Action Items

1. [x] Rewrite CONTEXT.md's **Animal** entry: a render-layer word for the sprite on screen, not
       a supertype over Focus and Agent Session. Add **Sprite**/**Motion** if the contract keeps
       those names.
2. [x] Rewrite issue 051 to rename the movement surface toward render vocabulary, and unblock it
       from the (now implemented) 055.
3. [x] Issue 057: introduce `RanchSprite` + `Motion`; have both halves project into it; delete
       the `Animal` union, `AnimalBase`, and `animalPen()`.
4. [x] Issue 058: move selection wholly into the Focus half; `usePigMovement` takes `frozenId`
       and emits click-by-id, with no `kind` awareness.
5. [x] Split `domain/lib.rs`'s flat prelude so `Focus` and `AgentSession` are not siblings at the
       crate root (cheap down-payment on Option C).
6. [x] File 055 into `issues/done/` — every box is ticked and its shape is in the code.
7. [x] Update `docs/architecture.md` §Agents as Animals, which still documents the pre-socket
       file-watcher design and is wrong on four counts.
8. [ ] Revisit Option C after the first agent motion slice.
