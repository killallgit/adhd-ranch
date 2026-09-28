# Feature Specification: Render Domain Seam

**Feature Branch**: `001-render-domain-seam`

**Created**: 2026-09-26

**Status**: Approved for planning

**Input**: User description: "Plan implementation of accepted ADR-0001 so Focuses and Agent
Sessions are independent applications that share only the renderer. Preserve an explicit seam for
additional Species and richer Animal behavior, and keep Claude/Codex session ingestion as its own
future domain rather than mixing it with Focus and Timer work. Keep source interpretation,
projection, layout policy, and physical Species policy in Rust; leave only frame-local animation,
pointer interaction, and drawing in the webview."

## User Scenarios & Testing *(mandatory)*

### User Story 1 - See Both Kinds of Animal Without Domain Coupling (Priority: P1)

As a Ranch user, I can see my Focus animals and observed Agent Session animals together on the
same ranch while each kind retains its own rules and lifecycle.

**Why this priority**: This is the core promise of ADR-0001: preserve the one genuinely shared
experience—the rendered ranch—without treating user work and observed agent activity as one
domain.

**Independent Test**: Populate the ranch with one Focus and one Agent Session, then verify that
both appear and move while their labels, size, region, and motion are derived from their own
domain state and both select the current Pig Species explicitly.

**Acceptance Scenarios**:

1. **Given** a Focus and an Agent Session are both active, **When** the ranch is displayed,
   **Then** both appear as animals in the shared renderer.
2. **Given** only Focus state changes, **When** the ranch refreshes, **Then** the Focus animal
   changes as required and the Agent Session animal retains its prior visible state.
3. **Given** only Agent Session state changes, **When** the ranch refreshes, **Then** the Agent
   Session animal changes as required and the Focus animal retains its prior visible state.
4. **Given** either domain produces an Animal, **When** the shared renderer presents it, **Then**
   the renderer selects Pig behavior from explicit Species rather than source-domain identity.

---

### User Story 2 - Interact Only With User-Owned Work (Priority: P2)

As a Ranch user, I can select and edit Focus animals, while Agent Session animals remain
read-only observations and never open or impersonate Focus details.

**Why this priority**: Focus interaction changes user-owned data. Keeping selection entirely in
the Focus domain prevents observed agent activity from gaining accidental edit behavior.

**Independent Test**: Click a Focus animal and an Agent Session animal separately, then verify
that only the Focus opens details and that removing the selected Focus clears the selection.

**Acceptance Scenarios**:

1. **Given** a visible Focus animal, **When** the user selects it, **Then** the matching Focus
   detail is shown and can be edited through existing Focus actions.
2. **Given** a visible Agent Session animal, **When** the user clicks it, **Then** no Focus detail
   is selected or opened.
3. **Given** a selected Focus is removed, **When** the ranch refreshes, **Then** the stale
   selection and its expanded interaction area are cleared.

---

### User Story 3 - Preserve Each Domain's Spatial Rules (Priority: P3)

As a Ranch user, I see Focus animals use the whole display space and Agent Session animals use
their assigned regions, without the renderer needing to understand timers, sessions, or pens.

**Why this priority**: Spatial behavior is visibly shared but its policy belongs to each source
domain. Keeping that distinction makes later Agent motion changes safe for Focus behavior.

**Independent Test**: Display an ungrouped Focus and a grouped Agent Session, then verify that
the Focus can use the ranch while the session stays in its assigned region and both obey the
same movement and hit-testing behavior.

**Acceptance Scenarios**:

1. **Given** a Focus animal and an Agent Session assigned to a region, **When** both move,
   **Then** the Focus uses the whole display space and the Agent Session remains in its region.
2. **Given** a Focus timer changes its growth or expiry state, **When** the animal is redrawn,
   **Then** only that Focus animal's size or motion changes.
3. **Given** an Agent Session changes between working and idle, **When** the animal is redrawn,
   **Then** only that Agent Session animal's motion changes.

### Edge Cases

- A Focus identifier and raw Agent Session identifier have the same text.
- A selected Focus disappears between input handling and the next render.
- A session references a region that is temporarily unavailable or not yet laid out.
- Settings or display information have not loaded when animals first appear.
- One domain has no animals while the other domain remains populated.
- Focus timer state refreshes frequently while the Agent Session roster remains unchanged.
- A time-varying size is sampled before its start, after its end, or with a zero duration.
- The DisplaySpace changes while the current Agent region layout is visible.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The system MUST derive the visible representation of Focus animals independently
  from the visible representation of Agent Session animals.
- **FR-002**: Both domains MUST meet through one flat render contract containing only a unique
  render identifier, label, Species, renderer-only absolute-pixel size policy, optional
  movement-region identifier, and motion.
- **FR-003**: The shared renderer and movement behavior MUST NOT require Focus, Task, Timer,
  Harness, Agent Session, Hook Firing, or Pen data to draw, move, hit-test, or freeze an animal.
- **FR-004**: A Focus animal's size and motion MUST be derived only from Focus and Timer state,
  and its movement region MUST be the whole available display space. Rust MUST convert Timer
  state into renderer vocabulary before the value crosses IPC.
- **FR-005**: An Agent Session animal's size, motion, and optional movement region MUST be
  derived only from Agent Session state and MUST NOT depend on a Focus clock or timer.
- **FR-006**: Render identifiers MUST remain unique when Focus and Agent Session source
  identifiers contain the same text.
- **FR-007**: Focus selection MUST resolve exclusively against the current Focus collection and
  MUST clear when the requested Focus no longer exists.
- **FR-008**: Clicking an Agent Session animal MUST NOT select, create, or edit a Focus.
- **FR-009**: The movement surface MUST accept a render identifier to freeze and MUST report
  animal clicks by render identifier without interpreting source-domain identity.
- **FR-010**: Composing the two sets of visible animals MUST NOT expose one domain's source data
  to the other domain.
- **FR-011**: Existing visible behavior for animal movement, dragging, region layout,
  hit-testing, Focus details, timer growth, and session activity MUST remain unchanged except
  where this specification explicitly separates domain ownership.
- **FR-012**: This feature MAY add one read-only render-scene command and its generated data
  contracts, but MUST NOT change persistence, integration transport, existing mutation commands,
  or hook/plugin protocols.
- **FR-013**: Shared movement concepts MUST use the settled Animal vocabulary; Pig terminology
  MUST remain only where behavior or assets are specific to the currently implemented species.
- **FR-014**: Species MUST be explicit renderer vocabulary. Both current projections MUST emit
  Pig, the only Species supported by this feature, rather than making shared code infer Species
  from a source domain, identifier, label, region, size, or motion.
- **FR-015**: Pig-specific physical policy—including footprint, speed, friction, turn timing, and
  animation cadence—MUST be defined by the Rust-owned Pig profile. Pig assets, CSS, and
  sprite-sheet presentation MUST remain behind a Pig-owned webview boundary. Generic Animal
  movement MUST own only source-independent frame behavior such as position, region confinement,
  freezing, and dragging.
- **FR-016**: The Agent Session projection MUST consume only the harness-neutral Agent Session
  domain record and MUST NOT import Claude hook payloads, event names, transport frames, or other
  vendor-specific integration types.
- **FR-017**: Rust MUST be the source of truth for the renderer contract, independent Focus and
  Agent Session projections, render-id namespacing, Motion choice, Species choice, size policy,
  logical region derivation, and scene composition.
- **FR-018**: The webview MUST consume generated Rust bindings for the render scene and MUST NOT
  maintain duplicate hand-written source projections or renderer-contract declarations.
- **FR-019**: Rust MUST deduplicate, order, style, and lay out Agent regions from current
  Agent Sessions, settings, and requested display geometry before returning the render scene.
- **FR-020**: The webview MAY sample a Rust-provided size curve and run transient movement at frame
  cadence, but MUST NOT receive Timer semantics or rederive Focus, Agent Session, Pen, Harness, or
  Species policy.
- **FR-021**: Frame animation, DOM presentation, pointer capture, drag/toss input, and current
  browser-local position/velocity MUST NOT require per-frame Tauri invocation or event traffic.

### Key Entities

- **Render Animal**: The renderer-only description of one visible animal: a unique identifier,
  label, Species, renderer-only size policy, optional movement-region identifier, and motion.
- **Animal Size**: A Rust-produced renderer value describing either a fixed absolute pixel size or
  a clamped time-varying absolute-pixel curve. It contains no Timer vocabulary.
- **Species**: Renderer vocabulary that selects an Animal's concrete appearance and behavior
  profile. Pig is the sole supported value in this feature; it is not a Focus/Agent source-domain
  discriminator.
- **Species Profile**: Rust-owned physical movement and cadence policy used by generic Animal
  runtime behavior. The current Pig profile preserves existing behavior; concrete asset and CSS
  presentation remain in the webview.
- **Motion**: A renderer-level value describing how an animal moves now. For this slice, the
  supported values remain walking and resting.
- **Focus Projection**: The read-only mapping from current Focus and Timer state to Render
  Animals. It owns Focus growth, expiry motion, and whole-display placement.
- **Agent Session Projection**: The read-only mapping from observed Agent Session state to
  Render Animals. It owns session identity, activity motion, and assigned-region placement.
- **Focus Selection**: The currently requested Focus identifier resolved against the current
  Focus collection; it never contains an Agent Session.
- **Render Scene**: The Rust-produced read model containing projected Animals, laid-out renderer
  regions, and the Species profiles needed by the frame runtime.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: All three primary scenarios—mixed display, Focus-only interaction, and independent
  spatial behavior—pass with no visible regressions in existing ranch behavior.
- **SC-002**: Focus projection can be validated with zero Agent Session inputs, and Agent Session
  projection can be validated with zero Focus or Timer inputs.
- **SC-003**: One hundred percent of values consumed by shared drawing and movement behavior are
  generated fields of the Rust-owned renderer contract rather than source-domain records.
- **SC-004**: One hundred percent of selection outcomes resolve from the current Focus collection;
  clicking any Agent Session produces no selected Focus.
- **SC-005**: The repository's full required quality gate passes after the change, including all
  existing behavior checks for movement, dragging, hit-testing, Focus details, timers, and Agent
  Sessions.
- **SC-006**: The feature is deliverable as no more than three independently reviewable vertical
  slices, each with one observable completion promise.
- **SC-007**: Every shared movement type and operation uses Animal vocabulary, while all retained
  Pig names refer to species-specific rendering or compatibility contracts.
- **SC-008**: One hundred percent of current Focus and Agent Session projections emit the explicit
  Pig Species, and shared code never derives Species from source-domain state.
- **SC-009**: Generic TypeScript Animal movement and region modules contain zero Pig-specific
  speed, friction, turn-cadence, animation-cadence, or footprint definitions; physical values come
  from the Rust Pig profile, while Pig assets and direction/frame presentation remain in
  Pig-specific webview modules.
- **SC-010**: Rust tests cover one hundred percent of source-semantic projection decisions,
  including id collisions, Timer-to-size/Motion mapping, Activity-to-Motion mapping, Species,
  region assignment/layout, and disabled-Agent behavior; the old TypeScript projection modules no
  longer exist.
- **SC-011**: Normal animation and dragging perform zero per-frame Tauri invokes or backend frame
  events; IPC refresh occurs only when source state, settings, or DisplaySpace inputs change.
- **SC-012**: Generated TypeScript render-scene bindings are produced from Rust and pass the
  repository's generated-type drift check with no hand-written duplicate contract.

## Assumptions

- The accepted decision in `docs/adr/0001-two-apps-over-one-renderer.md` is authoritative and is
  revalidated by the owner's 2026-09-26 direction to plan its implementation and refined by the
  2026-09-27 direction to preserve Species and Harness seams up front.
- `docs/adr/0003-rust-owned-render-scene.md` is authoritative for implementation placement: Rust
  owns the render read model and policies; TypeScript owns frame-local browser mechanics.
- Current `main` is the behavioral baseline. It still contains a source-domain `Animal` union,
  shared domain discriminators, and selection that consumes both domains; those are verified
  observations rather than inherited requirements.
- Existing issues 051, 057, and 058 are historical planning evidence only. Their text is not
  authoritative; this specification independently restates the requirements adopted here.
- ADR-0001's render-layer boundary remains in scope, but its frontend-only implementation tactic
  is replaced by ADR-0003. Option C build-enforced crate/folder isolation and a backend-owned
  fixed-step movement engine remain deferred until this contract has shipped and been measured.
- A second Species, Species selection, persistence, registries, and new animation behavior are out
  of scope. This feature adds only the explicit Species value and boundary needed to avoid baking
  Pig assumptions into supposedly generic Animal code.
- Complete Claude hook-schema modeling, a harness-neutral ingestion interface, Codex support, Rust
  schema-crate extraction, storage changes, and transport changes are out of scope. They form a separate
  Agent Activity roadmap after this cleanup pass; the current feature only protects that boundary.
- Focus, Task, and Timer product work remains a separate Work Management roadmap. It may project
  Animals through the same renderer but does not depend on Claude, Codex, hooks, or session
  ingestion.
- Existing desktop command names may remain as compatibility contracts even when the internal
  shared movement vocabulary changes.
- The temporary constitution draft in `.specify/memory/constitution.md` is treated as the current
  governing direction for planning; it must complete its own review before being committed.

## Source Provenance

- **Accepted decision**: `docs/adr/0001-two-apps-over-one-renderer.md` — Option B moves the seam to
  a renderer-only contract and keeps selection in the Focus domain.
- **Accepted implementation placement**: `docs/adr/0003-rust-owned-render-scene.md` — Rust owns the
  generated scene, projections, region layout, and physical Species policy; TypeScript owns
  frame-local renderer mechanics.
- **Verified current observation**: Current `main` still carries Focus and Agent Session source
  records through a discriminated animal union and shared consumers.
- **Present owner direction**: Plan ADR-0001 now, preserve the future Species and animated-element
  seam explicitly, and treat complete Claude/Codex session ingestion as a distinct future domain
  from Focus/Timer work, with Rust performing as much non-frame-local work as practical.
- **New proposal in this spec**: Deliver the accepted boundary as at most three vertical slices,
  including an explicit Pig Species and Pig-owned behavior boundary; exact technical sequencing is
  left to `plan.md` and `tasks.md`.
- **Rejected or deferred context**: ADR Option A remains rejected; Option C and separate processes
  remain outside this feature.
