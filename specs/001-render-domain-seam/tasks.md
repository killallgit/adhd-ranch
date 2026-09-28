---

description: "Dependency-ordered implementation tasks for the Rust-owned render domain seam"
---

# Tasks: Render Domain Seam

**Input**: Design documents from `/specs/001-render-domain-seam/`

**Prerequisites**: `plan.md`, `spec.md`, `research.md`, `data-model.md`, `contracts/animal-renderer.md`, `quickstart.md`

**Tests**: Required by the specification. Within each phase, add the listed tests first and confirm they fail for the intended missing behavior before implementing it.

**Organization**: Tasks are grouped by user story. The implementation strategy at the end packages them into no more than three independently reviewable vertical slices, as required by SC-006.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel because it changes different files and does not depend on another incomplete task in the same group
- **[Story]**: Maps the task to User Story 1, 2, or 3
- Every task names the exact file or files it changes

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Extend the existing Rust-to-TypeScript generation mechanism to the application command crate without adding a runtime dependency.

- [X] T001 Add optional `ts-rs` support and an `export-ts` feature matching the domain crate pattern in `crates/commands/Cargo.toml`
- [X] T002 [P] Extend `gen-types` and `gen-types:check` to run command-crate exports as well as domain exports in `Taskfile.yaml`

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Establish the generated renderer contract and shared size/profile vocabulary that every user story consumes.

**CRITICAL**: No source projection, Tauri scene command, or frontend cutover starts until this phase is green.

### Tests for the foundation

- [X] T003 Add failing serialization and contract tests in `crates/commands/tests/render_scene_contract.rs` for the exact camelCase generated shapes of `Species`, `Motion`, `RenderRect`, `AnimalSize`, `Animal`, `AnimalRegionLayout`, `SpeciesProfile`, `RenderSceneRequest`, and `RenderScene`; quote and prove these data-model constraints: Animal id is "Non-empty and unique across composed Focus and Agent Animals", label is "Display-only; carries no source identity", species is "Explicitly `pig` in this feature; never inferred from source identity", size "Resolves to a finite value greater than zero and contains no Timer vocabulary", `regionId` obeys "`null` means full DisplaySpace; non-null is resolved against current regions", and motion "Must be a supported Motion value"
- [X] T004 [P] Add failing Pig profile regression tests in `crates/commands/src/species.rs` for base size 48px, movement footprint 48px, walk speed 60px/s, friction 0.97, minimum-speed fraction 0.35, maximum toss multiplier 6, turn range 3000-8000ms, and frame interval 150ms
- [X] T005 [P] Add failing deterministic sampler tests in `src/lib/animalSize.test.ts` for fixed sizes, times before/during/after a linear curve, finite positive output, and the rule that zero-duration Timers are already fixed values before IPC

### Implementation for the foundation

- [X] T006 Define the Serde/`ts-rs` renderer DTOs and enums, validation constructors, and internal `AnimalRegion` descriptor in `crates/commands/src/render_scene.rs`; ensure `Animal` contains no Focus, Task, Timer, Agent Session, Harness, Hook Firing, Pen, Claude, Codex, selection, callback, or mutable movement record
- [X] T007 [P] Implement the sole Pig `SpeciesProfile` and exhaustive Species lookup in `crates/commands/src/species.rs`, with no dynamic registry, persistence, assignment setting, or second Species
- [X] T008 Export `render_scene` and `species` from `crates/commands/src/lib.rs`, keeping source-domain projectors out of the renderer DTO module
- [X] T009 Regenerate and commit `src/types/generated/Species.ts`, `src/types/generated/Motion.ts`, `src/types/generated/RenderRect.ts`, `src/types/generated/AnimalSize.ts`, `src/types/generated/Animal.ts`, `src/types/generated/AnimalRegionLayout.ts`, `src/types/generated/SpeciesProfile.ts`, `src/types/generated/RenderSceneRequest.ts`, and `src/types/generated/RenderScene.ts`
- [X] T010 Implement the frame-local fixed/linear absolute-pixel sampler in `src/lib/animalSize.ts` without importing or accepting Focus, Task, Timer, Agent Session, Pen, Harness, or transport types

**Checkpoint**: The Rust contract, generated TypeScript contract, Pig physical profile, and browser-only size sampler agree and pass their focused tests.

---

## Phase 3: User Story 1 - See Both Kinds of Animal Without Domain Coupling (Priority: P1) MVP

**Goal**: Render Focus Animals and Agent Session Animals together from independent Rust projections through one Rust-owned scene.

**Independent Test**: Populate one Focus and one enabled Agent Session, request a scene, and verify both appear and move with independent labels, size policies, region ids, motions, and explicit Pig Species; changing only one source leaves the other projected Animal unchanged.

### Tests for User Story 1

> Write these tests first and verify they fail for the intended missing behavior.

- [X] T011 [P] [US1] Add failing Focus projection tests in `crates/commands/src/focus_animals.rs` for raw Focus id, title label, explicit Pig, `regionId: null`, no-Timer fixed size, running/expired/zero-duration Focus Timer curves, expired-only resting Motion, and Task Timers having no effect
- [X] T012 [P] [US1] Add failing Agent projection tests in `crates/commands/src/agent_animals.rs` for `agent:` id namespacing, session name label, explicit Pig, fixed base size, Pen id assignment, Idle/Working Motion, distinct raw-id collisions, unique logical regions, and rejection of Focus, Timer, Claude payload, native-event, and transport inputs
- [X] T013 [P] [US1] Add failing scene-service tests in `crates/commands/src/render_scene.rs` for mixed and single-domain scenes, equal raw source ids, disabled-Agent filtering, independent source changes, unique non-empty ids, one Pig profile, and a serialized scene containing no source records
- [X] T014 [P] [US1] Add failing read-client tests in `src/api/tauriRenderSceneReader.test.ts` for `get_render_scene`, request-area serialization, and invalidation after `focuses-changed`, `agent-sessions-changed`, `settings-changed`, and `display-space`
- [X] T015 [P] [US1] Add failing full-display movement tests in `src/lib/animalMovement.test.ts` for generated Animal ids, walking/resting behavior, SpeciesProfile-supplied physics, freeze-by-id, drag/toss, and retained position when label, size, or Motion presentation values change
- [X] T016 [P] [US1] Add failing hook tests in `src/hooks/useAnimalMovement.test.ts` for mixed Focus/Agent rosters, one persistent `requestAnimationFrame` loop, sampled hit-rectangle size, Gather, drag state, and zero frame- or pointer-driven `get_render_scene` calls
- [X] T017 [P] [US1] Add failing mixed-render tests in `src/components/App.test.tsx` proving a scene with one Focus Animal and one Agent Animal renders both through explicit Species dispatch while the existing Focus reader remains available only for detail/edit data

### Implementation for User Story 1

- [X] T018 [P] [US1] Implement the pure Focus-to-`Animal` projector in `crates/commands/src/focus_animals.rs`, accepting only Focus/Timer-domain inputs and converting all Timer semantics to fixed or linear renderer size plus Motion before IPC
- [X] T019 [P] [US1] Implement the pure AgentSession-to-`Animal` and logical-region projector in `crates/commands/src/agent_animals.rs`, accepting only harness-neutral `AgentSession` records and emitting collision-safe `agent:` render ids
- [X] T020 [US1] Implement `RenderSceneService` composition in `crates/commands/src/render_scene.rs` so it reads current Focuses, enabled Agent Sessions, and settings through injected application-layer dependencies, calls the projectors independently, validates unique ids, attaches the Pig profile, and exposes no mutation or persistence behavior
- [X] T021 [US1] Add thin `RenderSceneState`/`get_render_scene` Tauri wiring in `src-tauri/src/ui_bridge/mod.rs` and construct/register it with `tauri::generate_handler!` in `src-tauri/src/app/mod.rs`
- [X] T022 [P] [US1] Implement the typed `get_render_scene` invoke and four invalidation subscriptions in `src/api/tauriRenderSceneReader.ts`, treating events only as refresh signals and never as scene payloads
- [X] T023 [US1] Implement display-area ownership, initial fallback geometry, and source/settings/display refresh coordination in `src/hooks/useRenderScene.ts` so scene reads occur only when scene inputs change
- [X] T024 [P] [US1] Port the source-neutral position, velocity, walking/resting, freeze, drag/toss, direction, and confinement math to `src/lib/animalMovement.ts`, obtaining every physical constant from generated `SpeciesProfile`
- [X] T025 [US1] Implement the frame-local roster/effect adapter in `src/hooks/useAnimalMovement.ts`, keyed by `Animal.id` and consuming only generated scene values plus `DisplaySpace`; keep existing Pig-named Tauri hit-rect and drag commands as compatibility contracts
- [X] T026 [P] [US1] Add exhaustive generated-Species dispatch in `src/components/AnimalSprite.tsx` and update `src/components/PigSprite.tsx` to accept absolute sampled size and Motion while retaining the Pig asset, sprite-sheet rows, bobbing, CSS, pointer capture, and DOM presentation
- [X] T027 [US1] Cut `src/components/App.tsx` and `src/main.tsx` over to `useRenderScene`, `useAnimalMovement`, generated Animals, and `AnimalSprite`; stop the ranch window from reading raw Agent Sessions or projecting source records while leaving `src/agent-debug.tsx` on its existing diagnostic reader
- [X] T028 [US1] Remove the replaced TypeScript contract and source projections in `src/types/animal.ts`, `src/hooks/useAnimals.ts`, `src/lib/animals.ts`, `src/lib/animals.test.ts`, `src/lib/focus/animals.ts`, `src/lib/focus/animals.test.ts`, `src/lib/session/animals.ts`, and `src/lib/session/animals.test.ts`

**Checkpoint**: The Rust service can independently produce either domain and the running ranch consumes one generated scene; no TypeScript production module projects Focus or Agent Session records into Animals.

---

## Phase 4: User Story 2 - Interact Only With User-Owned Work (Priority: P2)

**Goal**: Keep selection and editing entirely in the Focus domain while renderer interaction remains id-only.

**Independent Test**: Click a current Focus Animal and an Agent Animal separately, remove and recreate the selected Focus, and verify only the current Focus opens details while stale selection, freeze state, and expanded hit areas are cleared.

### Tests for User Story 2

> Write these tests first and verify they fail for the intended missing behavior.

- [X] T029 [P] [US2] Add failing state-transition tests in `src/hooks/useFocusSelection.test.tsx` for click/tray requests, Agent ids resolving to none, close, current Focus updates, removal clearing the request, and recreation of the same id not silently reopening details
- [X] T030 [P] [US2] Add failing interaction tests in `src/components/App.test.tsx` for Focus-only detail/edit actions, Agent clicks doing nothing, selected Focus freeze-by-render-id, stale selection cleanup, and narrow hit rectangles returning after removal or close

### Implementation for User Story 2

- [X] T031 [US2] Implement `requestedId` resolution exclusively against the current Focus collection in `src/hooks/useFocusSelection.ts`, forgetting any request that no longer resolves and exposing only `Focus | null`, request, and close operations
- [X] T032 [US2] Wire `useFocusSelection` into `src/components/App.tsx` so renderer clicks and the existing tray open-detail event pass only render ids, Agent ids never create/edit a Focus, and the selected scene Animal is used only for freeze and detail placement
- [X] T033 [US2] Remove the obsolete cross-domain selection implementation and tests in `src/hooks/useAnimalSelection.ts` and `src/hooks/useAnimalSelection.test.tsx`

**Checkpoint**: Focus detail/edit behavior remains intact, Agent Animals are observation-only, and stale Focus selection cannot survive source removal.

---

## Phase 5: User Story 3 - Preserve Each Domain's Spatial Rules (Priority: P3)

**Goal**: Give Focus Animals the full DisplaySpace and confine Agent Animals to Rust-laid-out regions without exposing Pen or source semantics to the renderer.

**Independent Test**: Move an ungrouped Focus and a grouped Agent Session together; verify the Focus uses DisplaySpace, the Agent remains in its assigned region, unresolved regions fall back safely, Focus Timer changes affect only the Focus, and Agent Activity changes affect only the Agent.

### Tests for User Story 3

> Write these tests first and verify they fail for the intended missing behavior.

- [X] T034 [P] [US3] Add failing pure Rust layout tests in `crates/commands/src/region_layout.rs` for unique/sorted regions, stable hue, centered capped cells, empty input, display resize, and the data-model constraints that region id is "Non-empty and unique in the region roster" and label is "Display-only; contains no Pen object"
- [X] T035 [P] [US3] Extend `src/lib/animalMovement.test.ts` with failing tests for `regionId: null` using DisplaySpace, known ids using scene rectangles, unresolved/disappearing ids falling back to DisplaySpace, reconfinement after region changes, and unchanged Pig speed/turn/friction/toss behavior
- [X] T036 [P] [US3] Extend `src/hooks/useAnimalMovement.test.ts` with failing tests proving size sampling does not respawn Animals or restart the loop, delayed scene/layout availability is safe, region changes preserve movement identity, and ordinary animation/dragging emits zero backend frame traffic
- [X] T037 [P] [US3] Add failing spatial integration tests in `src/components/App.test.tsx` proving Focus Timer growth/expiry changes only the Focus Animal, Agent Activity changes only the Agent Animal, and RegionBox renders scene values without Pen data

### Implementation for User Story 3

- [X] T038 [US3] Implement pure descriptor deduplication, id ordering, stable hue, centered grid layout, maximum-size capping, and empty-input behavior in `crates/commands/src/region_layout.rs`
- [X] T039 [US3] Integrate Agent logical regions, current maximum-region-size settings, and `RenderSceneRequest.area` into `RenderSceneService` in `crates/commands/src/render_scene.rs` so returned `AnimalRegionLayout` values contain no Pen records
- [X] T040 [P] [US3] Implement renderer-only region lookup and DisplaySpace fallback in `src/lib/regions.ts` without layout policy or Focus, Timer, Agent Session, Harness, Pen, or transport imports
- [X] T041 [US3] Complete region/profile consumption in `src/lib/animalMovement.ts` and `src/hooks/useAnimalMovement.ts`, including spawn, walking, resting, Gather, freeze, hit rectangles, dragging, and toss while keeping sampled presentation size separate from the fixed movement footprint
- [X] T042 [P] [US3] Replace Pen-specific region drawing with scene-only `AnimalRegionLayout` rendering in `src/components/RegionBox.tsx` and update region styles in `src/styles.css`
- [X] T043 [US3] Update `src/components/App.tsx` to draw `RegionBox` values directly from `RenderScene.regions` and pass only generated Animals, layouts, profiles, DisplaySpace, and `frozenId` into the frame runtime
- [X] T044 [US3] Remove superseded browser-owned policy files `src/hooks/usePigMovement.ts`, `src/hooks/usePigMovement.test.ts`, `src/lib/ranchAnimalMovement.ts`, `src/lib/ranchAnimalMovement.test.ts`, `src/lib/session/pens.ts`, `src/lib/session/pens.test.ts`, and `src/components/PenBox.tsx`

**Checkpoint**: Rust owns projection, scene composition, region layout, and Pig physical policy; TypeScript owns only size sampling, transient frame state, pointer interaction, and presentation.

---

## Phase 6: Polish & Cross-Cutting Verification

**Purpose**: Prove the final boundary, preserve documentation accuracy, and run the repository/platform gates without expanding scope.

- [X] T045 [P] Update the current implementation map and renderer boundary description in `docs/architecture.md`, citing accepted `docs/adr/0001-two-apps-over-one-renderer.md` and `docs/adr/0003-rust-owned-render-scene.md` without modifying their accepted decisions
- [X] T046 Run the prohibited-dependency, obsolete-symbol, and Pig-policy searches from `specs/001-render-domain-seam/quickstart.md`; resolve every production-code match in `crates/commands/src/focus_animals.rs`, `crates/commands/src/agent_animals.rs`, `src/lib/animalSize.ts`, `src/lib/animalMovement.ts`, `src/lib/regions.ts`, and `src/hooks/useAnimalMovement.ts`
- [X] T047 Run `cargo test -p adhd-ranch-commands`, focused Vitest commands, `task gen-types:check`, and the full `task check` gate from `Taskfile.yaml`; verify `git diff --check` and a clean generated-types diff
- [X] T048 Perform the running-app smoke test in `specs/001-render-domain-seam/quickstart.md`, first checking port 1420, and record any platform-only failures in `specs/001-render-domain-seam/tasks.md`

**T048 smoke record (2026-09-27, macOS)**: Port 1420 was clear before launch. `task dev`
built and started both Vite on port 1420 and the debug Tauri binary without runtime startup errors;
the test processes were then stopped and port 1420 was verified clear again. The automated Rust,
hook, and App integration tests cover mixed-domain rendering, Focus-only interaction, independent
Motion/size changes, and region rendering. At the time of this initial smoke, a live mixed
Claude-session visual check still required an active hook session; T050 records the subsequently
completed live mixed-session verification. Windows CI is deferred under the explicit platform
scope in `spec.md`.

---

## Dependencies & Execution Order

### Phase dependencies

- **Phase 1 (Setup)**: Starts immediately.
- **Phase 2 (Foundation)**: Depends on Phase 1 and blocks all user-story implementation.
- **Phase 3 (US1)**: Depends on Phase 2 and establishes the shared scene plus renderer cutover.
- **Phase 4 (US2)**: Depends on the generated scene and App cutover from US1; it changes only Focus-owned interaction semantics.
- **Phase 5 (US3)**: Depends on the scene/runtime interfaces from US1, but its Rust layout work can begin after Phase 2 while US1 frontend work proceeds.
- **Phase 6 (Polish)**: Depends on all selected user stories.
- **Phase 7 (Convergence)**: Depends on Phase 6 and closes remaining acceptance or boundary gaps before synchronization with current `main` and the final gate.

### User-story dependency graph

```text
Setup -> Foundation -> US1 mixed scene/rendering -> US2 Focus-only interaction
                         |
                         +-----------------------> US3 spatial policy/runtime
US2 + US3 -> final boundary checks and task check
```

- **US1 (P1)** is the MVP and owns the common scene/rendering path.
- **US2 (P2)** requires US1's id-only renderer interaction but does not affect Agent projection, layout, or movement.
- **US3 (P3)** requires US1's scene/runtime inputs but is otherwise independent of US2 selection state; it compares `frozenId` only by equality.

### Within each user story

- Add the listed tests and confirm the intended red state before implementation.
- Complete pure Rust/TypeScript policy before I/O adapters.
- Complete application services before Tauri commands and React integration.
- Remove legacy files only after their replacements and focused tests are green.
- Run the focused tests at the phase checkpoint before moving to the next merge slice.

## Parallel Opportunities

### User Story 1

```text
Parallel test group: T011, T012, T013, T014, T015, T016, T017
Parallel pure implementation group after tests: T018, T019, T024, T026
Then: T020 -> T021; T022 -> T023; T020 + T023 + T025 + T026 -> T027 -> T028
```

### User Story 2

```text
Parallel test group: T029, T030
Then: T031 -> T032 -> T033
```

### User Story 3

```text
Parallel test group: T034, T035, T036, T037
Parallel implementation group after Rust layout contract settles: T038, T040, T042
Then: T038 -> T039; T039 + T040 -> T041; T041 + T042 -> T043 -> T044
```

## Implementation Strategy

### MVP first

1. Complete Setup and Foundation.
2. Complete US1 through T028.
3. Stop and validate the mixed Focus/Agent scene independently.
4. Do not claim the full ADR complete until US2, US3, Phase 6, and Phase 7 pass.

### Three reviewable vertical slices

1. **Rust contract and independent projectors**: T001-T020. Completion promise: Rust independently turns either source domain into generated renderer-only values and proves all source-semantic decisions.
2. **Scene IPC, App cutover, and Focus-only selection**: T021-T033. Completion promise: the ranch renders the Rust scene, only current Focuses open details, and no TypeScript production module projects source records.
3. **Rust regions, Species-driven thin frame runtime, and convergence**: T034-T051. Completion promise: Rust owns region/physical policy, the browser owns only frame mechanics/presentation, legacy policy files are gone, the live mixed scene is verified, and the synchronized full gate passes.

**Approved delivery exception (2026-09-28)**: Owner approved delivering these three review slices
atomically in PR #88 because the generated contract, application cutover, and removal of the old
renderer authority form one replacement. Review PR #88 in the three ranges above. This exception
is limited to this feature and does not relax the normal small-PR rule.

Normally, each slice must synchronize with current `main`, restate its completion promise in its
PR, cite the source Spec Kit tasks, pass its focused tests and `task check`, and stay narrow enough
for a roughly fifteen-minute human review. Under the approved PR #88 exception, the three ranges
above remain separate review guides with focused phase checkpoints, while the assembled replacement
uses one post-convergence synchronization, full `task check`, and CI gate.

## Notes

- Existing Focus/Timer mutations, persisted files, Agent hook frames, socket transport, plugin lifecycle, settings schema, Pig hit-test/drag command names, debug reads, and integration diagnostics are protected and remain unchanged.
- Complete Claude hook schema modeling, the harness-neutral ingestion layer, Codex support, a second Species, Species assignment/persistence, and backend fixed-step movement are separate future milestones.
- A task is not complete when only focused tests pass; its slice must pass the repository gate and generated-type drift check.

---

## Phase 7: Convergence

- [X] T049 Remove frame-driven Tauri command traffic from `src/hooks/useAnimalMovement.ts` while preserving moving-animal hit-testing, dragging, and existing visible behavior; add regression coverage that observes every backend call and proves normal animation and dragging issue no per-frame invokes or events per FR-021 and SC-011
- [X] T050 Run the complete mixed Focus/Agent visual checklist in `specs/001-render-domain-seam/quickstart.md` with an active hook session and record the observed results in `specs/001-render-domain-seam/tasks.md` per SC-001 and T048
- [X] T051 Synchronize the feature branch with the latest `origin/main`, record the feature-head and main SHAs, then rerun `task check`, generated-type drift verification, and CI before merge

**T049 regression record (2026-09-27)**: The animation loop now advances only local movement
and presentation state. Moving-animal hit rectangles synchronize on an independent 64 ms cadence,
with immediate wide/narrow updates retained at drag boundaries. The hook regression observes both
backend adapters and proves ordinary and dragging animation frames add no Tauri calls; it also
proves explicit drag-active calls, wide drag/selection hit regions, and the narrow animal-only region
restored after selection closes.

**T050 mixed-scene visual record (2026-09-27, macOS)**: Ran `task dev` against an isolated
`ADHD_RANCH_HOME` containing one Focus and delivered an active Agent session through the real Unix
hook socket. Both Pig presentations rendered and moved without visible presentation changes. The
Focus ranged over the full enabled display while the Agent remained inside the centered, 320 px
capped `ADHD-RANCH` region. Expiring the Focus timer enlarged and rested only the Focus; changing the
Agent from working to idle rested only the Agent. Selecting the Focus opened its detail card and held
its accessibility position at `(-1010, 643)` for two seconds; closing the card removed the detail,
restored the narrow hit region, and movement resumed from `(-927, 711)` to `(-942, 750)`. Dispatching
the same pointer-up interaction to the Agent opened no Focus detail. Finally, enabled displays were
changed from the secondary display to both displays (`3000x1920`) and then to the primary display
(`1920x1080`); each backend `DisplaySpace` refresh kept the Agent region centered/capped and the Focus
free in the full enabled space. The app was stopped cleanly and port 1420 was clear afterward.

**T051 synchronization record (2026-09-28)**: Fetched `origin/main` at
`c188eea21593c229b28e952a7519e9f6984ca790` and verified feature head
`7649de1bbbe62d5f2cfc1ab8788e5d8343a6e5d6` was six commits ahead and zero behind. The synchronized
tree passed `task check`, including generated-type drift verification, and GitHub Actions CI run
[`36437871180`](https://github.com/killallgit/adhd-ranch/actions/runs/36437871180) completed successfully.
