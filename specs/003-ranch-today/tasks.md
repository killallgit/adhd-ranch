# Tasks: Ranch Today Baseline Validation

**Input**: Design documents in `specs/003-ranch-today/`: [plan](plan.md), [spec](spec.md), [research](research.md), [data model](data-model.md), [contracts](contracts/current-behavior.md), [quickstart](quickstart.md).

**Status**: Draft execution plan, generated 2026-10-17; reconciled 2026-10-18 against the clarified spec and revised plan. The specification remains Draft. Generating this list does not approve product changes or certify current behavior.

**Scope**: Preserve the baseline validation work and add only the owner-selected startup correction in T030–T035: remove automatic seeding, load existing work/Settings, and start empty without work. No dependency, schema, generated-type, onboarding, click-through, Agent integration, or FR-013–FR-015 change is authorized. Reuse existing tests; delete only the obsolete seed-behavior tests with the removed adapter and replace their acceptance promise with V0 native proof. No new test harness or source-text assertions.

**Organization**: Setup → foundation → five independently verifiable user stories → cross-cutting reconciliation. New tasks start unchecked. Existing IDs, completed task descriptions, checkbox states, and release proof are retained; the startup correction appends T030–T035 without duplicating earlier tasks. Mark an execution task done when its stated work and honest result record are complete; a recorded failure or unverified check is not a passed acceptance criterion.

**Execution record (2026-10-17)**: The current-main automated gate passed. Native acceptance remains unverified; T029 platform package builds passed in v0.1.5 as detailed in [the pass record](validation/pass-record.md). Checked tasks record completed evidence assessment, not passed product acceptance; all six success criteria still need end-to-end proof.

**Reconciliation scope (2026-10-18)**: T001–T022 and T029 retain their completed descriptions and states as historical evidence, not proof of the corrected runtime. T023–T028 remain open; T030–T035 are new unchecked tasks. Native/live work, including T034, remains deferred until the owner resumes it. The old seed expectation in T002 is superseded by the no-seeding rule and T034; T015's approximate timing wording is superseded by T025. A fresh runtime gate is mandatory after T031; old v0.1.5 gate/package proof applies only to its original source.

## Format and Paths

Tasks use `- [ ] Tnnn [P?] [USn?] description`. `[P]` marks independent source checks with distinct output files after the foundation completes. Other tasks run sequentially.

All file paths below are relative to the repository root. Execution creates one shared `specs/003-ranch-today/validation/pass-record.md` and one evidence note per story (`us1.md` through `us5.md` in the same directory). These outputs already exist from the prior execution; task generation neither recreates them nor changes their results.

Evidence notes record source pointers, commit/platform, exact commands and exit statuses, observed outcomes, pass/fail/unverified status, and discrepancies. Never substitute source inspection or past CI for a fresh test or native result. Do not include personal data, credentials, raw transcripts, or machine-specific scratch paths in committed evidence.

## Constitution Compliance Check

| Principles | Execution constraint |
|---|---|
| I — Local state | Use synthetic isolated data; inspect atomic-write/sidecar coverage; no personal work/configuration writes or outbound product feature. Automatic seed writes remain a current Constitution I / FR-019 violation; the owner-selected remedy is T031, with data preservation and native proof in T034. No exception is approved. |
| II — Domain separation | Validate Focus/Task/Timer independently from Agent activity; only the flat render contract joins them. |
| III — Typed boundaries | Reuse existing source owners and tests; generated contracts remain generated. The only planned implementation is deletion of the seed adapter and its composition references in T031; no new boundary or generated-contract edit. |
| IV — Explicit integrations | Toggle is not install. Distinguish validated configuration from observed delivery; live checks use isolated configuration and explicit installation. |
| V — Observable completion | Bound tasks by evidence outputs; synchronize with current main before the gate; retain failures and unverified outcomes. Shipping completion still requires review, green CI, and merge. |
| VI — Authority | Keep Draft status; accepted ADRs outrank observed shortcomings. Record ADR-0002 drift and route follow-ons without silently changing accepted decisions. |

**Result**: PASS for the bounded startup correction design in [plan.md](plan.md#constitution-check): the owner has selected no seeding and resumption from saved state. T030–T035 make that remedy executable. Current runtime compliance is still failed/unverified until implementation and actual startup proof; the policy decision does not close C1. Open-card click-through still differs from FR-004/FR-005 and remains outside this correction. These tasks grant no exception and do not change the specification's Draft status.

**Gap closure constraints**: T023/T024 must retain the source-confirmed open-card mismatch in their story notes and `specs/003-ranch-today/validation/pass-record.md`, with pointers to `src/hooks/useAnimalMovement.ts`, `src/components/AnimalDetail.tsx`, and `src-tauri/src/display/overlay.rs`. Passing outside-click closure alone cannot close pass-through. A separate runtime slice must first select and authorize an outside-click mechanism that preserves delivery to the underlying app. The seed policy is now resolved: T031 removes the writer in `src-tauri/src/app/mod.rs` and `src-tauri/src/app/seed.rs`; T034 owns V0 proof. T023 incorporates that proof for US1/AC4–AC5 without duplicating the fixture run. Existing seed tests or the historical FR-019 source-scope PASS do not establish corrected startup behavior. Do not edit the historical pass record merely to make it appear compliant; append dated observations when execution resumes. ADR-0002 drift remains separately routed by T019 and is not implemented here.

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

**Independent test**: With two explicitly created synthetic Focuses, identify both by visible Pig labels; verify absent/Running Focus Timer → Walking, Expired → Resting independently of Task Timers, interactive Animals/card, empty-space pass-through with card closed and open, outside/Escape closure, drag/toss/Gather, and no unsolicited attention. T023 owns the ambient native evidence and incorporates T034's independent no-seeding/saved-state proof for US1/AC4–AC5; the source-confirmed open-card mismatch remains a gap until separately fixed and proved.

- [X] T005 [P] [US1] Review `crates/commands/src/focus_animals.rs`, `crates/commands/src/render_scene.rs`, `crates/commands/tests/render_scene_contract.rs`, and `src/lib/animalMovement.test.ts`; write `specs/003-ranch-today/validation/us1.md` with source/coverage pointers for the model constraints “Species is Pig; Motion is Walking or Resting.”, “Size is fixed or linearly sampled over time.”, optional `regionId`, and “Movement and sprites consume neither domain types nor source discriminators.”; distinguish generated wire shape from native behavior.
- [X] T006 [US1] Execute V3 in `specs/003-ranch-today/quickstart.md` using two fresh synthetic Focuses; record labels/one Pig per Focus, drag/toss, Gather, primary-only defaults, display enablement/no gap traversal, empty-space click-through, and ambient attention observations in `specs/003-ranch-today/validation/us1.md`. Record unavailable multi-display/native checks as unverified rather than inferred from tests (SC-001).

**Checkpoint**: US1 has source evidence plus explicit native results or remaining gaps; this is the suggested MVP validation scope.

## Phase 4: US2 — Work a Focus from Its Animal (P1)

**Goal**: Verify card editing and Timer ownership without changing their implementation.

**Independent test**: Open one Focus card, add/edit/Complete/Delete Tasks, start/Clear Timers, compare files, and independently exercise Focus versus Task expiry. Duplicate a source with mixed Task completion and Focus/Task Timers: verify fresh identity/creation time, copied description and ordered Task text/completion, collision-safe copy title, no copied Timers, and an unchanged source (US2/AC6, T024).

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

**Independent test**: Add and separately Complete a checkbox in a valid local Focus file while its card is already open and visible. Measure successful save completion → corresponding Task row visibly matching the file; each observation must be ≤1.0 second. Record endpoints, common time base, precision/uncertainty, and actual times; insufficient precision leaves SC-003 unverified. Verify restart and malformed optional-sidecar behavior independently (T025).

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

Historical assessment tasks remain linked; the current open acceptance tasks and completed package proof are included explicitly.

| Requirement / outcome | Historical assessment | Current acceptance task / status |
|---|---|---|
| FR-001, FR-002 | T007–T008, T010, T014–T015 | T024/T025 existing work; T030–T035 startup correction/proof (FR-001). T024 includes duplication and US2/AC6. |
| FR-003 | T007, T009–T010 | T024 Timer behavior; T031/T033/T034 preserve loaded timestamps and normal expiry. |
| FR-004, FR-009 | T005–T006, T012 | T023; motion matrix and open/closed-card click-through (known mismatch). |
| FR-005 | T008, T016, T018 | T023/T024 outside/Escape and card interaction; T028 Agent non-editability. |
| FR-006, FR-008 | T011–T012, T015 | T026 tray/Settings; T025 Settings hand-edit/restart; T034 preserves saved Settings at startup. |
| FR-007 | T011, T013, T016, T018 | T026 Focus/Task crossing; T028 live-Agent exclusion. |
| FR-010, FR-011 | T016–T018 | T028 (open). |
| FR-012, FR-017 | T006, T020 | T027 process network proof; T023 ambient attention. |
| FR-013 | T014, T019 | T025 records positional drift only; no preservation or repair requirement. |
| FR-014, FR-015 | T016, T019 | T028 distinguishes known transport/legacy observations from live proof; no fixes authorized. |
| FR-016 | T020 | T029 complete for v0.1.5 only; T033 fresh host/Windows gates for corrected source, with no new package claim. |
| FR-018, FR-019, FR-020 | T010, T018, T020 | T024 timer-only writes; T028 Agent isolation; T027 network scope. T030–T035 implement/verify the no-seeding remedy for FR-019; native proof remains open despite historical source-scope PASS. |
| FR-021, FR-022 | T005–T006, T011–T012, T016, T020 | T023 displays/Gather/Pig; T026 tray; T028 Claude-only Agent observations. |
| SC-001 | T006 | T023 (open); T034 separately covers US1/AC4–AC5 without claiming full ambient acceptance. |
| SC-002 | T008–T010 | T024 (open). |
| SC-003 | T015 | T025 (open): ≤1.0 second with exact endpoints and valid-file/visible-card conditions. |
| SC-004 | T013, T018 | T026 and T028; both portions need evidence. |
| SC-005 | T017–T018 | T028 (open). |
| SC-006 | T020 | T027 (open). |
| US1/AC4–AC5 | No prior native proof | T031/T033 implement/check; T034 V0 proves startup; T035 reconciles delivery; T023 incorporates results. |

T021/T022 are historical reconciliation/format checks, not evidence for the 2026-10-18 clarifications. When T023–T028 execute, update their story notes and the shared pass-record index with dated results and remaining gaps. T029's existing release proof remains intact for the old source. T030/T033/T035 own the new correction's setup, gates, and final issue/evidence reconciliation; T031/T032 implement/document it and T034 proves actual startup. No FR/SC or new task is unmapped.

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

The graph above describes the original T001–T022 assessment order; it does not instruct rerunning completed tasks. Current native execution, when resumed by the owner, is:

```text
completed T001–T022 foundation/evidence
  → T030–T033 startup implementation/gates → T034 V0 (when testing resumes)
  → T023 US1 → T024 US2 → T026 US3 → T025 US4 → T028 US5 → T027 network scope
  → reconcile affected story notes + validation/pass-record.md within those tasks
completed T029 package proof → retained, no rerun without a relevant change
```

The native order is a serial schedule for the shared OS surface, not a dependency between independent story fixtures. T026 and T028 jointly establish SC-004. The startup correction uses the separate T030–T035 dependency graph below and can proceed through automated work before native testing resumes. Other runtime fixes remain outside this plan; read-only diagnosis or an honestly recorded native failure may proceed when testing resumes.

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

**MVP first**: Deliver the independently reviewable startup correction through T030–T033, then obtain its actual V0 proof in T034 and reconcile in T035. Do not wait for unrelated native acceptance or implement the separate click-through fix in this slice. For full US1, reuse completed T001–T006 assessment/setup and complete T023 once native testing resumes. US1 acceptance still requires its clarified motion and click-through evidence; the known open-card mismatch cannot be called passed. This establishes only the ambient Focus reminder, not Timer, tray, hand-edit, or Agent acceptance.

**Incremental validation**: Continue with T024 (US2), T026 (US3), T025 (US4), T028 (US5), and T027 (process scope) after US1 assessment; preserve separate fixtures/results and execute shared native work serially. Failures may be recorded without blocking independent observations, but remain acceptance gaps. T031 changes runtime source, so T033 must obtain fresh automated results; v0.1.5 gates/packages remain historical. Add no new tests for task-list prose.

**Stop condition**: T019–T022 are already assessed. Startup completion requires T031–T034 proof plus T035's delivery/evidence reconciliation. Full baseline completion separately requires actual T023–T028 evidence and corresponding updated FR/SC/acceptance-scenario rows, with unresolved failures left open; do not cycle another availability assessment as proof. A finished evidence pass may still have unverified product criteria. The startup correction has its own completion promise and fresh checks; completing it does not close full US1, the click-through gap, or the remaining baseline tasks. No approval-status change or full-baseline/release claim follows from task markers.

## Phase 9: Convergence

**Assessment (2026-10-17, before release):** 28 FR/SC entries, 16 acceptance scenarios, six plan decisions, four validation slices, and Constitution I–VI checked. Seven partial acceptance-evidence gaps remain: five HIGH, two MEDIUM. Existing implementation and automated proof do not establish the native results below. No additional confirmed code defect or constitutional violation was established by this bounded assessment.

**Clarification reconciliation (2026-10-18):** The 16-scenario assessment above is historical; the current spec has 19 acceptance scenarios including US2/AC6 and US1/AC4–AC5. T023/T024/T025 below supersede their earlier unexecuted descriptions, with no checkbox or completed-task change. Automatic seed creation is a Constitution I violation with an owner-selected removal remedy in T030–T035, not a preservation criterion; the open-card click-through mismatch remains unresolved.

**Constitution compliance:** This phase preserves local synthetic state, independent domains, typed boundaries, and explicit integration actions (I–IV). It requires actual acceptance evidence before completion claims (V), preserves Draft status and source classifications (VI), and grants no exception. FR-013–FR-015 remain observations; their fixes and stale follow-on issue contracts stay outside this baseline.

**Owner execution direction (2026-10-17):** Ship the existing implementation and these baseline documents through independent review, green CI, merge, and release before the owner performs native testing. T023–T028 remain open for that later test pass; T029 required matching platform packages at the time of that direction; it is now complete per the release checkpoint below. This direction does not mark acceptance passed, promote Draft artifacts, or authorize implementing Draft follow-ons. See the [owner direction record](validation/pass-record.md#owner-direction--ship-before-later-native-testing-2026-10-17).

**Completion rule:** These tasks close acceptance gaps left open by the availability assessment in T001–T022. Mark a task complete only after its stated proof is obtained and recorded. If native access, a disposable Claude setup, inspection tools, or a target platform is unavailable, record the limitation and leave the task unchecked. Do not replace observations with source inspection, historical CI, or another note that the check was unavailable. A failed observation remains an open acceptance gap and needs separately scoped resolution; Phase 9 remains validation-only and authorizes no product changes, new test infrastructure, or release publication. Only the separate Phase 11 startup task T031 changes runtime source.

- [ ] T023 [US1] **HIGH — F1:** When the owner resumes native testing, execute V3 in `specs/003-ranch-today/quickstart.md` with native app control and isolated synthetic data. Use T034's V0 results for no-seeding and saved-state resumption (US1/AC4–AC5), rather than rerunning its cases. If the corrected app creates any example work, record a failed FR-001 / FR-019 result; the policy is resolved. Empty-root behavior must not depend on the legacy seed marker. Explicitly create two Focuses and identify their Pig labels. Verify one Pig per Focus, “absent or Running Focus Timer → Walking; Expired → Resting.” and “Task Timer state does not control Animal motion”; independently vary Task Timer state. Check drag/toss/Gather, primary-only/default and enabled-display boundaries/no gaps, card controls receiving interaction, empty-space OS click-through with no card and with an open card, simultaneous outside-click closure/delivery to the underlying app, Escape closure, and no unsolicited attention. Retain the known whole-overlay hit-region/backdrop failure: closure alone does not pass FR-004/FR-005 / US1/AC2. Record revision/platform, actions, actual outcomes, startup proof status and unresolved click-through follow-on in `specs/003-ranch-today/validation/us1.md` and `specs/003-ranch-today/validation/pass-record.md`; append a dated clarification result without rewriting historical evidence. Leave missing/failed acceptance proof open per SC-001, US1/AC1–AC5, FR-001, FR-004, FR-005, FR-009, FR-017, FR-019, and FR-021. This task records native evidence only; T031 owns the startup correction and click-through remains outside scope.
- [ ] T024 [US2] **HIGH — F2:** When native testing resumes, execute V1 in `specs/003-ranch-today/quickstart.md`: Focus rename/delete, Task add/edit/Complete/undo/Delete, blank rejection, matching canonical files, interactive card controls, and outside-click/Escape closure. For duplication (US2/AC6), give the source a description, ordered Tasks with mixed completion states, and Focus/Task Timers; verify the independent copy has newly assigned identity/creation time, copied description and Task text/order/completion, and unchanged source files/state. Verify title `<source title> copy`, then suffix starting at 2 when identity collisions occur by duplicating the original again; “Neither the Focus Timer nor any Task Timer is copied.” Keep new copy Timers absent. Verify Focus/Task Timer start/restart/Clear, saved chips across pickers/restart, chip removal preserving running Timers, Focus growth/expiry appearance/tray membership, add-Task revival, and Task expiry independence. Compare synthetic work files across expiry for timer-only writes. Record actual observations/file outcomes and all US2/AC1–AC6 results in `specs/003-ranch-today/validation/us2.md` and `specs/003-ranch-today/validation/pass-record.md` per SC-002, FR-001–FR-003, and FR-005. Cross-reference T023 for simultaneous outside-click pass-through; do not mark that known mismatch passed solely because the card closes. Missing/failed native proof keeps this task open; no fix is authorized.
- [ ] T025 [US4] **HIGH — F3:** When native testing resumes, execute V2 and disposable-sidecar cases in `specs/003-ranch-today/quickstart.md`. With a valid local Focus file and its already-open, visible Focus card, separately add and Complete a checkbox Task and successfully save. Measure each change from successful save completion until its corresponding Task row visibly matches the saved file; SC-003 requires at most 1.0 second. Record timing method, both endpoints on a common time base, precision/uncertainty, actual elapsed times, revision/platform, and outcomes in `specs/003-ranch-today/validation/us4.md` and `specs/003-ranch-today/validation/pass-record.md`. A slower result fails; insufficient precision or missing preconditions leave the criterion unverified. Verify restart durability, hand-edited Settings applying on next launch, and “Missing/corrupt optional sidecars load as no Timer.” Preserve synthetic Markdown and keep positional Timer drift observational; neither watcher debounce nor historical T015 proves the bound. Leave this task open without actual proof per SC-003, US4/AC1, FR-001, and FR-008; no repair is authorized.
- [ ] T026 [US3] **HIGH — F4:** Execute native V4 in `specs/003-ranch-today/quickstart.md`: verify tray New Focus, Gather, delete cancellation/confirmation, Settings/Preferences naming, quit, immediate Settings application and restart persistence. Verify defaults 5/7, then cross lowered Focus/Task Limits while retaining writes, observe the red tray icon and one enabled notification per crossing without repeats, repeat with notifications disabled, and confirm completed Tasks count. Record evidence in `specs/003-ranch-today/validation/us3.md` and its `validation/pass-record.md` rows per SC-004, US3/AC1–AC3, FR-006–FR-008, and plan: ambient-shell validation (partial). T028 supplies the separate live-Agent exclusion proof; SC-004 cannot close until both portions are supported.
- [ ] T027 **HIGH — F5:** Obtain a matching packaged Ranch build using existing `Taskfile.yaml` build commands if needed, then execute V6 from `specs/003-ranch-today/quickstart.md` with process-scoped network inspection while exercising the core synthetic scenarios. Record build identity, platform, inspection tool, process, observation window, exercised actions, and actual connection/listener results in `specs/003-ranch-today/validation/pass-record.md`; separate Vite traffic and explicit Claude CLI effects from product runtime. Source searches alone do not close this task per SC-006, FR-012, and plan: runtime-boundary validation (partial).
- [ ] T028 [US5] **MEDIUM — F6:** Execute native/live V5 from `specs/003-ranch-today/quickstart.md` in an independently usable disposable Claude configuration with matching Ranch/Claude environment values and an explicit Install Hooks action. Verify default-off and opt-in visibility, no silent installation, a post-listener session Firing, Agent non-editability, unchanged Focus/Task files and Limits, and toggle-off hiding Agents while retaining Focus Animals/plugin. Open Agent Hooks and observe wiring/configuration health, live sessions, and recent Firings including an unchanged repeat; distinguish configuration health from real delivery. Record actual results in `specs/003-ranch-today/validation/us5.md` and its `validation/pass-record.md` rows without borrowing credentials or reading personal transcripts per SC-005, SC-004 (Agent exclusion), US5/AC1–AC4, FR-010–FR-011, and plan: Agent-boundary validation (partial).
- [X] T029 **MEDIUM — F7:** Obtain successful Windows and Linux package-build evidence matching the assessed runtime source, using the existing non-publishing Windows workflow in `.github/workflows/windows.yml` and package commands/platform prerequisites defined by `.github/workflows/release.yml`. Use an appropriate target environment or matching recorded build results; do not create new pipelines or dispatch publishing release workflows. Record source/build identity, target, commands, exit status, and package artifacts in `specs/003-ranch-today/validation/pass-record.md`; workflow configuration or a compile-only result is not package proof, and Windows hook delivery remains unsupported per FR-016 and plan: retained platform compatibility (partial).

**Execution and evidence:** Keep native work serial on the shared OS surface. Each task updates its story note where applicable and the shared evidence index, retaining earlier results as historical observations. T026's Focus/Task checks can finish independently; its Agent comparison is supplied by T028. Before declaring convergence, reconcile every affected FR/SC and acceptance scenario to the obtained proof. Preserve the existing current-source automated gate result unless relevant source changes or failures justify rerunning it. If required access remains unavailable, keep this phase open rather than cycling another implement/converge pass as proof.

Path clarification for this phase: shortened `validation/pass-record.md` references in T023–T026 and T028 mean `specs/003-ranch-today/validation/pass-record.md`.


**Release checkpoint (2026-10-17):** T029 is complete: matching Windows smoke builds and the native v0.1.5 release matrix produced all required packages. All five downloaded release packages matched `SHA256SUMS.txt`; source/build identity and hashes are in [the release proof](validation/pass-record.md#release-package-proof-t029). T023–T028 remain unchecked for the owner’s later native/live test pass. No SC is promoted by package builds.

## Phase 10: Startup Correction — Setup and Foundation

**Purpose:** Scope the owner-selected correction as one shipping issue and prepare a synchronized checkout without disturbing unrelated work. Historical T001–T022/T029 results are retained, not rerun or reset as preparation.

- [X] T030 Prepare the startup slice using `specs/003-ranch-today/plan.md#startup-correction-slice` and `issues/README.md`: create `issues/063-resume-saved-work.md` with owning spec FR-001/FR-019, US1/AC4–AC5, the completion promise, explicit source scope, acceptance criteria, Constitution I–VI check, and deferred native-proof status. Check the proposed filename is unoccupied before creating it; do not overwrite unrelated work. Record input revisions, source/base commit and isolation in a new dated startup section of `specs/003-ranch-today/validation/pass-record.md`. Arrange an isolated branch synchronized with current main, carrying only the selected current 003 artifacts and startup slice; preserve all unrelated dirty work in place. This task prepares the issue/checkout, not native acceptance or a lifecycle-status change.

**Checkpoint:** Startup scope is reviewable and isolated. T031 and T032 may begin independently; no new app launch, personal-data copying, plugin installation, or migration is needed for setup.

## Phase 11: US1 — Resume Saved Work Without Seeding (P1, Startup MVP)

**Goal:** Ranch starts from saved work and Settings, creates no example Focus or Task, and starts empty when no work exists. This is independent of full US1 click-through/ambient acceptance.

**Independent test:** Execute V0 in `specs/003-ranch-today/quickstart.md` against the corrected host. Check empty roots with/without a legacy marker, saved work including an old example, nondefault Settings, and future/Expired/due-while-closed Focus/Task Timers. Preserve work and Timer timestamps; permit only normal expiry status changes.

- [X] T031 [P] [US1] After T030, remove `pub mod seed` and the sole `seed::ensure_example_focus` call from `src-tauri/src/app/mod.rs`; delete `src-tauri/src/app/seed.rs`, including the two tests requiring the superseded seed behavior. Keep store construction, directory setup, `load_settings`, IPC readers, projections, and `timer_expiry::spawn` unchanged. Preserve the data-model constraints “Existing example Focuses remain ordinary persisted Focuses; do not detect or delete them by title or identity.” and “Reloaded Timers retain `started_at`, `duration_secs`, and status.” Normal expiry may change status without resetting start/duration or changing Task text/completion. Leave `.example-focus-seeded` untouched and ignore its presence/absence; no replacement no-op function, startup service, flag, onboarding UI, schema migration, data rewrite, cleanup pass, or Agent-session restoration. Search all seed references before editing and confirm no remaining runtime caller afterward. This is the only runtime edit authorized by this list (FR-001/FR-019, US1/AC4–AC5).
- [X] T032 [P] [US1] After T030, update the startup description in `docs/architecture.md` to match the planned correction: load existing persisted work/Settings, start empty without work, never seed or reset work/Timers, preserve old examples/markers, and retain normal expiry. Reference the existing persistence/Timer owners rather than inventing a new startup abstraction. Keep current runtime observations distinct from the intended change until T031 is integrated; do not imply restoration of Animal positions, open cards, or ephemeral Agent Sessions. This file is independent of T031's source files; do not edit the shared evidence record in parallel.
- [X] T033 [US1] After T031/T032 and synchronization with latest main, run fresh `rtk task check` and `rtk task check:windows` using `Taskfile.yaml` on the corrected source. Inspect existing storage empty/missing-root, saved Focus/Task, Timer-sidecar, and Timer due/not-due/expiry results in `crates/storage/src/focus_store.rs` and `crates/commands/src/timers.rs`; preserve their actual assertions and all generated-artifact gates. Record exact commands, revision/platform, exit statuses, remaining source scope, and the deliberate retirement of the two obsolete seed tests in `specs/003-ranch-today/validation/pass-record.md`. Do not replace them with a source-text absence test or a no-op wrapper, lower a gate, or reuse old runtime/package results. A failed or unavailable required gate leaves T033 open; passing units/compilation do not prove native startup or a new package build.
- [ ] T034 [US1] After T033, when the owner resumes native testing, execute all four V0 cases from `specs/003-ranch-today/quickstart.md` on the corrected revision with isolated synthetic data and Agents as Animals off. Empty roots must stay free of Focuses/Tasks across launch/restart, with marker absent or present; no new/modified marker is allowed. Saved work/Settings, including an old Welcome Focus, must retain identities, text, completion and bytes; do not delete old examples. For future/Expired/due-while-closed Focus/Task Timers, verify “Reloaded Timers retain `started_at`, `duration_secs`, and status.” except the documented normal Running → Expired transition; start/duration and canonical work must not reset. Record case-by-case actual UI/file evidence, revision/platform, actions and pass/fail/unverified in `specs/003-ranch-today/validation/us1.md` and `specs/003-ranch-today/validation/pass-record.md`; link relevant Timer evidence to `specs/003-ranch-today/validation/us2.md`. T023 incorporates these results for US1/AC4–AC5 without repeating V0. Leave T034 unchecked while testing is deferred or any required case lacks passing proof. This task does not require or establish the separate click-through fix.

**Checkpoint:** T031/T032 implement/document the deletion; T033 proves current-source automated/platform gates; only actual T034 evidence proves startup acceptance. No full-baseline criterion is promoted by this slice.

## Phase 12: Startup Correction — Final Reconciliation

- [ ] T035 After T033/T034, reconcile the startup evidence in `specs/003-ranch-today/validation/pass-record.md`, `specs/003-ranch-today/validation/us1.md`, and `issues/063-resume-saved-work.md` with FR-001/FR-019 and US1/AC4–AC5. Verify links, input/source revisions, unchanged data formats, preservation of unrelated work, and `rtk git diff --check`; distinguish fresh results from the retained v0.1.5 record. Record the startup issue/PR, independent review, current-head green CI, and merged commit when obtained; close the shipping issue only after its complete acceptance and delivery proof exists. Keep T023–T028, click-through, and other FR/SC results at their actual status; startup proof does not close full US1 or all of 003. T035 remains open without required native/delivery evidence, even if implementation and automated checks are complete. No release or package-build claim follows from old T029 results.

### Startup dependencies and parallel work

```text
T030 → T031 source deletion ─┐
     → T032 architecture ────┴→ T033 fresh gates → T034 V0 (when testing resumes) → T035 final proof
completed original foundation ──────────────────────────┘
T034 results → T023's US1/AC4–AC5 mapping (no duplicate V0 run)
```

T031 and T032 may run in parallel after T030 because they change different files. T033 runs once after both are integrated; native tasks share the OS surface and run serially. T030, T033, T034, and T035 write the shared pass record at distinct dependency stages and must not run concurrently. The completed source-review lanes in the earlier per-story examples remain historical; no other open task is marked parallel.

**Execution order for the new work:** T030–T033 can proceed before the owner resumes native testing. Leave T034/T035 pending until their actual proof is available; do not rerun an availability assessment or mark deferred work complete. Native checks for the other stories remain independently scheduled as above. Startup-only implementation and automated success may be reported separately from complete product acceptance.
