# Implementation Plan: Render Domain Seam

**Branch**: `001-render-domain-seam` | **Date**: 2026-09-27 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `specs/001-render-domain-seam/spec.md`

## Summary

Implement ADR-0001's renderer seam using ADR-0003's Rust-owned read model. Independent Rust Focus
and Agent Session projectors produce a generated `RenderScene`; the Rust application layer
composes Animals, lays out regions, and supplies physical Species policy through one read-only
Tauri command. The webview consumes generated renderer values and keeps only frame-local size
sampling, movement state, pointer interaction, sprite presentation, and Focus selection. Make
Species explicit, with Pig as the only current value, without hard-coding Pig policy into generic
TypeScript movement. Preserve visible behavior and existing mutation, persistence, and hook
contracts. Deliver the change as three independently green slices: Rust scene contract and
projectors, scene IPC plus App/selection cutover, then Rust-owned region/Species policy plus the
thin generic frame runtime.

## Technical Context

**Language/Version**: Rust 2021 with MSRV 1.77 for contracts, projections, composition, region
layout, and Species policy; TypeScript 5.9.3 in strict ES2022 mode with React 18.3.1 for generated
contract consumption and frame-local presentation.

**Primary Dependencies**: Existing Rust `serde` and feature-gated `ts-rs` for the scene contract;
Tauri 2.11.0 for the read bridge; React 18.3.1 and Vite 5.4.21 for presentation; Rust tests,
Vitest 2.1.9, and Testing Library for verification. No new third-party runtime dependency.

**Storage**: N/A. No persisted files, settings schemas, or runtime stores change. Generated
Rust-to-TypeScript renderer contracts do change.

**Testing**: Vitest/jsdom unit and component tests, TypeScript type checking, Biome, Knip, and the
repository-wide `task check` gate (including Rust checks and generated-type drift verification).

**Target Platform**: Tauri v2 desktop application; macOS is the primary design target. The changed
Rust read service and webview remain platform-neutral and do not alter Windows or Linux packaging
paths.

**Project Type**: Local-first desktop application with a React/TypeScript renderer and Rust/Tauri
host.

**Performance Goals**: Preserve animation cadence, movement, and drag responsiveness. Source,
settings, or DisplaySpace changes may refresh the Rust scene; ordinary frames perform no Tauri
invoke/event round trip. Focus growth samples a Rust-produced size curve locally without
restarting the movement roster or forcing Agent projection work.

**Constraints**: No visible behavior change; no source-domain records or discriminators in the
renderer contract; Species, source domain, and Harness are independent dimensions and none may be
inferred from another; one new read-only scene IPC and generated contract are allowed, while
persistence, existing mutation IPC, hook transport, plugin, and network behavior remain unchanged;
no per-frame IPC; no Option C crate/lint enforcement; every slice passes `task check` and remains
independently reviewable.

**Scale/Scope**: Two Rust producers (Focus and Agent Session), one composed RenderScene, one
read-only Tauri scene query refreshed by three state/event classes, one shared frame runtime, one
Focus selection flow, one Rust region-layout function, one Pig physical profile, and focused Rust
and TypeScript tests. Pig becomes the one explicit Species value, but no second Species, new
Motion, Harness, screen, or stored entity is introduced.

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-checked after Phase 1 design.*

### Pre-design gate

| Principle or constraint | Result | Evidence |
|---|---|---|
| I. User-Owned Local State | PASS | The plan changes no persistence, network behavior, telemetry, credentials, or integration installation. |
| II. Independent Domains, Shared Renderer | PASS | Independent Rust projectors emit only generated renderer values; neither source domain imports the other. |
| III. Functional Core, I/O at the Edges | PASS | Rust projections, size-curve construction, Species policy, and region layout are pure; Tauri owns the read edge; React owns subscriptions and frame-local effects. |
| IV. Explicit, User-Controlled Integrations | PASS | Harness installation, validation, transport, and diagnostics are untouched. |
| V. Observable Completion in Small Slices | PASS | ADR-0001 plus ADR-0003, three completion promises, focused Rust/TypeScript tests, generated-binding proof, and `task check` provide observable proof. |
| VI. Evidence Is Not Authority | PASS | Requirements come from the constitution, accepted ADR-0001, current `main`, and present owner direction. Issues 051/057/058 are evidence only; stale ADR action checkboxes do not override current code. |
| Product constraints | PASS | Tauri IPC remains the only frontend/core boundary; the read model is generated from Rust; Pig physical policy is not relabeled as generic Animal behavior; no per-frame transport or speculative compatibility layer is added. |

**Gate result**: PASS. No exception or complexity justification is required.

## Project Structure

### Documentation (this feature)

```text
specs/001-render-domain-seam/
├── spec.md
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
├── contracts/
│   └── animal-renderer.md
└── checklists/
    └── requirements.md
```

`tasks.md` is intentionally deferred to `$speckit-tasks`.

### Source Code (repository root)

```text
crates/
└── commands/
    └── src/
        ├── render_scene.rs          # generated DTOs, composition, and scene service
        ├── focus_animals.rs         # pure Focus -> Animal projection
        ├── agent_animals.rs         # pure AgentSession -> Animal/region projection
        ├── species.rs               # Rust-owned physical Pig profile
        └── region_layout.rs         # pure dedupe/order/style/layout

src-tauri/
└── src/
    └── ui_bridge/
        └── mod.rs                   # thin get_render_scene Tauri command

src/
├── types/
│   ├── generated/                   # ts-rs Animal/Scene/Species/Profile bindings
│   ├── display.ts                   # existing DisplaySpace event/fallback geometry
│   └── pig.ts                       # retained hit-test compatibility type
├── api/
│   ├── tauriRenderSceneReader.ts    # scene invoke + source/settings refresh events
│   └── pig.ts                       # existing display, hit-rect, and drag bridge
├── lib/
│   ├── animalSize.ts               # deterministic frame-time curve sampler only
│   ├── animalSize.test.ts
│   ├── animalMovement.ts           # renamed renderer-neutral movement core
│   ├── animalMovement.test.ts
│   └── regions.ts                  # lookup/fallback over Rust-laid-out rectangles
├── hooks/
│   ├── useFocusSelection.ts        # Focus-only requested-id resolution
│   ├── useFocusSelection.test.tsx
│   ├── useRenderScene.ts            # combines DisplaySpace with the scene reader
│   ├── useAnimalMovement.ts         # frame-local movement/effect adapter
│   └── useAnimalMovement.test.ts
├── components/
│   ├── App.tsx                     # scene consumer plus Focus detail composition
│   ├── App.test.tsx
│   ├── AnimalSprite.tsx            # exhaustive Species-to-presentation dispatch
│   ├── PigSprite.tsx               # concrete Pig Species asset remains Pig-specific
│   └── RegionBox.tsx               # draws renderer regions without Pen data

Taskfile.yaml                       # generate/check domain + command-layer TS bindings
```

The old cross-domain and generic-name files are removed as their responsibilities move:
`src/lib/animals.ts`, `src/hooks/useAnimals.ts`, `src/hooks/useAnimalSelection.ts`,
`src/hooks/usePigMovement.ts`, `src/lib/ranchAnimalMovement.ts`, and
`src/lib/focus/animals.ts`, `src/lib/session/animals.ts`, and `src/lib/session/pens.ts`. Exact file
moves occur only when their Rust or thin-renderer replacements are green.

**Structure Decision**: Put renderer DTOs and pure projectors in the existing Rust application
crate, not the business-domain modules and not a new crate. `render_scene` is the backend shell
composition point: it calls independent Focus and Agent projectors, then returns only generated
renderer values. The Tauri bridge stays thin. TypeScript imports generated values and contains no
source projection. This buys Rust authority without prematurely adding ADR-0001 Option C's crate
graph or forcing browser-frame work across IPC.

## Design and Delivery Strategy

### 1. Rust scene contract and independent projectors

Define `Animal`, `AnimalSize`, `Species`, `Motion`, `AnimalRegionLayout`, `SpeciesProfile`, and
`RenderScene` in the Rust application layer with Serde and `ts-rs` generation. Extend the existing
type-generation task to include command-layer exports and drift checks.

Implement two pure projectors. The Focus projector owns raw-id preservation, label, explicit Pig,
whole-display placement, Timer-to-Motion mapping, and Timer-to-`AnimalSize` conversion. A running
Timer becomes a clamped linear absolute-pixel curve; no Timer becomes a fixed size; zero-duration
and expired cases preserve current maximum-size behavior. The Agent projector consumes only
`AgentSession`, owns the `agent:` id namespace, label, Pig, Activity-to-Motion mapping, and Pen id
plus logical region descriptor. Neither projector accepts the other source domain.

Rust unit tests prove serialized shape, independent inputs, raw-id collisions, curve boundaries,
Activity/Motion mapping, explicit Species, and the absence of source records from renderer DTOs.

**Completion promise**: Rust can independently turn either source domain into generated
renderer-only values, with all source-semantic decisions proven before Tauri or React is involved.

### 2. Rust scene service, IPC, and App/selection cutover

Add a Rust `RenderScene` application service that calls the independent projectors and exposes one
read-only Tauri command. The request supplies current renderer area geometry; the service reads
current Focuses, enabled Agent Sessions, and settings, then returns one generated scene. The
webview scene reader refreshes after `focuses-changed`, `agent-sessions-changed`,
`settings-changed`, or `display-space`, but animation frames never invoke the command.

Cut `App` over from raw Agent Sessions and TypeScript projections to the generated scene. Keep the
existing Focus reader because the detail editor needs full user-owned Focus records. Replace
`useAnimalSelection` with `useFocusSelection(focuses)`: renderer clicks report only ids, Agent ids
resolve to no Focus, stale Focus ids clear, and the selected Animal is found in the scene only for
placement/freezing. The Agent debug window retains its existing source-specific reader.

Delete the hand-written Animal union and TypeScript source projectors only after the Rust command,
generated bindings, reader, App integration, and focused tests are green.

**Completion promise**: The running ranch renders the Rust scene, only Focuses open details, and
no TypeScript production module translates Focus or Agent Session state into Animal state.

### 3. Rust regions and Species policy; thin frame runtime

Move unique Pen derivation, deterministic ordering, stable hue choice, and rectangle layout into
pure Rust. The scene service lays out renderer regions from current settings and the requested
display area; it never returns a Pen. `regionId: null` still uses DisplaySpace, a known id uses the
supplied rectangle, and an unresolved id temporarily falls back to DisplaySpace.

Move Pig's physical values—base size/footprint, walk speed, friction/minimum speed, turn interval,
animation cadence, and toss-speed cap—into the Rust `SpeciesProfile` included in the scene. Rename
the TypeScript integrator and hook to Animal vocabulary and make them consume generated profiles
and region rectangles. They retain only position/velocity, frame-time curve sampling, confinement,
freeze/drag/toss mechanics, `requestAnimationFrame`, and hit-rectangle reporting.

`AnimalSprite` dispatches exhaustively on generated Species and currently delegates only to
`PigSprite`. Pig-specific webview code retains the asset import, sprite-sheet row mapping, bobbing,
CSS, and DOM presentation. This is a one-case generated profile, not dynamic registration,
Species settings, or persistence.

**Completion promise**: Existing Pig movement and regions are unchanged, Rust owns every stable
projection/layout/physical-policy decision, and TypeScript shared runtime contains no Focus,
Timer, Agent Session, Pen, Harness, or hard-coded Pig physics.

## Protected Extension Seams and Follow-on Milestones

The cleanup preserves three independent axes. They meet through adapters; no axis is encoded in or
inferred from another:

```text
Work Management                         Agent Activity
Focus + Task + Timer                    Claude / later Codex native events
        │                                           │
        └─> Rust Focus projection         Harness adapter ─> Agent Session
                    │                                  │
                    └──────> Rust RenderScene <────────┘
                                   │
                        Rust Species/region policy
                                   │
                          webview frame runtime
                                   │
                                renderer
```

The current feature is the first cleanup milestone only. It adds no Rust ingestion work and no
new product behavior. Subsequent work receives separate specs, tasks, and durable ADRs rather than
being folded into ADR-0001 implementation:

1. **Animal Runtime Core**: define the durable Species/profile boundary for additional Species and
   richer animated-element behavior. Decide assignment, selection, persistence, profile lookup,
   animation vocabulary, and movement-state migration only when a second Species or new behavior
   supplies concrete requirements. The explicit Pig field/profile in this cleanup is the seam,
   not a speculative registry. Separately prototype a Rust fixed-step movement engine with batched
   frames only if measurements justify moving the current synchronous webview loop.
2. **Claude Native Event Schema**: create a small provider-native Rust crate that models the
   complete *documented hook payload* surface, retains unknown event variants and fields, returns
   structured parse errors, and has no Ranch, Animal, Pen, Tauri, transport, or storage concepts.
   Snapshot sanitized fixtures and the tested Claude Code version. Transcript/session JSONL
   entries remain opaque values because Anthropic does not publish them as a stable exhaustive
   schema.
3. **Harness-neutral Session Core and Claude Adapter**: define provider-scoped Harness/session
   identity and a stable observation/change interface, then map typed Claude events into it. Parse
   once, reduce to `AgentSession` plus concrete session changes, and give diagnostics only redacted
   summaries. Full parsing does not authorize persisting prompt, tool, transcript, or assistant
   content.
4. **Harness-aware Transport Cutover**: evolve or supersede ADR-0002's frame envelope with Harness
   identity, native event name, send time, and versioning while preserving the installed plugin and
   socket delivery model. Cut the live store and journal over from raw Claude strings to neutral
   observations.
5. **Codex Adapter**: model Codex-native events behind the same neutral interface. Use this second
   provider to validate the abstraction before deciding whether either crate should be published
   externally.

Focus/Task/Timer work proceeds on the Work Management roadmap and never waits on these Harness
milestones merely because both applications can project Animals. Likewise, Agent ingestion does
not own Species choice or animation behavior.

### Compatibility and migration

This is an internal, unshipped read-model replacement, so no data migration or dual-read
compatibility layer is needed. The feature adds one read-only render-scene command and generated
renderer types. Existing mutation commands, Focus/session diagnostic reads, events, persisted
state, settings keys, hook protocol, and plugin behavior remain compatible. The old TypeScript
projection path is removed after cutover rather than retained as a second authority. The generated
type drift check, TypeScript compilation, and unused-code checks prove that only the Rust contract
remains.

## Post-design Constitution Check

| Gate | Result | Design proof |
|---|---|---|
| Local ownership and privacy | PASS | The new read command is local-only and carries no prompt/tool content; persistence, network, and integration behavior do not change. |
| Domain independence | PASS | Rust producers share only `Animal`/scene values; selection is Focus-only; Species, Harness, and source domain remain orthogonal. |
| Functional core and explicit edges | PASS | Rust projections, size policy, dedupe, layout, and Species policy are pure; Tauri is the read edge; subscriptions/rAF/pointer work stays in the webview. |
| Explicit integrations | PASS | No integration behavior is in scope. |
| Small observable slices | PASS | Three ordered slices each have one completion promise, focused regression tests, and the full gate. |
| Artifact authority | PASS | The plan records the stale checkboxes, relies on verified current code, and does not promote historical issue text. |
| Simplicity | PASS | The existing commands crate gains one read-model service and generated contract; no new crate, registry, per-frame protocol, or speculative enforcement mechanism is introduced. |

**Post-design gate result**: PASS. Phase 1 introduces no constitutional violation.
