# Implementation Plan: Ranch Today Baseline Validation

**Feature context**: `003-ranch-today` | **Date**: 2026-10-18 | **Spec**: [spec.md](spec.md)

**Checkout branch**: `main` at `7900cc445ac224464e254b31d4914b0a52055270` (feature context returned by setup: `003-ranch-today`)

**Status**: Draft. The specification remains Draft; these artifacts support owner review and do not promote it to approved authority.

**Input**: `specs/003-ranch-today/spec.md`, including the 2026-10-17 observation rule and the five 2026-10-18 clarifications, including the owner's refined startup answer: no seeding, resume persisted work, and start empty when none exists. The interaction, motion, latency, and duplication decisions remain in force.

## Summary

Establish a reviewable baseline for the existing local-first desktop product: Focus and Task editing, Timers, ambient Animals, tray and Settings, hand-edited Markdown, and optional Claude Code observations. Reuse existing code and tests, document current contracts, and prepare a repeatable validation pass. Add one bounded runtime correction selected by the owner: remove automatic startup example creation and use the existing persisted-state loading paths. No dependencies, storage migration, new IPC, onboarding UI, or archive work is proposed.

FR-013–FR-015 remain observations, not behavior to preserve. Automatic first-launch example creation is a current implementation violation of Constitution I / FR-001 / FR-019. The owner has now selected its remedy: no startup seeding, no reset of saved work or Timers. This plan defines that correction and grants no exception. ADR-0002's accepted transport direction is not fully implemented in current code; [research](research.md) records that discrepancy without approving it or absorbing its follow-on work into 003.

## Technical Context

**Language/Version**: Rust edition 2021, declared minimum 1.77; TypeScript ^5.6.3. Existing manifests and lockfiles govern installed versions.

**Primary Dependencies**: Tauri v2 (^2.1 API/CLI declarations), React ^18.3.1, Vite ^5.4.11, serde/serde_json, ts-rs-generated frontend types. No dependency changes.

**Storage**: Focus directories with `focus.md`, optional `timer.json` and `task-timers.json`; `settings.yaml` under the Ranch data root. Agent Sessions and recent Hook Firings are runtime observations, not user work. See [data model](data-model.md).

**Testing**: Existing Rust unit/integration tests, Vitest ^2.1.8, generated-type drift checks, and the native startup matrix in V0. Remove the obsolete seed-preservation tests with their deleted implementation; retain storage/Timer coverage and prove actual startup through native launch/restart, not a no-op wrapper or source-text assertion. [Quickstart](quickstart.md) distinguishes available coverage from checks actually executed.

**Target Platform**: macOS primary; retain Windows and Linux package compatibility. Windows has no hook delivery. The planned source deletion affects the shared Tauri startup composition; use the existing host and Windows cross-target gates, with native startup proof on macOS.

**Project Type**: Tauri desktop application with Rust workspace and React webviews; Tauri IPC is the frontend/core boundary.

**Performance Goals**: SC-003 requires at most 1.0 second from successful save completion of a valid local Focus file to the changed Task row matching it in an already-open, visible Focus card. This is an unmeasured acceptance target. Ambient rendering must not demand unsolicited attention; no new FPS or throughput target.

**Constraints**: User-controlled atomic local writes; safe malformed optional sidecars; independent Focus and Agent Session domains; no outbound product network, telemetry, model calls, or stored provider credentials. Integration install remains explicit.

**Scale/Scope**: One local user; one Animal per Focus plus optionally observed Claude Code sessions. Soft defaults 5 Focuses / 7 Tasks per Focus are thresholds, not capacity ceilings. Five user stories, 22 FR entries including three observations, six success criteria.

## Constitution Check

These gates assess the proposed startup correction and baseline validation design. They do not certify current runtime conformance or close existing acceptance gaps.

| Principle / gate | Before research | After design |
|---|---|---|
| I — User-owned local state | ERROR in current runtime: automatic startup example creation writes work without an authoring action. | PASS for the corrective design: remove that writer; reuse saved-state reads; preserve old work/markers and normal timer-only expiry. Runtime proof remains pending. |
| II — Independent domains | PASS: preserve the accepted renderer seam. | PASS: Agent activity remains separate from Focus Timers, editing, and Limits. |
| III — Functional core, typed edges | PASS: reuse existing layers and generated contracts. | PASS: delete the seed adapter and its composition references; no replacement abstraction, schema, or generated-type edit. |
| IV — Explicit integrations | PASS: no installation during planning. | PASS: isolated delivery proof; optional live check; configuration health is not delivery. |
| V — Observable small slices | PASS: baseline evidence only. | PASS: one startup correction with explicit source/proof boundaries, plus four baseline validation slices; new runtime code requires a fresh gate, review, and merge. |
| VI — Evidence and authority | PASS: keep the spec Draft and classify sources. | PASS: distinguish accepted ADR direction, observed code, and draft follow-ons; document existing transport and seed-writing drift. |

**Gate result:** PASS for the proposed startup correction design: the owner has selected a remedy consistent with Constitution I, with no exception needed. The current runtime still fails the no-seeding rule until the deletion is implemented and verified; the policy decision alone does not close C1. Phase 0/1 planning is complete for this bounded correction. The spec remains Draft; this command does not implement or ship it. Outside-click behavior and ADR-0002 drift remain separate unresolved work and are not bundled into the startup correction.

**Clarified behavior assessment (source inspection, not native proof):**

| Clarification | Assessment / next boundary |
|---|---|
| Startup | Policy resolved: load existing work/Settings; start empty if none exists; never seed, reset work, or restart Timers. Remove the seed call/module without deleting existing example Focuses or marker files. Normal expiry remains active. Current runtime correction and native proof are pending. |
| Outside-card click-through | Existing whole-overlay hit region/backdrop consumes outside clicks while closing the card. This differs from FR-004/FR-005 and US1/AC2. A separate runtime slice must design bounded Animal/card hit regions and an outside-click observation mechanism that does not swallow the underlying app’s click; current cursor polling alone observes position, not clicks. Native proof and relevant platform gates are required. No new global-input permission or platform mechanism is selected here. |
| Motion | Existing projection matches absent/Running → Walking and Expired → Resting, independent of Task Timers; native verification remains open. |
| Duplication | Existing workflow matches the clarified successful-copy semantics; new US2/AC6 still requires native evidence. |
| Update latency | Existing 200 ms debounce plus event/refetch is not measured end-to-end proof of ≤1.0 second; V2 defines the evidence. |

## Project Structure

### Documentation (this feature)

```text
specs/003-ranch-today/
├── spec.md
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
└── contracts/
    └── current-behavior.md
```

`tasks.md`, `validation/`, and the custom `checklists/baseline.md` already exist. This command preserves their history and checkbox states. The next task pass must append new unchecked tasks after T029 for the startup correction, documentation, fresh gates, and proof; update T023 for US1/AC4–AC5 and V0 while preserving checked T002's historical description. The current spec has 19 acceptance scenarios. Earlier constraints limiting every task to evidence-only work must be narrowed to allow this startup correction, while preserving the exclusions for click-through and FR-013–FR-015. Do not regenerate checked tasks or reuse v0.1.5 gates as proof of changed runtime source.

### Existing source and proof locations (repository root)

```text
crates/domain/src/       # Pure Focus, Task, Timer, Settings, session and hook types
crates/storage/src/      # Files, watcher, hook transport and journal
crates/storage/tests/    # Socket/client and storage fixtures
crates/commands/src/     # Workflows, Limits and render projections
crates/commands/tests/   # Render wire contract
crates/hook-client/src/  # Bounded local hook client
src-tauri/src/app/       # Composition, tray, windows, settings and plugin adapters
src-tauri/src/ui_bridge/ # Tauri commands and events
src/api/                # Typed IPC readers and writers
src/hooks/              # UI state and frame mechanics
src/components/         # Views and interaction tests
src/lib/                # Movement, display and timer-picker helpers
src/types/generated/    # Generated contracts, never manually edited
```

**Structure Decision**: Keep the existing layout. This phase updates only the five existing planning documents. Application code, unrelated dirty files, and the clarified specification remain unchanged.

## Startup Correction Slice

**Completion promise:** Startup displays the person's saved work, creates no example Focus or Task, and starts empty when no saved work exists (FR-001 / FR-019, US1/AC4–AC5). Existing user data remains compatible.

- **Source scope:** remove the sole `seed::ensure_example_focus` call and `pub mod seed` declaration in `src-tauri/src/app/mod.rs`; delete `src-tauri/src/app/seed.rs`, including its obsolete tests that require seeding. Update the startup description in `docs/architecture.md`. No replacement service, no-op function, seed flag, cleanup job, or new test harness.
- **Reuse:** keep `MarkdownFocusStore`, `load_settings`, IPC readers, render projection, directory setup, and `timer_expiry::spawn` unchanged. Persisted `started_at` and `duration_secs` remain authoritative; a Timer that became due while the app was closed may become Expired through the existing worker, without being restarted.
- **Data preservation:** do not identify/delete old Welcome Focuses or read/write/remove `.example-focus-seeded`. Storage already ignores that non-directory file. No schema migration or data rewrite is needed; ordinary saved examples remain editable work. A rollback to an older binary remains data-compatible but may restore its old seeding behavior on an empty unmarked root.
- **Proof:** V0 covers empty roots with/without a legacy marker, populated work including an old example, saved Settings, and Running/Expired/due-while-closed Timers. Reuse existing empty-store/read/sidecar/Timer tests for their actual boundaries; they are not proof of native startup. The deleted seed tests encode behavior the owner rejected; their removal must be paired with the V0 acceptance record, not a weakened gate or a claim that fewer tests prove the feature. Run a fresh `rtk task check` and `rtk task check:windows` on synchronized corrected source; retain unavailable native results as open.
- **Delivery boundary:** this correction is independently reviewable and does not fix click-through, alter Agent integration, or establish full 003 acceptance. Native testing remains deferred until the owner resumes it; no source edit or app launch occurs during this plan command.

## Validation Slices

1. **Files and Timers:** verify Task edits, duplication with fresh identity/creation time and copied ordered Task text/completion but no Timers, Timer ownership/expiry/revival, optional sidecar failures, and persistence using existing tests plus card/file observations. Promise: accurately describe user work without making positional drift a rule.
2. **Ambient shell:** verify selection, interactive Animals/open Focus card with empty-space click-through and outside/Escape closure, Focus Timer absent/Running → walking and Expired → resting independently of Task Timers, displays, Gather, tray, Settings, and soft Limits. Promise: visible behavior and platform-dependent gaps have explicit evidence or an unverified result.
3. **Agent boundary:** verify opt-in visibility, non-editability, exclusion from Limits, plugin health, unchanged Firings, and isolated delivery. Promise: configuration and actual delivery are reported separately.
4. **Evidence reconciliation:** map all FRs/SCs through [the contract](contracts/current-behavior.md) and [quickstart](quickstart.md), record commands/results/platform/commit, measure SC-003 under its precise conditions, track implementation and proof of the selected no-seeding correction, and route discrepancies to their owning follow-on. Promise: no acceptance claim rests only on source inspection or old CI.

The [2026-10-17 pass record](validation/pass-record.md) and v0.1.5 package proof remain historical evidence: T029 is complete, T023–T028 are open, and all six SCs remain unverified end to end. Those results predate the refined startup policy and cannot establish its 19-scenario acceptance set. A subsequent runtime change invalidates reuse of the old gate/package results for that new source; fresh relevant checks and package identity are required for a new shipping claim. Native/live testing remains deferred by the owner; this planning command runs no app, gate, PR, merge, or release.

The source-confirmed open-card click-through mismatch is not fixed by this documentation pass and must not be closed by source inspection or old UI tests. A discovered mismatch is evidence to report, not authorization for unrelated fixes. New acceptance gaps need a narrow issue. The startup correction above is the only runtime work planned here. FR-013–FR-015 already have follow-on context. Any later runtime slice requires relevant gates after synchronization with current main, independent review, green CI, and merge before shipping completion is claimed.

## Complexity Tracking

No new abstraction, protocol, schema, dependency, or constitutional exception is introduced. The startup design removes a forbidden writer instead of exempting it; runtime compliance remains unverified until implementation and proof. See [the startup decision](research.md#8-resume-saved-work-without-seeding) and [separate accepted transport drift](research.md#5-record-accepted-decision-drift-without-enshrining-it).
