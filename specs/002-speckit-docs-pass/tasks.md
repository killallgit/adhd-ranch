# Tasks: Spec Kit Docs Pass

**Input**: Design documents from `specs/002-speckit-docs-pass/`

**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/, quickstart.md

**Tests**: No product tests. This pass edits Markdown only. Proof is the reader checks in `specs/002-speckit-docs-pass/quickstart.md` and a final `task check`.

**Organization**: Tasks are grouped by user story. The pass record is foundational. User Story 2 waits for the current specification. User Story 3 waits for the queue, because `PRD.md` is deleted only after the spec exists and active issues no longer cite it.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (US1, US2, US3)
- Every task includes an exact file path

## Path Conventions

Documents only, at the repository root:

- `specs/003-ranch-today/spec.md` — Draft current product specification
- `specs/002-speckit-docs-pass/pass-record.md` — inventory, classification, unresolved list
- `PRD.md` — read during the pass, then deleted
- `CONTEXT.md`, `docs/adr/`, `docs/architecture.md`, `docs/research/`, `issues/`, `README.md`
- Do not edit `src/`, `src-tauri/`, `crates/`, manifests, `Taskfile.yaml`, `src/types/generated/`, `specs/001-render-domain-seam/**`, or `.specify/memory/constitution.md`

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Reserve the product-spec directory and keep Spec Kit pointed at this feature

- [x] T001 Confirm `.specify/feature.json` contains `"feature_directory": "specs/002-speckit-docs-pass"` and do not edit that file
- [x] T002 Create the directory `specs/003-ranch-today/` and do not write `specs/003-ranch-today/spec.md` in this task

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Inventory every required source before any requirements text is written or any issue file is moved

**⚠️ CRITICAL**: No user story work can begin until this phase is complete

- [x] T003 Create `specs/002-speckit-docs-pass/pass-record.md` with the sections How to read this, Inventory, Issue classification, Corrected, and Unresolved, per `specs/002-speckit-docs-pass/contracts/pass-record.md`. How to read this states four rules: `PRD.md` is deleted only after `specs/003-ranch-today/spec.md` exists; remaining historical sources are not requirements; the current description is `specs/003-ranch-today/spec.md` and its status is Draft; accepted decisions remain in `docs/adr/` and names remain in `CONTEXT.md`. Unresolved is `None.` until a later task cannot decide a disagreement
- [x] T004 Add one inventory row in `specs/002-speckit-docs-pass/pass-record.md` for each of `PRD.md`, `CONTEXT.md`, `docs/architecture.md`, `docs/adr/0001-two-apps-over-one-renderer.md`, `docs/adr/0002-agent-hook-transport.md`, `docs/adr/0003-rust-owned-render-scene.md`, `docs/research/serena-multi-agent.md`, `docs/research/manual-github-release-retention.md`, `docs/research/claude-codex-session-terminology.md`, and `specs/001-render-domain-seam/spec.md`. Columns are Source, Class, Checked, and Disposition. Class is one or more of: Accepted decision, verified observation, present direction, unresolved assumption, historical context. Disposition is one of: Kept as authority, carried into `specs/003-ranch-today/spec.md`, marked historical, deleted after carry-forward, left deferred, listed as unresolved. Set `PRD.md` to deleted after carry-forward. Set each ADR, all of which are Accepted, to Kept as authority. Set `specs/001-render-domain-seam/spec.md` to Kept as authority and do not edit that file
- [x] T005 Add inventory rows in `specs/002-speckit-docs-pass/pass-record.md` for `issues/README.md`, `issues/031-notification-source-settings.md`, `issues/051-ranch-animal-vocabulary-seam.md`, `issues/056-task-timer-binding.md`, `issues/059-self-describing-hook-frame.md`, `issues/060-session-staleness-sweep.md`, `issues/061-hook-change-events.md`, `issues/062-stable-hook-client-path.md`, `issues/icebox/021-all-monitors-default.md`, `issues/icebox/022-wrangle.md`, one shared row for the 49 files under `issues/done/`, and the closed-without-building notes for 042, 045, 046, and 047. Record that `issues/README.md` lists 046 as closed without implementation while `issues/done/046-timer-expiry-service-extraction.md` exists, and do not choose which statement wins in this task

**Checkpoint**: A reader can take any inventoried source and see its class. No issue file has been moved and `PRD.md` still exists.

---

## Phase 3: User Story 1 - Read the product as it is now (Priority: P1) 🎯 MVP

**Goal**: One Draft specification states today's capabilities, limits, and non-goals in the glossary's settled words

**Independent Test**: Hand a reader who has not seen the repository only `specs/003-ranch-today/spec.md`. They can name current capabilities, limits, and non-goals, and they do not need `PRD.md` to do it.

### Implementation for User Story 1

- [x] T006 [US1] Write `specs/003-ranch-today/spec.md` per `specs/002-speckit-docs-pass/contracts/current-product-spec.md` with status `Draft`, feature branch `003-ranch-today`, and an input line that this specification converts the product requirements document into the current description. Include user scenarios of a person using the ranch today, edge cases that are true today, functional requirements, limits and non-goals, and a sources appendix. Carry a statement forward only when it is still true on `main`
- [x] T007 [US1] Revise `specs/003-ranch-today/spec.md` so it uses only settled names from `CONTEXT.md`: Animal, Species, Motion, Focus, Task, Timer, Limit, Over Limit, Harness, Agent Session, Session Activity, Hook Firing, Pen, Agents as Animals, DisplaySpace, Gather, and Settings. Pig is the Species and the sprite asset name, not the word for every Animal. Limit is the soft threshold. Gather brings Animals back onto the primary display. Settings is the stored configuration. Preferences appears only as the macOS menu label for the window that edits Settings
- [x] T008 [US1] Revise `specs/003-ranch-today/spec.md` so requirements omit implementation phases, a localhost HTTP API, a proposal queue, deferred multi-monitor default-on, and wrangle. State the render outcome in one short capability: Focus and Agent Session each project Animals, the shared renderer does not import either domain, and Pig is the only Species. Name `specs/001-render-domain-seam/spec.md` as the record of that work and do not copy its requirements, `CONTEXT.md`, or any accepted ADR
- [x] T009 [US1] Add a sources appendix entry in `specs/003-ranch-today/spec.md` for every carried-forward requirement, in the form `- <requirement id>: source \`<path>\` — checked against \`main\` on <YYYY-MM-DD>. <one sentence of what was confirmed>.` Cite `docs/adr/` or `.specify/memory/constitution.md` when that is the source. Omit statements that are no longer true. In `specs/002-speckit-docs-pass/pass-record.md`, set Disposition to `carried into specs/003-ranch-today/spec.md` only for rows whose statements were actually carried

**Checkpoint**: User Story 1 is testable from `specs/003-ranch-today/spec.md` alone. `PRD.md` is still present until User Story 3.

---

## Phase 4: User Story 2 - Trust the work still to do (Priority: P2)

**Goal**: The active queue contains only undone issues, each with one promise and one owning specification

**Independent Test**: Compare `issues/README.md` and the files still in `issues/` with the product on `main`. Every active item is still undone, names one observable outcome, and cites one owning specification. Deferred items stay in `issues/icebox/`. Issues closed without being built stay closed.

### Implementation for User Story 2

- [x] T010 [US2] Add one classification row in `specs/002-speckit-docs-pass/pass-record.md` for issues 031, 051, 056, 059, 060, 061, 062, 021, 022, 042, 045, 046, and 047. Columns are Issue, Class, Evidence, and Owning specification. Class is exactly one of: still to do, already satisfied, superseded, deferred. Judge the completion promise against `main`, not the original checklist. still to do means the promise is not true, the owner has not deferred it, and later work has not replaced it. already satisfied means the promise is true on `main`, including when a different approach landed. superseded means later work replaced the issue or its premise was wrong; if the outcome is nevertheless true, classify it already satisfied instead. deferred means the owner has deferred it. If the 046 note in `issues/README.md` and `issues/done/046-timer-expiry-service-extraction.md` still disagree after that check, correct the lower-authority line or add an unresolved entry with both statements
- [x] T011 [US2] Move each issue classified already satisfied from `issues/` to `issues/done/` and add this note at the top of that file: `> **Shipped.** The completion promise is true on \`main\`. <One sentence naming the landed approach when it differs from this write-up; otherwise "The write-up matches what landed.">` If none are already satisfied, write that in `specs/002-speckit-docs-pass/pass-record.md` and move nothing
- [x] T012 [US2] Move each issue classified superseded, whose outcome did not ship, from `issues/` to `issues/done/`, add it to the closed-without-building list in `issues/README.md`, and add this note at the top of that file: `> **Closed without building.** <One sentence naming the replacement or the wrong premise.> This file is not a current requirement.` Leave 042, 045, and 047 closed. Classify 046 only by the T010 rule: when its promise is true on `main`, it is already satisfied under T011 and leaves the closed-without-building list. If none of the active files are superseded, write that in `specs/002-speckit-docs-pass/pass-record.md` and move nothing
- [x] T013 [P] [US2] In `issues/icebox/021-all-monitors-default.md` and `issues/icebox/022-wrangle.md`, replace the whole `## Parent PRD` section with `## Owning specification` followed by `None. Deferred. Not a requirement of \`specs/003-ranch-today/spec.md\`.` Leave both files in `issues/icebox/`. Do not add either outcome to `specs/003-ranch-today/spec.md`
- [x] T014 [US2] For each issue that remains in `issues/` outside `issues/done/` and `issues/icebox/`, replace the whole `## Parent PRD` section in that issue file with `## Owning specification` and exactly one path, and keep exactly one `## Completion promise`. Use `specs/003-ranch-today/spec.md` plus a requirement id or section name, unless all three gap-test conditions are true: the promise describes behavior a person using the ranch can observe; `specs/003-ranch-today/spec.md` does not already state that outcome as a capability or a limit; the promise is not only an internal seam, a rename, or a regression fix of behavior that specification already states. When all three are true, create only `specs/<next-free-number>-<slug>/spec.md` with status Draft, user scenarios, requirements, and success criteria, and no `plan.md` or `tasks.md`, and cite that path
- [x] T015 [US2] Update `issues/README.md` so the priority queue and the architecture queue contain only issues classified still to do, the icebox section still lists `issues/icebox/021-all-monitors-default.md` and `issues/icebox/022-wrangle.md`, the closed-without-building notes match the classification, and the documented issue file shape shows `## Owning specification` instead of `## Parent PRD`

**Checkpoint**: The active queue is only undone work. User Story 1's specification is unchanged except that no deferred outcome was added to it.

---

## Phase 5: User Story 3 - Tell history from authority (Priority: P3)

**Goal**: `PRD.md` is gone, remaining historical sources are marked, and every disagreement is corrected or listed

**Independent Test**: Take each source in `specs/002-speckit-docs-pass/pass-record.md`. A reader can say which role it has. `PRD.md` is absent. Every disagreement with a higher authority is either corrected in the lower document or present as an unresolved entry with both statements.

### Implementation for User Story 3

- [x] T016 [US3] Delete `PRD.md` after `specs/003-ranch-today/spec.md` and `specs/002-speckit-docs-pass/pass-record.md` both exist. Do not leave a banner or a stub. In `specs/002-speckit-docs-pass/pass-record.md`, state that `PRD.md` was removed and keep its disposition as deleted after carry-forward
- [x] T017 [P] [US3] In `README.md`, replace the sentence that says to see `PRD.md` for the design so it points at `specs/003-ranch-today/spec.md`. Do not tell a reader to open `PRD.md`
- [x] T018 [P] [US3] Insert this banner immediately under the title of `docs/research/serena-multi-agent.md`: `> **Historical.** This note is evidence. It is not a current requirement and not an accepted decision. The constitution, accepted ADRs, and the current product specification (after owner approval) govern where they disagree with it. Draft specifications are context only. See [specs/002-speckit-docs-pass/pass-record.md](../../specs/002-speckit-docs-pass/pass-record.md).`
- [x] T019 [P] [US3] Insert the same historical banner immediately under the title of `docs/research/manual-github-release-retention.md`
- [x] T020 [P] [US3] Insert the same historical banner immediately under the title of `docs/research/claude-codex-session-terminology.md`
- [x] T021 [P] [US3] Create `issues/done/README.md` with the heading `# Finished issues` and the banner `> **Historical.** These write-ups record what was asked at the time. They are not current requirements. A file marked **Shipped** had its outcome become true, including when the landed approach differs from the write-up. A file marked **Closed without building** was not built. The active queue is [../README.md](../README.md).` Do not add that banner to each existing file under `issues/done/`
- [x] T022 [P] [US3] In `CONTEXT.md`, read every entry and do not rewrite a settled definition. If every entry is settled, replace the under-review blockquote with `> Settled names for the ranch. Specifications, issues, and UI copy use these words. The current product specification does not redefine them.` If any entry is not settled, mark that entry with `**Unsettled**` on its own line under the term and replace the banner with `> Settled entries govern. Entries marked **Unsettled** do not. The current product specification uses only settled names.`
- [x] T023 [US3] Compare `docs/architecture.md` with `.specify/memory/constitution.md`, `docs/adr/0001-two-apps-over-one-renderer.md`, `docs/adr/0002-agent-hook-transport.md`, `docs/adr/0003-rust-owned-render-scene.md`, and `specs/003-ranch-today/spec.md`. Correct the lower-authority sentence in `docs/architecture.md`, or add an unresolved entry to `specs/002-speckit-docs-pass/pass-record.md` as `### U-<n> <short title>` with bullets `Higher: \`<path>\` — <statement>`, `Lower: \`<path>\` — <statement>`, and `Check: <what was looked at on main, and why it did not decide>`. Do not convert `docs/architecture.md` into a specification. Do not rewrite the three ADR bodies
- [x] T024 [US3] Read `CHANGELOG.md` and `CLAUDE.md` for a sentence that contradicts `specs/003-ranch-today/spec.md`. Correct that sentence in the file that contains it, or record it on the Corrected or Unresolved list in `specs/002-speckit-docs-pass/pass-record.md`. Do not rewrite either file wholesale. A corrected disagreement is a Corrected list item: path, one sentence, higher authority cited

**Checkpoint**: `PRD.md` is absent. Finished write-ups and research notes are findable and marked historical. The glossary and accepted ADRs still hold their roles.

---

## Phase 6: Polish & Cross-Cutting Concerns

**Purpose**: Prove the pass changed documents only and that the three reader checks hold together

- [x] T025 Walk sections 1 through 3 of `specs/002-speckit-docs-pass/quickstart.md` and fix any miss in the file that check names
- [x] T026 From the repository root, run `git diff --stat -- src src-tauri crates package.json package-lock.json Cargo.toml Cargo.lock Taskfile.yaml src/types/generated` and confirm the output is empty. Do not edit those paths
- [x] T027 From the repository root, run `task check` as defined in `Taskfile.yaml` and confirm the gate passes
- [x] T028 Confirm `.specify/feature.json` still contains `"feature_directory": "specs/002-speckit-docs-pass"`

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies
- **Foundational (Phase 2)**: Depends on Setup. Blocks every user story
- **User Story 1 (Phase 3)**: Depends on Foundational. Does not wait for User Story 2 or 3
- **User Story 2 (Phase 4)**: Depends on User Story 1, because an owning-spec citation and the gap test require `specs/003-ranch-today/spec.md`
- **User Story 3 (Phase 5)**: Depends on User Story 2, because active and deferred issues must stop citing `PRD.md` before that file is deleted
- **Polish (Phase 6)**: Depends on User Story 3

### User Story Dependencies

- **User Story 1 (P1)**: Starts after the pass record inventory. No dependency on the queue or on deleting `PRD.md`
- **User Story 2 (P2)**: Starts after User Story 1. Does not delete `PRD.md` and does not rewrite `specs/001-render-domain-seam/`
- **User Story 3 (P3)**: Starts after User Story 2. Deletes `PRD.md` and marks the sources that remain

### Within Each User Story

- User Story 1 edits `specs/003-ranch-today/spec.md` in order: draft, vocabulary, boundaries, citations
- User Story 2 classifies in the pass record before moving files, then updates `issues/README.md` last
- User Story 3 deletes `PRD.md` and settles `docs/architecture.md` in the pass record before the polish read. Research banners, the archive mark, the glossary banner, and the README pointer touch different files and can run beside the deletion

### Parallel Opportunities

- T001 and T002 touch different paths and can run together
- T013 can run beside T011 and T012. It edits only the two icebox files
- T017, T018, T019, T020, T021, and T022 can run together. They edit different files
- User stories themselves are sequential

---

## Parallel Example: User Story 3

```text
Task: "T017 Point README.md at specs/003-ranch-today/spec.md"
Task: "T018 Banner on docs/research/serena-multi-agent.md"
Task: "T019 Banner on docs/research/manual-github-release-retention.md"
Task: "T020 Banner on docs/research/claude-codex-session-terminology.md"
Task: "T021 Create issues/done/README.md"
Task: "T022 Correct the banner in CONTEXT.md"
```

T016, T023, and T024 stay in sequence because they share `specs/002-speckit-docs-pass/pass-record.md`.

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1 and Phase 2
2. Complete Phase 3
3. Stop and hand a reader only `specs/003-ranch-today/spec.md`
4. Continue only after that reader can name capabilities, limits, and non-goals

### Incremental Delivery

1. Setup and the pass record produce the inventory without changing the product description
2. User Story 1 adds the current specification. `PRD.md` is still the old document beside it
3. User Story 2 makes the active queue match work that is still undone
4. User Story 3 deletes `PRD.md` and marks what remains, so Spec Kit is the only planning document

### Sequential Story Strategy

One implementer should finish the stories in order P1, then P2, then P3. The parallel work is inside User Story 3, on files that do not share the pass record.

---

## Notes

- [P] tasks use different files and do not wait on each other
- [US1], [US2], and [US3] map to the three stories in `specs/002-speckit-docs-pass/spec.md`
- Do not edit application code. A green `task check` does not excuse a diff under `src/`, `src-tauri/`, or `crates/`
- Do not edit `specs/001-render-domain-seam/**`
- Leave `specs/003-ranch-today/spec.md` at status Draft
- Commit after each phase checkpoint


## Phase 7: Convergence

- [X] T029 Correct the remaining user-guide terminology in `README.md`: use Complete/Delete for Task actions, reserve Clear for Timers, and use Limit/Over Limit in prose while preserving literal settings keys and actual menu labels. Record the settlement against `CONTEXT.md` and `specs/003-ranch-today/spec.md` in `specs/002-speckit-docs-pass/pass-record.md` per FR-010 and US3/AC3 (partial).
