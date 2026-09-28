# Data Model: Render Domain Seam

This feature adds a transient Rust-owned render read model, one read-only Tauri query, and
generated TypeScript bindings. No persisted schema, existing mutation payload, hook protocol, or
settings format changes.

## Entity: Animal

The flat, immutable value one source domain gives the shared renderer.

| Field | Type | Required | Meaning | Validation/invariant |
|---|---|---:|---|---|
| `id` | string | yes | Stable identity within the current rendered roster | Non-empty and unique across composed Focus and Agent Animals |
| `label` | string | yes | Text displayed beside the concrete Species | Display-only; carries no source identity |
| `species` | Species | yes | Concrete appearance and behavior policy | Explicitly `pig` in this feature; never inferred from source identity |
| `size` | AnimalSize | yes | Fixed or time-varying absolute-pixel presentation | Resolves to a finite value greater than zero and contains no Timer vocabulary |
| `regionId` | string or null | yes | Opaque renderer-region assignment | `null` means full DisplaySpace; non-null is resolved against current regions |
| `motion` | Motion | yes | Current renderer movement behavior | Must be a supported Motion value |

Animal MUST NOT contain source records, a source-domain or Harness discriminator, Pen data,
callbacks, selection state, or mutable movement state.

## Value: AnimalSize

Rust-produced renderer vocabulary for absolute display size.

| Variant | Fields | Resolution |
|---|---|---|
| `fixed` | `px` | Always resolves to `px` |
| `linear` | `fromPx`, `toPx`, `startedAtMs`, `durationMs` | Linear interpolation clamped to the closed interval from start to end |

All pixel values are finite and positive. `durationMs` is positive for a linear curve; Rust
projects a zero-duration Timer directly to a fixed `toPx` value. The TypeScript sampler handles
times before the start and after the end by returning the respective endpoint. The curve is
renderer vocabulary: it never exposes Timer status, preset, ownership, or task association.

## Value: Species

Renderer vocabulary selecting the concrete Animal appearance and behavior profile.

| Value | Status in this feature | Meaning |
|---|---|---|
| `pig` | sole supported value | Preserve the current Pig asset, footprint, motion tuning, direction mapping, and animation behavior |

Species is orthogonal to the source domain and Harness. A Focus and an Agent Session can both
project Pig Animals; a future Claude and Codex session can use the same Species; a future Species
does not imply a different source domain or Harness.

## Value: SpeciesProfile

Rust-owned static physical policy returned once per supported Species in `RenderScene`, not copied
onto every Animal.

| Field/concern | Owner |
|---|---|
| Species key, base size, and movement footprint | Rust Species profile |
| walk speed, friction, minimum-speed fraction, and toss-speed cap | Rust Species profile |
| turn interval and animation-frame cadence | Rust Species profile |
| direction-to-frame mapping, bobbing, sprite-sheet metadata, asset, and CSS | concrete TypeScript Species presentation |
| position, velocity, region confinement, freeze state, drag state, and loop lifecycle | generic TypeScript frame runtime |

This feature has one Pig profile and an exhaustive one-case lookup. It introduces no dynamic
registry, user assignment, persistence, or second Species. Those decisions wait for the Animal
Runtime Core milestone and concrete requirements.

## Value: Motion

Renderer vocabulary for how an Animal moves now.

| Value | Renderer behavior | Source examples |
|---|---|---|
| `walking` | Advance through the normal movement loop | Focus with no expired Timer; working Agent Session |
| `resting` | Stop walking and remain/reconfine within the applicable region | Focus with expired Timer; idle Agent Session |

Motion does not encode *why* the Animal moves that way. A future source may produce a new Motion
without adding that state to every source domain.

## Entity: AnimalRegion

A renderer-only description of one labeled movement region.

| Field | Type | Required | Meaning | Validation/invariant |
|---|---|---:|---|---|
| `id` | string | yes | Opaque key referenced by `Animal.regionId` | Non-empty and unique in the region roster |
| `label` | string | yes | Visible region label | Display-only; contains no Pen object |

Agent Session projection derives regions from unique Pens at the domain edge. Focus projection
emits no regions because `regionId: null` means the full DisplaySpace.

## Entity: AnimalRegionLayout

Transient geometry used by shared movement and region presentation.

| Field | Type | Meaning |
|---|---|---|
| `region` | AnimalRegion | Renderer descriptor being laid out |
| `rect` | Rect | Resolved movement rectangle in DisplaySpace coordinates |
| `hue` | number | Deterministic presentation hue derived from region id |

Layouts are sorted and colored by region id so a region keeps a stable cell as sessions appear or
disappear. Geometry is recomputed by pure Rust scene layout from current display geometry and the
existing maximum region-size setting; it is never persisted.

## Entity: RenderScene

The complete transient Rust read model consumed by the ranch renderer.

| Field | Type | Meaning |
|---|---|---|
| `animals` | Animal array | Independently projected Focus and Agent Animals |
| `regions` | AnimalRegionLayout array | Deduplicated and laid-out renderer regions |
| `speciesProfiles` | SpeciesProfile array | Physical policy for every Species referenced by the scene |

The scene is produced on demand from current stores, settings, and requested display geometry. It
is not persisted. It contains no Focus, Timer, Agent Session, Pen, Harness, Hook Firing, Claude, or
Codex record.

## Projection: Focus to Animal

| Animal field | Source/calculation |
|---|---|
| `id` | `Focus.id` |
| `label` | `Focus.title` |
| `species` | `pig` |
| `size` | Rust `fixed` base size when no Timer exists; otherwise a clamped `linear` absolute-pixel curve derived from the Focus Timer |
| `regionId` | `null` |
| `motion` | `resting` only when the Focus Timer is Expired; otherwise `walking` |

Task Timers do not alter the Focus Animal. Rust constructs the curve once when projecting current
source state; TypeScript only samples it. The projection accepts no Agent Session or Pen data.

## Projection: Agent Session to Animal and Region

| Animal field | Source/calculation |
|---|---|
| `id` | `agent:` plus `AgentSession.id` |
| `label` | `AgentSession.name` |
| `species` | `pig` |
| `size` | fixed base Pig size from the Rust Species profile |
| `regionId` | `AgentSession.pen.id` |
| `motion` | `resting` for Idle; `walking` for Working |

Each distinct session Pen also maps once to `AnimalRegion { id: pen.id, label: pen.name }`; the Rust
scene service deduplicates and lays out those descriptors. The projection accepts no Focus, Timer,
clock, Claude payload, or transport data.

## Entity: FocusSelection

Transient Focus-owned UI state.

| Field | Type | Meaning |
|---|---|---|
| `requestedId` | string or null | Last renderer id explicitly requested through click, tray event, or close |
| `selected` | Focus or null | `requestedId` resolved against the current Focus collection |

`requestedId` is internal hook state, not part of Animal. `selected` is derived on every render.

### Selection transitions

```text
Closed
  └─ request(id matching current Focus) ─> Selected(Focus)

Closed
  └─ request(id not matching Focus) ─────> Closed (request is forgotten)

Selected(Focus)
  ├─ close ──────────────────────────────> Closed
  ├─ Focus removed ──────────────────────> Closed (request is forgotten)
  └─ Focus remains ──────────────────────> Selected(updated Focus)
```

An Agent render id cannot resolve because Agent ids use the `agent:` namespace and the selection
module reads only the Focus collection.

## Relationships

```text
Focus + Timer ───────> Rust Focus projection ───────> Animal[] ─┐
                                                                 ├─> Rust RenderScene service
Agent Session[] ─────> Rust Agent projection ───────> Animal[] ─┤
                                  └───────────────> AnimalRegion[]

Display geometry + settings ──> Rust region layout ──> AnimalRegionLayout[]
Rust Pig policy ─────────────────────────────────────> SpeciesProfile[]

RenderScene ──Tauri/generated types──> TypeScript frame runtime ──> screen
renderer click id ──> FocusSelection(current Focuses) ─────────────> Focus detail or none
```

No relationship crosses directly between Focus and Agent Session. Their projected arrays meet only
in the Rust scene service, and shared consumers depend only on generated renderer values. The future Harness
adapter terminates before `AgentSession`; neither Harness identity nor native Claude/Codex event
data reaches Animal. Species selection also does not encode source-domain or Harness identity.

## Region Resolution Rules

1. `regionId === null`: use all `DisplaySpace.movementRegions`; spawn in
   `DisplaySpace.spawnRegion`.
2. `regionId` matches a current layout: use that single layout rectangle for movement and spawn.
3. `regionId` is temporarily unresolved: fall back to the same DisplaySpace behavior as null until
   the region becomes available.
4. A region disappearing causes the next movement update to use DisplaySpace fallback; it does not
   delete or mutate the Animal.

## Identity and Update Rules

- A Focus retains its raw stable slug id.
- An Agent Session uses `agent:${session.id}`, keeping the render id space distinct even when raw
  source ids match.
- Label, size policy, and motion changes update presentation without respawning movement state.
- Species participates in movement/presentation identity. If a later feature permits Species to
  change for an existing id, that feature MUST explicitly choose whether to reinitialize or
  migrate movement state; this cleanup permits no such transition.
- Region-id changes may reconfine the existing movement state to the new region.
- Removing an Animal removes its movement state; adding a new id creates movement state.
- Changing Focus size does not alter the current base-size movement footprint in this feature.
