# Implementation Plan: Spec Kit Docs Pass

**Branch**: `002-speckit-docs-pass` | **Date**: 2026-10-15 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `specs/002-speckit-docs-pass/spec.md`

## Summary

Turn the living product requirements document into one Draft current product specification, and make the issue queue, glossary banner, architecture description, and historical sources agree with it. The pass writes and edits Markdown only. It does not change what the ranch does.

The current specification is a new Spec Kit spec at `specs/003-ranch-today/spec.md`. It describes today's capabilities, limits, and non-goals in the glossary's settled words, with each carried-forward statement citing its source and recording that it was checked against `main`. After that spec exists, `PRD.md` is deleted. Spec Kit is the only planning document that remains. `specs/001-render-domain-seam/` stays the record of that shipped work and is not rewritten. `CONTEXT.md`, accepted ADRs, `docs/architecture.md`, and the issue queue keep their roles.

A pass record in this feature directory inventories every required source, classifies it, and lists every disagreement that was corrected or left unresolved. Active issues are classified against the completion promise and the product on `main`. Issues that remain to do keep one promise and name the spec that owns it. A follow-on spec is created only when that promise is undone, user-visible, and absent from the current specification.

## Technical Context

**Language/Version**: Markdown documents. No Rust, TypeScript, Tauri, or frontend change.

**Primary Dependencies**: The Spec Kit spec shape already used by `specs/001-render-domain-seam/spec.md` and `.specify/templates/spec-template.md`. The issue file shape in `issues/README.md`. No new tool or package.

**Storage**: Git files only. No Ranch data directory, settings schema, or persisted user state changes.

**Testing**: Reader checks in [quickstart.md](quickstart.md). `task check` once at the end proves the product gate still passes. No new product tests.

**Target Platform**: The repository's documents, read on any platform. The ranch's packaged platforms are unchanged.

**Project Type**: Documentation pass over the existing local-first Tauri desktop application. This feature does not modify the application.

**Performance Goals**: N/A. No runtime path changes.

**Constraints**: Documents only (FR-012, SC-006). Authority order is the constitution, then accepted ADRs, then the current specification, then an issue. Historical sources sit outside that order. Settled glossary words only. `specs/001-render-domain-seam/` is not rewritten. `PRD.md` is deleted only after `specs/003-ranch-today/spec.md` exists and its carry-forward citations are written. Deferred issues stay deferred. Issues closed without being built stay closed. `.specify/feature.json` stays pointed at `specs/002-speckit-docs-pass`. The new product spec stays Draft until the owner approves it. No second planning document.

**Scale/Scope**: One PRD, one glossary, one architecture description, three accepted ADRs, three research notes, one accepted render-domain spec, the issue index, 7 active issue files, 49 finished issue files under `issues/done/`, 2 icebox files, and the closed-without-building notes in the index. Install guide (`README.md`), release history (`CHANGELOG.md`), and working agreements (`CLAUDE.md`) are scanned for contradicting sentences only.

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-checked after Phase 1 design.*

### Pre-design gate

| Principle or constraint | Result | Evidence |
|---|---|---|
| I. User-Owned Local State | PASS | No persistence, network, telemetry, credentials, or integration install path is in scope. |
| II. Independent Domains, Shared Renderer | PASS | No domain or renderer code is edited. The documents must keep Focus and Agent Session independent and must not reintroduce a shared domain type. |
| III. Functional Core, I/O at the Edges | PASS | No modules, traits, or React components are added. |
| IV. Explicit, User-Controlled Integrations | PASS | Harness install, validation, and transport are untouched. |
| V. Observable Completion in Small Slices | PASS | The pass is four document slices, each with one reader-visible promise. `task check` is the regression gate. Review size stays about fifteen minutes per slice. |
| VI. Evidence Is Not Authority | PASS | The pass inventories the PRD before writing the product spec, carries forward only revalidated statements, then deletes the PRD. On 2026-10-15 the owner directed that the old planning document not stay in the tree. Research notes and finished issues stay marked historical. `spec.md` for this feature still says Draft; this command does not edit that status. The owner requested planning by running `/speckit-plan`. |
| Product constraints | PASS | No IPC, localhost API, or network surface. Vocabulary comes from `CONTEXT.md`. Accepted ADRs stay the architecture authority. The simplest design is new Markdown plus banners, not a new documentation system. |
| Work-management order | PASS | The pass uses a feature spec, this plan, later tasks, and the existing issue queue together. It does not replace issues with a task list. |

**Gate result**: PASS. No exception is required.

`specs/002-speckit-docs-pass/spec.md` remains Draft in the file. Planning does not promote it. The product specification this pass writes also stays Draft until the owner approves that file. Until that approval, Principle VI says the Draft product spec is not yet the governing requirements authority. The pass still writes it, marks the PRD historical, and points readers at it as the current description, which is what FR-002 and US1 require on paper.

## Project Structure

### Documentation (this feature)

```text
specs/002-speckit-docs-pass/
├── spec.md                          # this feature's intent; not edited by the plan
├── plan.md                          # this file
├── research.md                      # Phase 0
├── data-model.md                    # Phase 1
├── quickstart.md                    # Phase 1
├── contracts/
│   ├── current-product-spec.md      # shape of specs/003-ranch-today/spec.md
│   ├── pass-record.md               # inventory, classification, unresolved list
│   └── source-marks.md              # historical banners and issue citations
├── checklists/
│   └── requirements.md              # already present
└── pass-record.md                   # written during implementation, not by this command

specs/003-ranch-today/
└── spec.md                          # Draft current product specification; implementation

specs/001-render-domain-seam/        # shipped record; do not rewrite
```

`tasks.md` is deferred to `/speckit-tasks`.

### Documents the pass may edit

```text
PRD.md                               # deleted after specs/003-ranch-today/spec.md exists
CONTEXT.md                           # review banner only; definitions stay
docs/architecture.md                 # contradicting sentences only
docs/research/*.md                   # historical banner on each note
issues/README.md                     # index matches classification
issues/done/README.md                # new archive mark
issues/031-*.md                      # classify; cite an owning spec if still to do
issues/051-*.md
issues/056-*.md
issues/059-*.md
issues/060-*.md
issues/061-*.md
issues/062-*.md
issues/done/<moved>.md               # only files moved here, plus a shipped or closed note
issues/icebox/*.md                   # stay deferred; no promotion
README.md                            # a contradicting pointer only
CHANGELOG.md                         # a contradicting sentence only, if one exists
CLAUDE.md                            # a contradicting sentence only, if one exists
```

A follow-on spec, if the gap test in [research.md](research.md) passes for a remaining issue, is a new `specs/<next-number>-<slug>/spec.md` and nothing else in that directory.

### Untouched

Application and host code (`src/`, `src-tauri/`, `crates/`), package and Cargo manifests, `Taskfile*`, generated types, `specs/001-render-domain-seam/**`, accepted ADR bodies, `.specify/feature.json`, and `.specify/memory/constitution.md`.

**Structure Decision**: Keep the conversion inside the existing Spec Kit and issue layout. The product specification gets the next free number, `003`, so a later `/speckit-specify` allocates `004` or higher. This feature's directory holds the pass record and stays the directory named by `.specify/feature.json`. Finished issues stay in `issues/done/`. Deferred issues stay in `issues/icebox/`. No new documentation tree, site, or issue tracker is introduced.

## Design and Delivery Strategy

Implementation order is fixed because later slices read earlier documents.

### 1. Pass record

Write `specs/002-speckit-docs-pass/pass-record.md` in the shape of [contracts/pass-record.md](contracts/pass-record.md). Inventory every source named in FR-001. Classify each one before any other document changes. Do not copy historical material into the record as a requirement.

**Completion promise**: A reader can take any inventoried source and say which class it has.

### 2. Current product specification

Write `specs/003-ranch-today/spec.md` in the shape of [contracts/current-product-spec.md](contracts/current-product-spec.md). Start from current behavior on `main`, accepted ADRs, and settled glossary entries. Carry a PRD statement forward only when it is still true, and record the check. Leave finished phases, removed capabilities, and deferred work out of the requirements.

State the shipped render outcome in glossary words and point at `specs/001-render-domain-seam/spec.md` as the record of that work. Do not restate that spec.

**Completion promise**: A reader who has not seen the repository can list current capabilities, limits, and non-goals from this file alone.

### 3. Issue queue

Classify every issue that was active at the start of the pass, using the rules in [data-model.md](data-model.md). Move already-satisfied files to `issues/done/` with a shipped note. Move superseded issues that did not ship out of the active directory, record them on the index's closed-without-building list, and keep the file findable under `issues/done/` with a closed-without-building note. Leave icebox files in `issues/icebox/`. Leave issues that are already closed without being built on that list.

For each issue that remains to do, replace `## Parent PRD` with one owning-spec citation, as in [contracts/source-marks.md](contracts/source-marks.md). Keep a single completion promise. Create a follow-on spec only when the gap test passes. Update `issues/README.md` so its queues match the files, and so the issue-file shape shows `## Owning specification` instead of `## Parent PRD`.

**Completion promise**: The active queue contains only undone issues, and each one cites exactly one owning specification and one observable promise.

### 4. Marks and settlements

Delete `PRD.md` only after `specs/003-ranch-today/spec.md` and the pass record exist. Point `README.md` at the current specification, not at the deleted file. Add the historical banners from [contracts/source-marks.md](contracts/source-marks.md) to each research note and to `issues/done/README.md`. Correct the glossary banner, or mark unsettled entries, per the edge case in the feature spec. Where an inventoried document disagrees with a higher authority about current behavior, correct the lower document or add both statements to the pass record's unresolved list. Scan `CHANGELOG.md` and `CLAUDE.md` for contradicting sentences only.

**Completion promise**: The working tree has no `PRD.md`. Finished write-ups and research notes are findable and marked historical. Every disagreement found is either corrected or on the unresolved list.

Do not edit `specs/001-render-domain-seam/**`. Do not edit application code.


## Protected Seams

- The ranch's behavior, persistence, IPC, and integrations stay as they are.
- `CONTEXT.md` remains the glossary. The product spec uses it and does not absorb it.
- Accepted ADRs remain the decision records. The pass does not rewrite an accepted decision. A status correction that does not change the decision may be made in place only when the record's status is wrong; none is expected.
- `specs/001-render-domain-seam/spec.md` remains the render-domain record.
- The issue queue remains the shipping unit. This pass does not emit `tasks.md` entries as a replacement queue.

## Complexity Tracking

No constitution violations require justification.

### Post-design gate

Re-checked after [research.md](research.md), [data-model.md](data-model.md), [quickstart.md](quickstart.md), and `contracts/`.

| Principle or constraint | Result | Evidence |
|---|---|---|
| I. User-Owned Local State | PASS | Design artifacts name Markdown paths only. |
| II. Independent Domains, Shared Renderer | PASS | The product-spec contract forbids a second render-domain specification and forbids retired shared-domain wording. |
| III. Functional Core, I/O at the Edges | PASS | No code design is introduced. |
| IV. Explicit, User-Controlled Integrations | PASS | No integration contract is added. |
| V. Observable Completion in Small Slices | PASS | Four slices, each with one promise. Quickstart maps those promises to reader checks and a final `task check`. |
| VI. Evidence Is Not Authority | PASS | Research orders authorities and defines the carry-forward rule. The PRD is deleted only after that carry-forward. The pass record separates observation, accepted decision, present direction, unresolved assumption, and historical context. The product spec stays Draft. |
| Product constraints | PASS | Glossary contract binds the spec to settled names. Number `003` reuses the existing spec directory convention. |
| Work-management order | PASS | Follow-on specs are conditional and do not include plans or task lists. `.specify/feature.json` is left on this feature. |

**Gate result**: PASS.
