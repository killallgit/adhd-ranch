# Data Model: Spec Kit Docs Pass

These are document entities. Nothing here is persisted in the Ranch data directory or sent over IPC.

Authority, highest first: the constitution (`.specify/memory/constitution.md`), an Accepted ADR (`docs/adr/`), the current product specification, that specification's plan and tasks, then an issue. Historical sources are outside the order. A Draft specification is not approved authority under the constitution; this pass still writes it as the current description and records that status.

## Entity: Current product specification

| Field | Required | Rule |
|---|---|---|
| Path | yes | `specs/003-ranch-today/spec.md` |
| Status | yes | `Draft` until the owner changes it |
| Body | yes | Spec Kit spec shape in [contracts/current-product-spec.md](contracts/current-product-spec.md) |
| Vocabulary | yes | Settled entries from `CONTEXT.md` only |
| Carry-forward | yes | Each inherited statement cites a source and records the check against `main` |

Relationships: cites the glossary and accepted ADRs. Points at `specs/001-render-domain-seam/spec.md` for the shipped render-domain record. Does not embed that spec, the glossary, or an ADR. Does not list deferred work or removed capabilities as requirements.

## Entity: Follow-on specification

| Field | Required | Rule |
|---|---|---|
| Path | yes | `specs/<next-free-number>-<slug>/spec.md` |
| Status | yes | `Draft` |
| Body | yes | Spec Kit spec shape: user scenarios, requirements, success criteria |
| Plan and tasks | no | Must not exist as part of this pass |

Exists only when the gap test in [research.md](research.md) passes for one still-to-do issue. One follow-on may own more than one issue only when those issues are the same user-visible outcome. It does not replace the issue.

## Entity: Issue

| Field | Required | Rule |
|---|---|---|
| Id | yes | Existing numeric id. Do not renumber |
| File | yes | Stays Markdown |
| Completion promise | yes | Exactly one, when the issue remains to do |
| Owning specification | yes | Exactly one path, when the issue remains to do |
| Class | yes | One of the classifications below |

### Classification

| Class | Queue after the pass | File |
|---|---|---|
| still to do | Active index: priority queue or architecture queue | Stays in `issues/` |
| already satisfied | Not active. Reader can see that it shipped | `issues/done/`, with the shipped note |
| superseded | Closed-without-building list, unless the outcome is true, in which case the class is already satisfied | `issues/done/`, with the closed-without-building note |
| deferred | Icebox section | Stays in `issues/icebox/` |

An issue that was already closed without being built stays on that list and is not copied into the product specification.

State transitions during the pass are one-way: active → still to do, already satisfied, or superseded. Deferred does not become active. Closed-without-building does not become a requirement.

## Entity: Domain glossary

| Field | Required | Rule |
|---|---|---|
| Path | yes | `CONTEXT.md` |
| Role | yes | Unchanged. Names live here |
| Banner | yes | Settled banner, or unsettled entries marked and the governing banner used |

The product specification must not become a second glossary.

## Entity: Decision record

| Field | Required | Rule |
|---|---|---|
| Path | yes | `docs/adr/0001-two-apps-over-one-renderer.md`, `docs/adr/0002-agent-hook-transport.md`, `docs/adr/0003-rust-owned-render-scene.md` |
| Status | yes | Accepted. Only Accepted records govern |
| Body | yes | Unchanged by this pass unless a non-decision status correction is required |

Draft, rejected, and superseded records would be context. None of those statuses are in `docs/adr/` at planning time.

## Entity: Historical source

| Field | Required | Rule |
|---|---|---|
| Kind | yes | Finished issue write-up, or research note |
| Path | yes | Original path, still findable |
| Mark | yes | Banner in [contracts/source-marks.md](contracts/source-marks.md) |

`PRD.md` is inventoried, then deleted. It is not a historical source that remains. Finished write-ups are the files under `issues/done/` plus any file moved there because it shipped. Research notes are the three files in `docs/research/`.

## Entity: Pass record

| Field | Required | Rule |
|---|---|---|
| Path | yes | `specs/002-speckit-docs-pass/pass-record.md` |
| Inventory row | yes | One row per FR-001 source |
| Class | yes | Accepted decision, verified observation, present direction, unresolved assumption, or historical context |
| Unresolved entry | when needed | Both statements, both paths, and why the check could not pick one |

The record is evidence of the pass. It is not an accepted ADR and not a second product specification.

### Source class

| Class | Meaning |
|---|---|
| Accepted decision | An Accepted ADR, or a constitution rule |
| Verified observation | Behavior checked on `main` |
| Present direction | What the owner still wants done, including a still-to-do issue |
| Unresolved assumption | A statement that could not be checked |
| Historical context | A finished phase, a finished issue, a research note, or the PRD after it is marked |

A row may name a document that contains more than one class. The row says so, and the product specification receives only the verified and accepted parts.

## Entity: Unresolved disagreement

| Field | Required | Rule |
|---|---|---|
| Higher statement | yes | Quote or close paraphrase, with path |
| Lower statement | yes | Quote or close paraphrase, with path |
| Why unresolved | yes | What was checked on `main` and why it did not decide |

Absent from the current specification, the glossary, accepted ADRs, and `docs/architecture.md` means the pass failed SC-004. A corrected lower document does not also need an unresolved entry.
