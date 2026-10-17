# Implementation Plan: Ranch Today Baseline Validation

**Feature context**: `003-ranch-today` | **Date**: 2026-10-17 | **Spec**: [spec.md](spec.md)

**Checkout branch**: `main` at `a48525d9a89555bf9bc925a20d626c0dc634b4ce`

**Status**: Draft. The specification remains Draft; these artifacts support owner review and do not promote it to approved authority.

**Input**: `specs/003-ranch-today/spec.md`, including the 2026-10-17 clarification that known limitations are observations which follow-on specifications may fix.

## Summary

Establish a reviewable baseline for the existing local-first desktop product: Focus and Task editing, Timers, ambient Animals, tray and Settings, hand-edited Markdown, and optional Claude Code observations. Reuse existing code and tests, document current contracts, and prepare a repeatable validation pass. No runtime implementation, dependencies, storage migration, new IPC, or archive work is proposed.

FR-013–FR-015 are observations, not behavior to preserve. ADR-0002's accepted transport direction is not fully implemented in current code; [research](research.md) records that discrepancy without approving it or absorbing its follow-on work into 003.

## Technical Context

**Language/Version**: Rust edition 2021, declared minimum 1.77; TypeScript ^5.6.3. Existing manifests and lockfiles govern installed versions.

**Primary Dependencies**: Tauri v2 (^2.1 API/CLI declarations), React ^18.3.1, Vite ^5.4.11, serde/serde_json, ts-rs-generated frontend types. No dependency changes.

**Storage**: Focus directories with `focus.md`, optional `timer.json` and `task-timers.json`; `settings.yaml` under the Ranch data root. Agent Sessions and recent Hook Firings are runtime observations, not user work. See [data model](data-model.md).

**Testing**: Existing Rust unit/integration tests, Vitest ^2.1.8, generated-type drift checks, native manual scenarios. [Quickstart](quickstart.md) distinguishes available coverage from checks actually executed.

**Target Platform**: macOS primary; retain Windows and Linux package compatibility. Windows has no hook delivery. No platform code changes.

**Project Type**: Tauri desktop application with Rust workspace and React webviews; Tauri IPC is the frontend/core boundary.

**Performance Goals**: Hand-edited Markdown visible within about one second (SC-003); ambient rendering without unsolicited attention. No new FPS or throughput target.

**Constraints**: User-controlled atomic local writes; safe malformed optional sidecars; independent Focus and Agent Session domains; no outbound product network, telemetry, model calls, or stored provider credentials. Integration install remains explicit.

**Scale/Scope**: One local user; one Animal per Focus plus optionally observed Claude Code sessions. Soft defaults 5 Focuses / 7 Tasks per Focus are thresholds, not capacity ceilings. Five user stories, 22 FR entries including three observations, six success criteria.

## Constitution Check

These gates apply to this documentation and validation design, not a certification of all existing runtime behavior.

| Principle / gate | Before research | After design |
|---|---|---|
| I — User-owned local state | PASS: inspect existing files; no migration or personal-data writes. | PASS: synthetic validation data; no new write path or network service. |
| II — Independent domains | PASS: preserve the accepted renderer seam. | PASS: Agent activity remains separate from Focus Timers, editing, and Limits. |
| III — Functional core, typed edges | PASS: reuse existing layers and generated contracts. | PASS: source owners linked; no code or generated-type edits. |
| IV — Explicit integrations | PASS: no installation during planning. | PASS: isolated delivery proof; optional live check; configuration health is not delivery. |
| V — Observable small slices | PASS: baseline evidence only. | PASS: four bounded validation slices below; no passing-test or shipped-completion claim. |
| VI — Evidence and authority | PASS: keep the spec Draft and classify sources. | PASS: distinguish accepted ADR direction, observed code, and draft follow-ons; document existing transport drift. |

**Gate result:** PASS for Phase 0 and Phase 1 planning. No new exception is requested. ADR-0002 drift is an existing gap, not a compliant runtime result. Follow-on work must reconcile lower-level artifacts with accepted decisions before implementation. Owner approval of the current specification remains necessary before treating it as governing an implementation.

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

`tasks.md` is a later `$speckit-tasks` output, not created by this command.

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

**Structure Decision**: Keep the existing layout. This phase authors only five planning documents. Application code, unrelated dirty files, and the clarified specification remain unchanged.

## Validation Slices

1. **Files and Timers:** verify Task edits, Timer ownership/expiry/revival, optional sidecar failures, and persistence using existing tests plus card/file observations. Promise: accurately describe user work without making positional drift a rule.
2. **Ambient shell:** verify selection, movement, displays, Gather, tray, Settings, and soft Limits. Promise: visible behavior and platform-dependent gaps have explicit evidence or an unverified result.
3. **Agent boundary:** verify opt-in visibility, non-editability, exclusion from Limits, plugin health, unchanged Firings, and isolated delivery. Promise: configuration and actual delivery are reported separately.
4. **Evidence reconciliation:** map all FRs/SCs through [the contract](contracts/current-behavior.md) and [quickstart](quickstart.md), record commands/results/platform/commit, and route discrepancies to their owning follow-on. Promise: no acceptance claim rests only on source inspection or old CI.

A discovered mismatch is evidence to report, not authorization for unrelated fixes. New acceptance gaps need a narrow issue. FR-013–FR-015 already have follow-on context. Any later runtime slice requires relevant gates after synchronization with current main, independent review, green CI, and merge before shipping completion is claimed.

## Complexity Tracking

No new abstraction, protocol, schema, dependency, or constitutional exception. Existing accepted-decision drift is recorded in [research](research.md#5-record-accepted-decision-drift-without-enshrining-it).
