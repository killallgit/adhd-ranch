# ADR-0003: Rust owns the render scene; TypeScript owns frame mechanics

**Status:** Accepted
**Date:** 2026-09-27
**Deciders:** Ryan (project owner)

## Context

ADR-0001 correctly moves the shared seam from the Focus and Agent Session domains to a neutral
renderer contract. Its recommended implementation tactic was intentionally frontend-only: two
TypeScript projections, no Rust or IPC changes, and browser-owned region policy.

The project owner has since clarified a stronger architectural constraint: the Rust backend should
perform as much actual work as practical. Current `main` does the opposite at the render boundary:
TypeScript maps both source domains to Animals, computes timer growth, namespaces Agent ids,
deduplicates and lays out Pens, and defines Pig movement policy. Rust already owns the source
records, timers, settings, monitor geometry, Agent Session lifecycle, and OS hit testing.

Moving the complete animation loop into the Tauri backend is not automatically an improvement.
Positions, pointer capture, drag input, CSS pixels, sprite frames, and `requestAnimationFrame`
cadence are synchronous webview concerns. Sending every frame across asynchronous IPC would add
serialization, ordering, and latency problems. A backend movement engine would need to own the
whole fixed-step simulation and emit batched frames; that is a separate design and benchmark.

This ADR refines ADR-0001's implementation placement. It does not change ADR-0001's domain seam:
Focus and Agent Session remain independent applications that meet only through renderer values.

## Decision

Rust owns the render read model and all source-semantic projection work.

The Rust application layer will expose a read-only `RenderScene` containing:

- independently projected `Animal` values;
- laid-out renderer regions derived from Agent Pens, settings, and the requested display area;
- physical Species profiles used by the frame runtime.

Rust owns:

- the serialized `Animal`, `AnimalSize`, `Species`, `Motion`, region, Species-profile, and
  `RenderScene` contracts;
- generated TypeScript bindings for those contracts;
- Focus-to-Animal and Agent-Session-to-Animal projections;
- render-id namespacing, labels, Species choice, Motion choice, region assignment, and region
  deduplication/order/layout;
- conversion of Timer state into a renderer-only absolute-pixel size curve;
- physical Pig policy such as footprint, walk speed, friction, turn intervals, and animation
  cadence;
- composition of both independent projections into one scene behind a read-only Tauri command.

TypeScript owns only browser-local renderer mechanics:

- sampling a Rust-provided size curve at the current frame time;
- transient position and velocity, `requestAnimationFrame`, confinement against supplied region
  rectangles, freezing, and drag/toss input;
- Focus selection and detail-card state;
- concrete assets, CSS, sprite-sheet row mapping, and DOM presentation;
- reporting current hit rectangles to the existing Rust OS hit tester.

`AnimalSize` is a renderer value rather than a Timer leak. It supports at least a fixed absolute
pixel size and a clamped linear absolute-pixel curve. Rust chooses the curve and its parameters;
TypeScript performs only deterministic interpolation. This avoids continuous backend polling or
per-frame IPC while keeping Timer semantics in Rust.

The render command refreshes on Focus, Agent Session, settings, and DisplaySpace changes. Existing
Focus commands remain for detail/edit workflows, and existing Agent Session commands remain for
diagnostics. The render scene carries no Focus, Timer, Pen, Harness, Claude, or Codex record.

## Options Considered

### Option A: Keep the renderer seam frontend-only

| Dimension | Assessment |
|---|---|
| Complexity | Low |
| Backend ownership | Low |
| Contract drift | Hand-maintained TypeScript contract |
| Frame performance | Good |

**Pros:** Smallest diff; matches ADR-0001's original implementation tactic.

**Cons:** Leaves domain projection, timer growth policy, region policy, and Species physics in the
webview even though their authoritative inputs already live in Rust. Rejected by present owner
direction.

### Option B: Rust render scene plus browser frame runtime

| Dimension | Assessment |
|---|---|
| Complexity | Medium |
| Backend ownership | High for semantics and policy |
| Contract drift | Prevented by generated bindings |
| Frame performance | Preserves synchronous browser loop |

**Pros:** Rust becomes authoritative for the read model and policies; the renderer receives only
what it needs; no per-frame IPC; Focus, Harness, and Species remain independent dimensions.

**Cons:** Adds a read-only IPC contract, generated types, and a small deterministic size sampler in
TypeScript.

### Option C: Move the complete movement engine into the Tauri backend now

| Dimension | Assessment |
|---|---|
| Complexity | High |
| Backend ownership | Maximum |
| Contract drift | Low |
| Frame performance | Unknown without prototype |

**Pros:** One Rust simulation could own movement and hit-test state.

**Cons:** Requires a backend fixed-step loop, batched frame events, drag-input protocol, recovery
from delayed frames, and multi-display lifecycle work. A partial migration would be worse than the
current synchronous loop. Deferred to the Animal Runtime milestone and a measured prototype.

## Trade-off Analysis

Option B places computation according to meaning and latency rather than language preference.
Source-domain interpretation and stable policy belong in Rust. Work that must happen synchronously
with DOM input or paint stays in TypeScript. The generated contract makes that boundary explicit
and testable.

This is more work than ADR-0001's frontend-only tactic, but it removes duplicated authority before
the second Species and second Harness make that authority harder to relocate.

## Consequences

**Easier**

- Rust tests prove Focus and Agent projections without a browser.
- The TypeScript renderer cannot import source-domain records to decide presentation.
- Timer, Pen, Activity, and Species policy have one authoritative implementation.
- Adding Codex changes the Harness adapter and Agent Session reducer, not rendering.
- Generated-type drift is caught by the existing repository gate.

**Harder**

- The render scene is a new Tauri read contract.
- Display-area changes must refresh Rust-owned region layout.
- Size is a small render curve rather than one static number.
- The application temporarily retains source-specific reads for detail and diagnostics alongside
  the renderer-specific scene read.

**Deferred**

- A Rust-owned fixed-step movement engine and direct Rust movement state.
- Dynamic Species registration, assignment, and persistence.
- Claude/Codex native event schemas and the neutral Harness adapter.

## Action Items

1. [ ] Define and generate the Rust-owned render-scene contract.
2. [ ] Add independent Rust Focus and Agent Session projectors with focused tests.
3. [ ] Add the read-only scene command and refresh it on source, settings, and DisplaySpace changes.
4. [ ] Cut the webview over to generated scene values and delete TypeScript source projections.
5. [ ] Move region layout and physical Pig policy to Rust; retain only frame mechanics and assets
       in TypeScript.
6. [ ] Revisit backend-owned movement only through a separately measured Animal Runtime design.
