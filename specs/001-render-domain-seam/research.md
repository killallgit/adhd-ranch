# Phase 0 Research: Render Domain Seam

## Source Classification

- **Authoritative**: `.specify/memory/constitution.md` as the current approved planning direction;
  accepted `docs/adr/0001-two-apps-over-one-renderer.md`; accepted
  `docs/adr/0003-rust-owned-render-scene.md`; the approved feature `spec.md`.
- **Verified observation**: Code and tests on current `main` at `c188eea`.
- **Present owner direction**: Plan ADR-0001 now, preserve the seam for additional Species and
  richer Animal behavior, keep Claude/Codex ingestion separate from Focus/Timer work, and place as
  much non-frame-local work as practical in Rust.
- **Historical evidence only**: Issues 051, 057, and 058. Their useful detail was rechecked against
  current code and restated here; their unchecked or checked boxes carry no authority.
- **Known documentation mismatch**: ADR-0001 action items 3 and 4 are checked, but current `main`
  still contains the source-domain Animal union and shared selection. Planning follows the code,
  not those stale completion marks.

## Decision 1: Use `Animal` as the Flat Renderer Contract

**Decision**: Replace `FocusAnimal | SessionAnimal` with the glossary's render-layer `Animal`:
`id`, `label`, `species`, renderer-only `size`, `regionId`, and `motion`. `AnimalSize` is either a
fixed absolute pixel value or a clamped linear absolute-pixel curve. Define `Species` initially as
`pig` and `Motion` initially as `walking | resting`. Rust owns and generates the serialized
contract. It imports no Focus, Timer, Agent Session, Harness, Hook Firing, or Pen record.

**Rationale**: ADR-0001 settles the seam at the renderer and gives the first five source-neutral
fields (`docs/adr/0001-two-apps-over-one-renderer.md:88`). The owner's 2026-09-27 clarification and
the existing Species definition add the sixth renderer field so the seam does not assume Pig
forever. `CONTEXT.md:19` subsequently settles the final name as Animal and rejects RanchAnimal and
Sprite for the general concept. Current
`src/types/animal.ts:1` still embeds both source records, so the accepted decision is not present
on `main`.

**Alternatives considered**:

- Keep or widen the discriminated union: rejected because every shared consumer must learn source
  identity and future Agent motion changes remain coupled to Focus.
- Name the type `RanchSprite`: rejected because that ADR name was illustrative and the canonical
  glossary now specifies Animal.
- Put callbacks, selection, or movement state on Animal: rejected because those are renderer
  inputs/outputs or runtime state, not projection data.

## Decision 2: Keep Two Pure Rust Projections and Compose in a Scene Service

**Decision**: Rust Focus code alone maps `Focus[]` to `Animal[]`; Rust Agent code alone maps
`AgentSession[]` to `Animal[]` and logical renderer regions. A Rust application service calls both
projectors and returns one generated `RenderScene` through Tauri. `App.tsx` consumes that scene; it
does not concatenate source-domain projections. Delete the TypeScript `projectAnimals`,
`projectFocusAnimals`, `projectSessionAnimals`, and `animalPen` helpers.

**Rationale**: The current projection rules are already largely separate in
`src/lib/focus/animals.ts:6` and `src/lib/session/animals.ts:8`, but `src/lib/animals.ts:8` combines
both domains and a clock in one interface. Composition belongs at the backend shell edge already
holding the authoritative sources. A render-only size curve keeps Timer interpretation in the Rust
Focus projector without giving Agent projection or the frame runtime a Timer.

**Alternatives considered**:

- Retain TypeScript `projectAnimals` as a convenience: rejected because it leaves a second source
  of truth and reintroduces a cross-domain module whose signature owns both source types.
- Return only a current scalar size from Rust: rejected because growth would freeze between source
  events or require continuous backend polling.
- Namespace both source id spaces: rejected for this slice because retaining raw Focus ids
  preserves tray/detail semantics; `agent:${session.id}` already gives collision-safe render ids.

## Decision 3: Add a Render-Only Region Descriptor

**Decision**: Rust Agent projection emits unique logical region descriptors derived from Pens.
Pure Rust scene layout sorts, styles, and resolves those descriptors against requested display
geometry and the current maximum-size setting. The scene returns `AnimalRegionLayout` values;
TypeScript movement uses only `regionId` and supplied rectangles and never imports `Pen`.

**Rationale**: `regionId` identifies placement but does not carry the label needed to draw a
region. Current `src/lib/session/pens.ts:1` and `src/hooks/usePigMovement.ts:18` carry generated Pen
records into shared movement. A two-field renderer descriptor preserves labels and deterministic
layout without copying Pen policy into the Animal contract. It implements the ADR statement that
the Agent half computes regions and the renderer receives regions.

**Alternatives considered**:

- Put a Pen or `regionLabel` on every Animal: rejected because it leaks or duplicates group data
  and violates the flat renderer-only Animal contract.
- Pass `Pen[]` separately to shared movement: rejected because shared movement would still depend
  on an Agent-domain type.
- Keep pixel layout in the movement hook: rejected by the Rust-ownership requirement because the
  algorithm is pure policy and its DisplaySpace input changes only on display/settings events.

## Decision 4: Preserve Current Region Fallback Semantics

**Decision**: `regionId: null` means the full DisplaySpace. A known non-null id uses its resolved
region. A temporarily unresolved id falls back to DisplaySpace for spawn and movement until its
layout is available.

**Rationale**: Current helpers already provide this behavior at
`src/hooks/usePigMovement.ts:90-105`, including the period before settings and maximum Pen size
arrive. Retaining it avoids stranding or hiding Agent animals during startup.

**Alternatives considered**:

- Hide animals with unresolved regions: rejected as a visible regression.
- Freeze them until layout exists: rejected because it introduces a new Agent-only movement state
  and startup behavior outside the accepted ADR.

## Decision 5: Move Selection Fully Into Focus

**Decision**: Replace `useAnimalSelection(animals)` with `useFocusSelection(focuses)`, returning
`Focus | null`. Renderer clicks report only a render id; Focus selection resolves that id against
the current Focus collection and forgets it when it stops resolving. Movement accepts only
`frozenId`.

**Rationale**: ADR-0001 explicitly assigns click meaning to Focus and makes the renderer operate by
id. Current `src/hooks/useAnimalSelection.ts:14` accepts both domains, then discards Agent animals
through a `kind` test. The existing stale-id behavior is valuable and remains unchanged.

**Alternatives considered**:

- Keep selection in App as ad hoc state: rejected because the subscription, stale-id transition,
  and Focus-only invariant form one coherent module.
- Make Agent animals selectable: rejected by the accepted ADR and current feature scope.

## Decision 6: Convert Timer State to a Rust-owned Absolute-pixel Size Curve

**Decision**: Rust Focus projection converts Timer state to `AnimalSize::Fixed` or a clamped
`AnimalSize::Linear { from_px, to_px, started_at_ms, duration_ms }`. Rust Agent projection uses a
fixed base Pig size. TypeScript has one deterministic sampler that resolves current absolute pixels
for sprite drawing, detail anchoring, and hit rectangles. The curve stays outside movement roster
identity, and collision/spawn geometry uses the Rust Species footprint.

**Rationale**: Drawing and hit testing currently multiply scale separately
(`src/components/PigSprite.tsx:49`, `src/hooks/usePigMovement.ts:127`). A render curve removes Timer
knowledge while avoiding per-frame IPC. Excluding the sampled pixel value from roster identity
preserves the current non-restarting animation loop. Changing scaled-animal collision geometry
would be a separate visible behavior decision.

**Alternatives considered**:

- Keep scale or Timer timestamps as hand-authored TypeScript projection data: rejected because it
  makes the renderer interpret Focus rules and leaves policy outside Rust.
- Emit a new scalar size from Rust every frame: rejected because serialization and asynchronous
  Tauri ordering are unnecessary for deterministic interpolation.
- Use changing size for spawn/clamp collision geometry now: rejected as scope expansion without
  an accepted behavior decision.

## Decision 7: Rename Only Generic Movement Vocabulary

**Decision**: Rename general `usePigMovement` and `RanchAnimal*` concepts to Animal vocabulary.
Retain `PigSprite`, the sprite sheet, Pig presentation styles, and Pig-named Tauri IPC wrappers
where they are Species-specific or released compatibility contracts.

**Rationale**: The glossary requires Animal for shared movement and reserves Pig for the concrete
Species. Renaming Rust/Tauri commands would expand the change across a stable boundary for no user
value. The package is private, but the product constitution still protects released protocols and
requires the simplest design.

**Alternatives considered**:

- Rename all Pig-named IPC in the same feature: rejected because it creates a cross-language
  compatibility migration unrelated to the domain seam.
- Keep `RanchAnimal*` aliases: rejected because unused-code checks should prove the old general
  vocabulary is gone rather than preserve thin forwarding layers.

## Decision 8: Verification and Platform Gates

**Decision**: Add Rust projection, serialization, collision-id, size-curve, region-layout,
Species-profile, and scene-service tests plus focused TypeScript sampler, Focus-selection,
movement-adapter, and App tests. Each slice runs focused tests and `task check`. Rust/Tauri paths
trigger the existing Windows CI job; no unsupported local cross-compilation is required.

**Rationale**: `Taskfile.yaml:142` defines `task check` as lint, typecheck, Rust/frontend tests, and
generated-contract drift. CI runs it on macOS for every PR. The Windows workflow is path-filtered
to exactly the Rust/Tauri paths this plan changes, so it becomes a required CI signal. Existing
tests already cover movement, dragging, timer visuals, regions, stale selection, and Agent clicks;
new assertions prove backend ownership and the boundary itself rather than only visible behavior.

**Alternatives considered**:

- Run only frontend tests: rejected because the constitution requires the repository gate.
- Add release packaging: rejected because no platform or packaging contract changes.

## Decision 9: Make Species Explicit Without Building a Species System

**Decision**: Add `species: "pig"` to every current Animal. Route concrete presentation and
behavior through an exhaustive Species boundary, with Pig as the sole case. Keep a second Species,
selection, persistence, user settings, dynamic registries, and new animation behavior out of this
feature.

**Rationale**: `CONTEXT.md:23` says Species determines how an Animal looks and moves. Leaving
Species implicit would rename Pig-specific assumptions as generic Animal behavior and require the
renderer contract to change as soon as the next Species arrives. The small discriminator makes
the three independent axes explicit: source domain (Focus or Agent), Harness (Claude or Codex),
and Species (Pig or later values). None can be inferred from another.

**Alternatives considered**:

- Keep the five-field contract and infer Pig because it is the only current asset: rejected
  because the shared renderer would have no deliberate extension point and future Species would
  again cut across every producer.
- Build a runtime Species registry now: rejected because one Species provides no concrete
  requirements for registration, persistence, customization, or plugin loading.
- Put Species in Focus or Agent Session: rejected because it is renderer/runtime vocabulary, not
  ownership or Harness identity.

## Decision 10: Separate Generic Animal Runtime From Pig Policy

**Decision**: The browser's generic movement runtime owns coordinates, velocity, region
confinement against supplied rectangles, freezing, drag orchestration, and loop lifecycle. The
Rust Pig profile owns the current footprint, speed, friction, turn cadence, animation cadence, and
toss cap. Pig webview modules retain direction/frame mapping, sprite metadata, assets, CSS, and
DOM presentation. Generic code selects physical policy only from explicit Species.

**Rationale**: Current `src/lib/ranchAnimalMovement.ts` hard-codes the Pig footprint and movement
tuning while `PigSprite.tsx` owns a Pig-specific four-direction sprite contract. Moving the
constants into a generated Rust profile, not changing them, preserves behavior and prevents the
first implementation from silently becoming the definition of every future Species.

**Alternatives considered**:

- Rename every Pig constant to Animal and defer the split: rejected because that creates false
  genericity and makes later Species behavior a breaking refactor.
- Put all animation metadata on each Animal: rejected because static Species policy would be
  duplicated across the roster and mixed with transient projection data.

## Decision 11: Protect a Two-layer Harness Boundary After This Cleanup

**Decision**: The current cleanup may add a Rust render read model, but it MUST NOT refactor native
hook ingestion. Its Rust Agent projector terminates at a harness-neutral `AgentSession`. A later
provider-native schema module will parse Claude event DTOs and preserve unknown data; a separate
Ranch-owned adapter will translate those DTOs to provider-neutral observations, `AgentSession`, and
concrete session changes. Codex will implement the same neutral interface later.

**Rationale**: The live path currently maps five installed Claude hooks to four Ranch verbs,
passes `verb\n` plus raw JSON through the socket, parses the payload as `serde_json::Value`, reads
only `session_id` and `cwd`, and constructs Ranch records immediately. `Stop` and `StopFailure`
both collapse to Idle. The store and journal then parse the raw provider payload again; the journal
correctly avoids retaining it because prompt and tool payloads may contain user text. This loses
native event meaning, couples transport/storage to Claude JSON, and leaves no provider-scoped
identity for Codex.

The later target flow is:

```text
native JSON -> provider schema DTO -> Harness adapter -> neutral observation
            -> session reducer -> AgentSession + SessionChange -> Animal projection
```

**Alternatives considered**:

- Parse complete Claude records directly into `AgentSession`: rejected because provider evolution
  and future Codex fields would define the Ranch domain model.
- Keep raw JSON as the shared abstraction: rejected because every consumer would repeat parsing,
  redaction, and vendor-specific branching.
- Put Harness or native event fields on Animal: rejected because rendering must not encode how an
  observed session was discovered.

## Decision 12: Own the Claude Hook Schema; Keep Transcript Records Opaque

**Decision**: Plan a small independent workspace crate for the complete *documented Claude hook
payload* schema, with an unknown-event fallback, unknown-field retention, structured errors, and
sanitized conformance fixtures. Do not adopt an existing crate as the contract boundary. Treat
Claude transcript/session-entry JSON as ordered opaque JSON with narrowly typed extractors only
for separately approved enrichment work.

**Rationale**: The [official Claude hooks reference](https://code.claude.com/docs/en/hooks)
documents more than 30 event names and shared fields including session, prompt, transcript,
scratchpad, permission, effort, and subagent identity. That surface changes over time. Anthropic's
[session-storage contract](https://code.claude.com/docs/en/agent-sdk/session-storage) deliberately
types transcript entries only by `type` plus arbitrary JSON and tells adapters to preserve them in
order. “Complete schema” can therefore be a truthful claim for the documented hook input surface,
not for the private/evolving transcript format.

No maintained Rust crate found on 2026-09-27 provides that complete boundary:

- [`claude-codes`](https://crates.io/crates/claude-codes) tracks the CLI stream/control protocol;
  its hook callback input remains raw JSON.
- [`anthropic-agent-sdk`](https://crates.io/crates/anthropic-agent-sdk) is community-maintained and
  models an older, much smaller hook subset.
- [`coding-agent-hooks`](https://crates.io/crates/coding-agent-hooks) has a promising Claude/Codex
  adapter shape but only a partial normalized event union, an unstable API, and a Rust requirement
  above this workspace's MSRV.
- `claude-hooks` and `agent-hooks` manage registration/configuration rather than the complete
  payload schema.
- The [official Agent SDK overview](https://code.claude.com/docs/en/agent-sdk/overview) provides
  Python and TypeScript SDKs, not an official Rust schema crate.

**Alternatives considered**:

- Depend on the closest community crate and fill its gaps locally: rejected because its types
  would become an incomplete public boundary and upstream compatibility is not demonstrated.
- Exhaustively model transcript JSONL now: rejected because there is no stable published schema;
  it would create brittle coupling and encourage retention of sensitive content.
- Persist every parsed field: rejected because full typed parsing is not permission to log or
  retain prompts, tools, transcripts, or assistant content.

## Decision 13: Rust Owns the Render Scene; TypeScript Owns Frame Mechanics

**Decision**: Adopt ADR-0003. Put the generated renderer DTOs, both independent source
projections, scene composition, region layout, and physical Species policy in the existing Rust
commands/application crate. Expose one read-only scene command. Keep size-curve sampling,
transient position/velocity, `requestAnimationFrame`, pointer capture, drag/toss input, assets, and
DOM drawing in TypeScript.

**Rationale**: Current `main` already makes Rust authoritative for Focus/Timer records,
AgentSession lifecycle, settings, monitor geometry, and OS hit testing, but then moves projection,
timer growth, Pen layout, and Pig physics into TypeScript. Moving those stable semantic and policy
decisions to Rust gives them one implementation and lets the existing `ts-rs` drift gate enforce
the boundary.

The 60 Hz loop is the important exception. Browser positions, pointer events, CSS pixels, and
sprite frames must update synchronously. Driving them through asynchronous Tauri invokes/events
would add serialization and ordering latency. A complete Rust movement engine could be coherent
only if it owns fixed-step state, hit rectangles, drag protocol, and batched frame emission; that
requires a measured Animal Runtime prototype rather than a partial migration in this cleanup.

**Alternatives considered**:

- Keep ADR-0001's frontend-only tactic: rejected by present owner direction and because it leaves
  stable policy beside the pixels instead of its authoritative Rust inputs.
- Put renderer DTOs and projection methods directly in the business-domain modules: rejected
  because presentation is an application read model, not Focus or Agent Session behavior.
- Add a new Rust crate immediately: rejected because the existing commands/application crate is a
  sufficient ownership boundary; crate enforcement remains ADR-0001 Option C follow-up work.
- Move every frame into Rust now: rejected until a fixed-step prototype proves frame cadence,
  multi-display behavior, and drag latency.

## Follow-on Milestone Order

1. Finish this renderer cleanup with explicit Species and a neutral Agent projection boundary.
2. Define and fixture-test the Claude-native hook schema crate.
3. Decide the neutral Harness observation/session-change interface and Claude adapter in a new
   ADR.
4. Version the transport envelope with Harness identity and cut storage/journaling over to parse
   once plus redacted diagnostics.
5. Add richer Claude behavior without leaking provider types into Agent Session rendering.
6. Add the Codex-native adapter; only then decide whether to publish the schema/core crates.
7. Advance additional Species and animation behavior under its own ADR/spec, independently of the
   Harness milestones and the Focus/Timer roadmap.

## Resolved Clarifications

All planning unknowns are resolved. No open planning question remains.
