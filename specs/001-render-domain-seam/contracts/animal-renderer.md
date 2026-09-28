# Contract: Rust Render Scene and Browser Frame Runtime

**Status**: Design contract for `001-render-domain-seam`

**Scope**: Read-only Rust-to-TypeScript Tauri contract plus the internal browser runtime boundary.
Rust Serde/`ts-rs` declarations are authoritative. The TypeScript shapes below describe their
generated serialized form; they MUST NOT be maintained as duplicate hand-written declarations.

## Generated Scene Values

```ts
export type Species = "pig";

export type Motion = "walking" | "resting";

export interface RenderRect {
  readonly x: number;
  readonly y: number;
  readonly w: number;
  readonly h: number;
}

export type AnimalSize =
  | { readonly kind: "fixed"; readonly px: number }
  | {
      readonly kind: "linear";
      readonly fromPx: number;
      readonly toPx: number;
      readonly startedAtMs: number;
      readonly durationMs: number;
    };

export interface Animal {
  readonly id: string;
  readonly label: string;
  readonly species: Species;
  readonly size: AnimalSize;
  readonly regionId: string | null;
  readonly motion: Motion;
}

export interface AnimalRegionLayout {
  readonly id: string;
  readonly label: string;
  readonly rect: RenderRect;
  readonly hue: number;
}

export interface SpeciesProfile {
  readonly species: Species;
  readonly baseSizePx: number;
  readonly movementFootprintPx: number;
  readonly walkSpeedPxPerSecond: number;
  readonly friction: number;
  readonly minimumSpeedFraction: number;
  readonly maximumTossSpeedMultiplier: number;
  readonly turnMinMs: number;
  readonly turnMaxMs: number;
  readonly frameIntervalMs: number;
}

export interface RenderScene {
  readonly animals: readonly Animal[];
  readonly regions: readonly AnimalRegionLayout[];
  readonly speciesProfiles: readonly SpeciesProfile[];
}
```

Field serialization may use explicit Serde/`ts-rs` renames, but the generated binding and actual
JSON payload MUST agree without a TypeScript mapping layer.

## Animal Invariants

- `id` is non-empty and unique across the composed scene.
- `label` is display text and communicates no source-domain or Harness identity.
- `species` explicitly selects physical and visual policy; Pig is the sole value in this feature.
- `size` resolves to a finite positive absolute CSS-pixel value and contains no Timer vocabulary.
- `regionId: null` means the full DisplaySpace.
- A non-null `regionId` is opaque and resolves only against current scene regions.
- `motion` expresses renderer behavior, not Timer Status or Session Activity.
- Animal contains no Focus, Task, Timer, Agent Session, Harness, Hook Firing, Pen, Claude, or Codex
  record.
- Species MUST NOT be inferred from id prefix, label, region, size, motion, source domain, or
  Harness.

## Size Resolution

Rust owns conversion from source state to `AnimalSize`:

- no Focus Timer -> fixed Pig base size;
- running Focus Timer -> linear curve from base to current maximum, using the Timer start and
  duration;
- expired Focus Timer -> the same clamped curve, which resolves to maximum after its end;
- zero-duration Timer -> fixed maximum size;
- Agent Session -> fixed Pig base size.

The TypeScript frame runtime implements one deterministic sampler:

- fixed -> `px`;
- linear before `startedAtMs` -> `fromPx`;
- linear after `startedAtMs + durationMs` -> `toPx`;
- otherwise -> clamped linear interpolation.

Sampling is presentation mechanics, not source projection. It MUST NOT accept a Timer or duplicate
the source-to-curve rules. Size changes do not alter Animal roster identity or restart movement.

## Projection Mappings

| Rust producer | `id` | `label` | `species` | `size` | `regionId` | `motion` |
|---|---|---|---|---|---|---|
| Focus | Focus id | Focus title | `pig` | Timer-derived fixed/linear curve | `null` | expired Focus Timer -> `resting`; otherwise `walking` |
| Agent Session | `agent:` + Session id | Session name | `pig` | fixed Pig base size | Pen id | Idle -> `resting`; Working -> `walking` |

The Agent projector MUST NOT receive Focus, Task, Timer, Claude payload, native event, or transport
frame data. The Focus projector MUST NOT receive Agent Session or Pen data.

## Region Layout

Rust owns unique Pen derivation, ordering, hue choice, and rectangle layout. Pens are converted to
renderer descriptors before layout and never appear in the scene.

The scene command receives current overlay-local display area geometry. It reads the current
maximum-region-size setting itself. Layout preserves current behavior:

- regions are unique and sorted by id;
- hue is stable for a given id;
- cells are centered after maximum-size capping;
- no regions yields an empty region list.

Browser resolution rules remain:

| Animal input | Movement/spawn geometry |
|---|---|
| `regionId === null` | current `DisplaySpace.movementRegions` / `spawnRegion` |
| known non-null `regionId` | matching Rust-supplied region rectangle |
| unresolved non-null `regionId` | current DisplaySpace fallback |

The fallback preserves startup and event-order behavior; it is not permission for TypeScript to
inspect Pen data or lay out regions again.

## Species Ownership

The Rust Pig profile supplies stable physical values: base size, movement footprint, speed,
friction, minimum speed, toss cap, turn range, and frame cadence. Generic TypeScript movement
receives the profile selected by explicit Species and defines none of those Pig values itself.

Pig TypeScript presentation owns only the sprite asset, sprite-sheet row/direction mapping,
bobbing, CSS, and DOM rendering. `AnimalSprite` dispatches exhaustively by generated Species. This
feature adds no dynamic registry, Species selection, settings, persistence, or second Species.

## Tauri Read Contract

Conceptually:

```ts
interface RenderSceneRequest {
  readonly area: RenderRect;
}

type GetRenderScene = (request: RenderSceneRequest) => Promise<RenderScene>;
```

The actual command is `get_render_scene`. The Rust application service obtains current Focuses,
enabled Agent Sessions, and settings; calls each independent projector; lays out regions; attaches
the Pig profile; and returns the scene. The Tauri bridge performs no projection policy.

The webview refreshes the scene after:

- `focuses-changed`;
- `agent-sessions-changed`;
- `settings-changed`;
- `display-space`, using the new current area.

These are invalidation signals, not scene payloads. Animation frames and pointer moves MUST NOT
invoke `get_render_scene` or require backend frame events.

## Browser Runtime Input and Output

The shared browser runtime consumes:

```ts
interface AnimalFrameRuntimeInput {
  readonly scene: RenderScene;
  readonly displaySpace: DisplaySpace;
  readonly frozenId: string | null;
  readonly nowMs: number;
}
```

It may keep transient position, velocity, drag, and animation-frame state keyed by Animal id. It
reports click interaction by render id only:

```ts
type OnAnimalClick = (animalId: string) => void;
```

App routes that id to browser-local Focus selection. The runtime compares `frozenId` only by
equality and does not know why an Animal is frozen.

## Compatibility

The following existing contracts remain unchanged:

- Focus and Timer mutation commands and persisted Markdown/sidecar formats;
- Agent hook frames, socket transport, plugin installation, and diagnostics reads;
- Pig-named rectangle and drag commands used by the Rust OS hit tester;
- `display-space`, Gather, and tray/detail event meanings;
- settings keys and stored schema.

This feature adds only the read-only render-scene command and generated scene values. Existing
Focus reads remain for details/editing; Agent Session reads remain for diagnostics.

## Prohibited Dependencies

Rust renderer DTOs MUST NOT contain source-domain records. Each Rust projector may import only its
own source domain plus renderer DTOs. Rust scene composition may call both projectors but MUST NOT
let either projector inspect the other's input.

Generic TypeScript movement, layout lookup, and presentation modules MUST NOT import or accept:

- Focus, Task, Timer, Agent Session, Hook Firing, or Pen types;
- Claude/Codex payloads, native event names, raw transport frames, or source/Harness `kind` values;
- storage, filesystem, network, or integration adapters;
- hard-coded Pig physical constants already supplied by `SpeciesProfile`.

Species presentation modules may import generated renderer values and browser assets, but MUST NOT
import Focus, Timer, Agent Session, Harness, Pen, or transport types.
