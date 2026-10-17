# Tasks: Ranch Today Baseline Validation

**Input**: Design documents in `specs/003-ranch-today/`: [plan](plan.md), [spec](spec.md), [research](research.md), [data model](data-model.md), [contracts](contracts/current-behavior.md), [quickstart](quickstart.md).

**Status**: Draft execution plan, generated 2026-10-17. The specification remains Draft. Generating this list does not approve product changes or certify current behavior.

**Scope**: Validate and document existing behavior. No product implementation, dependency changes, new tests, migrations, generated-type edits, or fixes for FR-013–FR-015. Existing tests are reused as evidence. New runtime work needs its own approved scope.

**Organization**: Setup → foundation → five independently verifiable user stories → cross-cutting reconciliation. Every task starts unchecked. Mark an execution task done when its stated work and honest result record are complete; a recorded failure or unverified check is not a passed acceptance criterion.

**Execution record (2026-10-17)**: The current-main automated gate passed. Native/platform observations remain unverified as detailed in [the pass record](validation/pass-record.md). Checked tasks record completed evidence assessment, not passed product acceptance; all six success criteria still need end-to-end proof.

## Format and Paths

Tasks use `- [ ] Tnnn [P?] [USn?] description`. `[P]` marks independent source checks with distinct output files after the foundation completes. Other tasks run sequentially.

All file paths below are relative to the repository root. Execution creates one shared `specs/003-ranch-today/validation/pass-record.md` and one evidence note per story (`us1.md` through `us5.md` in the same directory). These outputs are not created by task generation.

Evidence notes record source pointers, commit/platform, exact commands and exit statuses, observed outcomes, pass/fail/unverified status, and discrepancies. Never substitute source inspection or past CI for a fresh test or native result. Do not include personal data, credentials, raw transcripts, or machine-specific scratch paths in committed evidence.

## Constitution Compliance Check

| Principles | Execution constraint |
|---|---|
| I — Local state | Use synthetic isolated data; inspect atomic-write/sidecar coverage; no personal work/configuration writes or outbound product feature. |
| II — Domain separation | Validate Focus/Task/Timer independently from Agent activity; only the flat render contract joins them. |
| III — Typed boundaries | Reuse existing source owners and tests; generated contracts remain generated. No new implementation is authorized. |
| IV — Explicit integrations | Toggle is not install. Distinguish validated configuration from observed delivery; live checks use isolated configuration and explicit installation. |
| V — Observable completion | Bound tasks by evidence outputs; synchronize with current main before the gate; retain failures and unverified outcomes. Shipping completion still requires review, green CI, and merge. |
| VI — Authority | Keep Draft status; accepted ADRs outrank observed shortcomings. Record ADR-0002 drift and route follow-ons without silently changing accepted decisions. |

**Result**: PASS for this documentation/validation task design; no exception is requested. Existing runtime drift is not declared compliant. These tasks may collect baseline evidence without promoting a Draft spec to governing implementation authority.

## Phase 1: Setup

**Purpose**: Establish the scope and disposable validation environment; reuse the existing project.

- [X] T001 Create `specs/003-ranch-today/validation/pass-record.md` with the input artifact versions, current commit/branch, relation to current main, scope exclusions, and Constitution I–VI check; preserve all existing dirty work and arrange a synchronized isolated checkout if necessary for the later gate rather than resetting or stashing it.
- [X] T002 Follow the isolation setup in `specs/003-ranch-today/quickstart.md` and record the procedure in `specs/003-ranch-today/validation/pass-record.md`: use a fresh short temporary root with separate `ADHD_RANCH_HOME` and `CLAUDE_CONFIG_DIR`, synthetic data only, and one running Ranch instance; check first-launch example creation and that deleting it/restarting does not recreate it.

## Phase 2: Foundational

**Purpose**: Establish shared authority, evidence conventions, and automated results before any story validation.

- [X] T003 Reconcile `specs/003-ranch-today/contracts/current-behavior.md` and `specs/003-ranch-today/research.md` against the selected validation commit and accepted ADRs 0001–0003; record any staleness in `specs/003-ranch-today/validation/pass-record.md`, retaining the distinction between current observations, accepted decisions, Draft follow-ons, and unresolved evidence.
- [X] T004 After confirming synchronization with current main, run `rtk task check` from the repository root using `Taskfile.yaml`; record command, commit, exit status, and any failures in `specs/003-ranch-today/validation/pass-record.md`. Do not weaken gates or edit generated types to get green; report failures for separately scoped resolution. This single gate supplies the existing automated results referenced by story notes; rerun only after relevant changes or unresolved failures justify it.

**Checkpoint**: Evidence setup exists and the automated gate has an honest result. Failures must remain visible; no story may claim coverage from a failed or unexecuted test.

## Phase 3: US1 — Glance and Know What Is Open (P1, MVP)

**Goal**: Establish that Focus Animals are useful, ambient reminders on enabled displays.

**Independent test**: With two synthetic Focuses, identify both by their visible Pig labels, verify empty-space click-through, drag/toss/Gather, and observe no unsolicited attention.

- [X] T005 [P] [US1] Review `crates/commands/src/focus_animals.rs`, `crates/commands/src/render_scene.rs`, `crates/commands/tests/render_scene_contract.rs`, and `src/lib/animalMovement.test.ts`; write `specs/003-ranch-today/validation/us1.md` with source/coverage pointers for the model constraints “Species is Pig; Motion is Walking or Resting.”, “Size is fixed or linearly sampled over time.”, optional `regionId`, and “Movement and sprites consume neither domain types nor source discriminators.”; distinguish generated wire shape from native behavior.
- [X] T006 [US1] Execute V3 in `specs/003-ranch-today/quickstart.md` using two fresh synthetic Focuses; record labels/one Pig per Focus, drag/toss, Gather, primary-only defaults, display enablement/no gap traversal, empty-space click-through, and ambient attention observations in `specs/003-ranch-today/validation/us1.md`. Record unavailable multi-display/native checks as unverified rather than inferred from tests (SC-001).

**Checkpoint**: US1 has source evidence plus explicit native results or remaining gaps; this is the suggested MVP validation scope.

## Phase 4: US2 — Work a Focus from Its Animal (P1)

**Goal**: Verify card editing and Timer ownership without changing their implementation.

**Independent test**: Open one Focus card, add/edit/Complete/Delete Tasks, start/Clear Timers, compare files, and independently exercise Focus versus Task expiry.

- [X] T007 [P] [US2] Review `crates/domain/src/focus.rs`, `crates/domain/src/parse.rs`, `crates/commands/src/focus.rs`, `crates/commands/src/timers.rs`, and `src/components/AnimalDetail.test.tsx`; record source/coverage in `specs/003-ranch-today/validation/us2.md` for “App writes reject blank text.”, “parser requires frontmatter with `id` and `title`.”, optional `timer`, `status: Running / Expired`, and one-owner `Focus { focus_id }` / `Task { focus_id, index }` addressing. Treat positional identity as an observation, not a constraint to preserve.
- [X] T008 [US2] Execute V1 card/CRUD actions from `specs/003-ranch-today/quickstart.md`; record title rename, duplicate and delete, Task add/edit/Complete/undo/Delete, blank-input rejection, matching Markdown, and outside-click/Escape closure in `specs/003-ranch-today/validation/us2.md`. Verify “Editing text preserves completion.” and distinguish Complete/Delete from Timer Clear (SC-002).
- [X] T009 [US2] Execute V1 start/restart/Clear and saved-chip checks; inspect `crates/domain/src/settings.rs` and `src-tauri/src/app/settings_workflow.rs` against “Saved custom minutes are positive, unique, and exclude built-ins 10/30/60.” Record shared pickers, restart persistence, owning sidecars, and chip removal leaving running Timers/built-ins intact in `specs/003-ranch-today/validation/us2.md`.
- [X] T010 [US2] Execute V1 short-Timer expiry/revival checks and compare fixture work files before/after expiry; record Focus growth/resting/ghostly/away/tray behavior, allowed notifications, and timer-only persistence in `specs/003-ranch-today/validation/us2.md`. Verify “Adding a Task clears an Expired Focus Timer (Revive), never a Task Timer.” and “Task Timer expiry changes neither projection nor tray membership.”

**Checkpoint**: Editing, Timer selection, and both expiry paths have independent results; no new model/service/UI work is introduced.

## Phase 5: US3 — Run the Ranch from the Tray (P2)

**Goal**: Verify tray actions, live Settings, and soft Limits.

**Independent test**: Create, Gather, cancel/confirm deletion, and open Settings from the tray; lower Limits and cross them without losing writes.

- [X] T011 [P] [US3] Review `src-tauri/src/app/tray.rs`, `crates/domain/src/settings.rs`, `crates/domain/src/caps.rs`, `crates/commands/src/caps.rs`, and `src-tauri/src/app/settings_workflow.rs`; record coverage in `specs/003-ranch-today/validation/us3.md`, including defaults 5/7 and the model constraint “Completed Tasks count because all stored Tasks are counted; writes above a threshold remain permitted.” Agent Sessions are excluded from the evaluator.
- [X] T012 [US3] Execute V4 tray/Settings actions from `specs/003-ranch-today/quickstart.md`; record New Focus, Gather, delete cancellation/confirmation, Agents toggle, Settings (Preferences menu naming on macOS), quit, immediate application of app edits, and restart persistence in `specs/003-ranch-today/validation/us3.md`.
- [X] T013 [US3] Execute V4 Focus and Task Limit crossings with notification settings enabled and disabled; record successful over-threshold writes, red tray state, completed-Task counting, and one allowed notification per crossing without repeated notifications while above in `specs/003-ranch-today/validation/us3.md` (SC-004). Cross-reference the separate Agent exclusion proof from US5 when available; do not require US5 to validate Focus-only crossings.

**Checkpoint**: Tray/Settings behavior and Limit crossings have results; live Agent exclusion is separately traced to US5.

## Phase 6: US4 — Hand-Edit the Markdown (P2)

**Goal**: Verify canonical file ownership, external edit latency, and safe optional sidecars.

**Independent test**: Append/Complete a checkbox in a fixture file, measure save-to-card visibility, and verify work remains usable across restart and malformed optional sidecars.

- [X] T014 [P] [US4] Review `crates/domain/src/parse.rs`, `crates/storage/src/focus_store.rs`, `crates/storage/src/watcher.rs`, and `src-tauri/src/app/paths.rs`; record coverage in `specs/003-ranch-today/validation/us4.md` for “Missing/corrupt optional sidecars load as no Timer.”, nonempty checkbox parsing, derived `focus_id:index` identity with “no stable persisted Task ID.”, and current data-root resolution. Record FR-013 as an observation; do not add stable IDs, reorder repair, or a migration.
- [X] T015 [US4] Execute V2 and the V1 disposable-sidecar checks from `specs/003-ranch-today/quickstart.md`; record external checkbox add/Complete, measured latency against about one second, restart durability, Settings hand-edit applying on next launch, and missing/corrupt optional sidecar behavior in `specs/003-ranch-today/validation/us4.md` (SC-003). Preserve fixture Markdown and distinguish measured UI timing from watcher debounce.

**Checkpoint**: External editing is measured; Task Timer drift remains a documented limitation, not an acceptance promise.

## Phase 7: US5 — See an Agent without Tracking It as Work (P3)

**Goal**: Verify opt-in observation, explicit installation, delivery diagnostics, and domain independence.

**Independent test**: Observe one session after a Firing, toggle its Animal on/off, confirm it cannot open a Focus card or affect work/Limits, and distinguish healthy configuration from delivery.

- [X] T016 [P] [US5] Review `crates/domain/src/session/agent_session.rs`, `crates/domain/src/agents/hooks.rs`, `crates/storage/src/agent_hooks/journal.rs`, `crates/storage/tests/agent_hook_socket.rs`, `src-tauri/src/app/claude_plugin.rs`, `src-tauri/src/app/claude_hook.rs`, and `src/components/App.test.tsx`; record existing coverage in `specs/003-ranch-today/validation/us5.md` for `activity: Working / Idle`, `action: Start / Working / Idle / End`, optional `session_id`/`pen`, unchanged Firings, bounded absent-app delivery, ephemeral sessions, and configuration health versus observed delivery.
- [X] T017 [US5] Execute the non-installing V5 UI checks from `specs/003-ranch-today/quickstart.md`; record default-off behavior, enabling without silent installation, and Install Hooks dialog/action separation in `specs/003-ranch-today/validation/us5.md`. Use isolated automated plugin install/repair and delivery results from T004; never present those fixtures as real-session evidence.
- [X] T018 [US5] Attempt the optional V5 live check only in an independently usable disposable Claude configuration, with matching Ranch/Claude environment values and explicit Install Hooks; otherwise record that live evidence is unverified and why. Open the Agent Hooks window and verify that wiring/configuration health, the live session roster, and recent Hook Firings are visible, including a Firing that changed nothing. In `specs/003-ranch-today/validation/us5.md`, separately record those window checks, plugin health, a post-listener Firing/unchanged repeat, non-editable Agent Animal, unchanged work files/Limits, and toggle hiding Agents while retaining Focuses/plugin (FR-011, SC-004, SC-005). Mark any unavailable window or live checks unverified. Do not read personal transcripts, borrow credentials, perform cold-start census, or treat Windows delivery as supported.

**Checkpoint**: Automated, UI, and live evidence are distinguished. Unavailable live delivery leaves its acceptance claim open.

## Phase 8: Polish and Cross-Cutting Reconciliation

**Purpose**: Consolidate truthful baseline evidence without turning gaps into feature implementation.

- [X] T019 Reconcile V7 observations with `specs/003-ranch-today/research.md`, `issues/056-task-timer-binding.md`, `issues/059-self-describing-hook-frame.md`, `issues/060-session-staleness-sweep.md`, `issues/061-hook-change-events.md`, and `issues/062-stable-hook-client-path.md`; record current status and owning Draft follow-ons in `specs/003-ranch-today/validation/pass-record.md`. Explicitly retain ADR-0002 accepted-direction/current-code drift and FR-013–FR-015 observation status; do not promote issue proposals to requirements or implement their fixes.
- [X] T020 Execute the V6 scoped boundary/non-goal review and, where available, packaged-process network observation in `specs/003-ranch-today/quickstart.md`; record tools, exercised actions, results or unverified gaps, and macOS/Windows/Linux evidence in `specs/003-ranch-today/validation/pass-record.md` (SC-006). Use `Taskfile.yaml`, `.github/workflows/windows.yml`, and `.github/workflows/release.yml` to distinguish compile, package, and native proof; run relevant platform gates only when changes require them, and do not trigger release workflows. Vite/explicit Claude CLI activity is reported separately from product runtime traffic.
- [X] T021 Consolidate `specs/003-ranch-today/validation/us1.md` through `specs/003-ranch-today/validation/us5.md` into a full FR/SC evidence index in `specs/003-ranch-today/validation/pass-record.md`; classify every entry as passed, failed, unverified, or observation, link the exact supporting task/result, and route new gaps to narrow follow-on issues without silently rewriting the spec, accepted ADRs, or acceptance bar.
- [X] T022 Verify links, checklist format, task/result consistency, whitespace, and unchanged unrelated files for `specs/003-ranch-today/tasks.md` and `specs/003-ranch-today/validation/`; record the final Constitution I–VI check, remaining acceptance gaps, and completion limits in `specs/003-ranch-today/validation/pass-record.md`. Do not claim the product baseline fully verified with failed/unverified criteria, or claim shipped completion without independent review, green CI, and merge.

## Coverage Index

| Requirement / outcome | Tasks |
|---|---|
| FR-001, FR-002 | T007–T008, T010, T014–T015 |
| FR-003 | T007, T009–T010 |
| FR-004, FR-009 | T005–T006, T012 |
| FR-005 | T008, T016, T018 |
| FR-006, FR-008 | T011–T012, T015 |
| FR-007 | T011, T013, T016, T018 |
| FR-010, FR-011 | T016–T018 |
| FR-012, FR-017 | T006, T020 |
| FR-013 | T014, T019 (observation only) |
| FR-014, FR-015 | T016, T019 (observation only) |
| FR-016 | T020 (separate platform evidence; no fabricated build result) |
| FR-018, FR-019, FR-020 | T010, T018, T020 |
| FR-021, FR-022 | T005–T006, T011–T012, T016, T020 |
| SC-001 | T006 |
| SC-002 | T008–T010 |
| SC-003 | T015 |
| SC-004 | T013, T018 |
| SC-005 | T017–T018 |
| SC-006 | T020 |

T021 reconciles all entries; T022 checks the final evidence and governance claims.

## Dependencies and Execution Order

```text
T001 → T002 → T003 → T004
                      |
                      +→ US1: T005 → T006
                      +→ US2: T007 → T008 → T009 → T010
                      +→ US3: T011 → T012 → T013
                      +→ US4: T014 → T015
                      +→ US5: T016 → T017 → T018
                      |
all story results → T019 → T020 → T021 → T022
```

- All stories depend on the foundation, not on another story's successful outcome. Create fresh story-specific synthetic state as needed; do not rely on a previous story's mutations.
- After T004, T005/T007/T011/T014/T016 can run together: they read source and write different story notes. They reference T004's results without running concurrent generated-type/build gates.
- Native tasks share the foreground app/OS surface and must run one at a time. Prefer priority order US1 → US2 → US3 → US4 → US5; independent story fixtures allow a different order.
- US3's Focus-only crossing result stands alone; its full Agent exclusion claim incorporates T018. US5 uses its own fixture and does not depend on T013.
- A failed gate does not prohibit read-only diagnosis or recording native observations; it prohibits calling failed coverage passed or claiming readiness.

## Parallel Examples per Story

These are scheduling examples, not instructions to spawn agents during task generation.

| Story | Safe parallel example after T004 | Sequential native follow-through |
|---|---|---|
| US1 | T005 alongside T007 (different notes) | T006 after T005 and when foreground app is free |
| US2 | T007 alongside T011 | T008 → T009 → T010 after T007 |
| US3 | T011 alongside T014 | T012 → T013 after T011 |
| US4 | T014 alongside T016 | T015 after T014 |
| US5 | T016 alongside T005 | T017 → T018 after T016 |

## Implementation Strategy

**MVP first**: Complete T001–T006 and stop to assess the ambient Focus reminder. This establishes only US1, not Timer, tray, hand-edit, or Agent acceptance.

**Incremental validation**: Add US2, then US3, US4, and US5; preserve separate story results and execute shared native work serially. Reuse T004's unchanged-code automated gate rather than adding new tests or rerunning whole suites per story.

**Stop condition**: Finish T019–T022 with all evidence reconciled and remaining gaps explicit. A finished evidence pass may still have unverified product criteria. No runtime fix, new abstraction, approval-status change, PR, merge, or release is implied by checking off this documentation workflow.

## Phase 9: Convergence

**Assessment (2026-10-17):** 28 FR/SC entries, 16 acceptance scenarios, six plan decisions, four validation slices, and Constitution I–VI checked. Seven partial acceptance-evidence gaps remain: five HIGH, two MEDIUM. Existing implementation and automated proof do not establish the native results below. No additional confirmed code defect or constitutional violation was established by this bounded assessment.

**Constitution compliance:** This phase preserves local synthetic state, independent domains, typed boundaries, and explicit integration actions (I–IV). It requires actual acceptance evidence before completion claims (V), preserves Draft status and source classifications (VI), and grants no exception. FR-013–FR-015 remain observations; their fixes and stale follow-on issue contracts stay outside this baseline.

**Owner execution direction (2026-10-17):** Ship the existing implementation and these baseline documents through independent review, green CI, merge, and release before the owner performs native testing. T023–T028 remain open for that later test pass; T029 still requires successful matching platform packages. This direction does not mark acceptance passed, promote Draft artifacts, or authorize implementing Draft follow-ons. See the [owner direction record](validation/pass-record.md#owner-direction--ship-before-later-native-testing-2026-10-17).

**Completion rule:** These tasks close acceptance gaps left open by the availability assessment in T001–T022. Mark a task complete only after its stated proof is obtained and recorded. If native access, a disposable Claude setup, inspection tools, or a target platform is unavailable, record the limitation and leave the task unchecked. Do not replace observations with source inspection, historical CI, or another note that the check was unavailable. A failed observation remains an open acceptance gap and needs separately scoped resolution; this phase authorizes no product changes, new test infrastructure, or release publication.

- [ ] T023 [US1] **HIGH — F1:** Execute V3 in `specs/003-ranch-today/quickstart.md` with native app control and isolated synthetic data; observe first-launch example creation and non-reappearance after deletion/restart, then identify two Focuses by visible labels. Verify one Pig per Focus, drag/toss/Gather to primary, enabled-display boundaries/no gaps, empty-space OS click-through, and no unsolicited attention. Record platform, source revision, actions and actual outcomes in `specs/003-ranch-today/validation/us1.md` and the corresponding `validation/pass-record.md` rows; leave unobserved items open per SC-001, US1/AC1–AC3, FR-004, FR-009, FR-017, FR-021, and plan: ambient-shell validation (partial).
- [ ] T024 [US2] **HIGH — F2:** Complete native V1 from `specs/003-ranch-today/quickstart.md`: Focus rename/duplicate/delete, Task add/edit/Complete/undo/Delete and blank rejection, outside-click/Escape closure, and matching canonical files. Verify Focus/Task Timer start/restart/Clear, shared saved chips across pickers/restart, chip removal preserving running Timers, Focus growth/expiry appearance/tray membership, add-Task revival, and Task expiry independence. Compare synthetic work files across expiry to establish timer-only writes. Record actual observations and file outcomes in `specs/003-ranch-today/validation/us2.md` and its `validation/pass-record.md` rows per SC-002, US2/AC1–AC5, FR-001–FR-003, FR-005, and plan: files-and-Timers validation (partial).
- [ ] T025 [US4] **HIGH — F3:** Execute native V2 and the disposable-sidecar cases from `specs/003-ranch-today/quickstart.md`; externally add/Complete checkbox Tasks and measure save-to-visible-card elapsed times against the approximately one-second outcome. Verify native restart durability, hand-edited Settings applying on next launch, and usable Focuses with missing/corrupt optional sidecars. Record measurement method, actual times, revision/platform, and results in `specs/003-ranch-today/validation/us4.md` and its `validation/pass-record.md` rows; do not substitute debounce duration for end-to-end timing or repair positional Timer drift per SC-003, US4/AC1, FR-001, FR-008, and plan: hand-edit validation (partial).
- [ ] T026 [US3] **HIGH — F4:** Execute native V4 in `specs/003-ranch-today/quickstart.md`: verify tray New Focus, Gather, delete cancellation/confirmation, Settings/Preferences naming, quit, immediate Settings application and restart persistence. Verify defaults 5/7, then cross lowered Focus/Task Limits while retaining writes, observe the red tray icon and one enabled notification per crossing without repeats, repeat with notifications disabled, and confirm completed Tasks count. Record evidence in `specs/003-ranch-today/validation/us3.md` and its `validation/pass-record.md` rows per SC-004, US3/AC1–AC3, FR-006–FR-008, and plan: ambient-shell validation (partial). T028 supplies the separate live-Agent exclusion proof; SC-004 cannot close until both portions are supported.
- [ ] T027 **HIGH — F5:** Obtain a matching packaged Ranch build using existing `Taskfile.yaml` build commands if needed, then execute V6 from `specs/003-ranch-today/quickstart.md` with process-scoped network inspection while exercising the core synthetic scenarios. Record build identity, platform, inspection tool, process, observation window, exercised actions, and actual connection/listener results in `specs/003-ranch-today/validation/pass-record.md`; separate Vite traffic and explicit Claude CLI effects from product runtime. Source searches alone do not close this task per SC-006, FR-012, and plan: runtime-boundary validation (partial).
- [ ] T028 [US5] **MEDIUM — F6:** Execute native/live V5 from `specs/003-ranch-today/quickstart.md` in an independently usable disposable Claude configuration with matching Ranch/Claude environment values and an explicit Install Hooks action. Verify default-off and opt-in visibility, no silent installation, a post-listener session Firing, Agent non-editability, unchanged Focus/Task files and Limits, and toggle-off hiding Agents while retaining Focus Animals/plugin. Open Agent Hooks and observe wiring/configuration health, live sessions, and recent Firings including an unchanged repeat; distinguish configuration health from real delivery. Record actual results in `specs/003-ranch-today/validation/us5.md` and its `validation/pass-record.md` rows without borrowing credentials or reading personal transcripts per SC-005, SC-004 (Agent exclusion), US5/AC1–AC4, FR-010–FR-011, and plan: Agent-boundary validation (partial).
- [ ] T029 **MEDIUM — F7:** Obtain successful Windows and Linux package-build evidence matching the assessed runtime source, using the existing non-publishing Windows workflow in `.github/workflows/windows.yml` and package commands/platform prerequisites defined by `.github/workflows/release.yml`. Use an appropriate target environment or matching recorded build results; do not create new pipelines or dispatch publishing release workflows. Record source/build identity, target, commands, exit status, and package artifacts in `specs/003-ranch-today/validation/pass-record.md`; workflow configuration or a compile-only result is not package proof, and Windows hook delivery remains unsupported per FR-016 and plan: retained platform compatibility (partial).

**Execution and evidence:** Keep native work serial on the shared OS surface. Each task updates its story note where applicable and the shared evidence index, retaining earlier results as historical observations. T026's Focus/Task checks can finish independently; its Agent comparison is supplied by T028. Before declaring convergence, reconcile every affected FR/SC and acceptance scenario to the obtained proof. Preserve the existing current-source automated gate result unless relevant source changes or failures justify rerunning it. If required access remains unavailable, keep this phase open rather than cycling another implement/converge pass as proof.

Path clarification for this phase: shortened `validation/pass-record.md` references in T023–T026 and T028 mean `specs/003-ranch-today/validation/pass-record.md`.
