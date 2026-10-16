# Research: Spec Kit Docs Pass

All technical unknowns for this documentation pass are resolved. No product capability was chosen here; the decisions are about where documents live and how a disagreement is recorded.

## Decision: The current product specification is `specs/003-ranch-today/spec.md`

**Rationale**: The feature asks for one current specification in the format the project already uses, which is a Spec Kit `spec.md`. Number `003` is the next free prefix after `specs/001-render-domain-seam/` and `specs/002-speckit-docs-pass/`. `create-new-feature.sh` allocates the next number from existing directory prefixes, so a later specify command will use `004` or higher. The file is written by this pass. It is not created by `/speckit-specify`, because that command would retarget `.specify/feature.json` and draft from a new prompt.

The spec stays **Draft**. The constitution says a feature specification remains Draft until the owner approves it. This pass must not flip that status.

**Alternatives considered**:

- Rewriting `PRD.md` in place. Rejected. The owner wants Spec Kit as the only planning document. A rewritten PRD would still be a second plan.
- Keeping `PRD.md` with a historical banner. Rejected on 2026-10-15. The owner directed that the old planning document be removed.
- Putting the product spec inside `specs/002-speckit-docs-pass/`. Rejected. That directory is the conversion feature, not the product description a later slice should cite.

## Decision: This feature's directory keeps the pass record

**Rationale**: FR-001 and the pass-record entity need an inventory, a classification, and a visible unresolved list. `specs/002-speckit-docs-pass/pass-record.md` is that list. `.specify/feature.json` already points at this directory and must keep doing so, or later Spec Kit commands for this feature will look at the product spec instead.

The unresolved list is visible because the historical banners and the corrected `README.md` pointer name the pass record. A disagreement is not settled by omitting it. The pass record also notes that `PRD.md` was removed after the current specification was written.

**Alternatives considered**:

- A new `docs/decisions/` index. Rejected. Accepted decisions already live in `docs/adr/`. A second index would look like a new authority.
- Leaving the inventory inside `PRD.md`. Rejected. That file is deleted.

## Decision: Delete `PRD.md` after the current specification exists

**Rationale**: The owner directed, on 2026-10-15, that the old planning document come out and that Spec Kit be the planning format that remains. FR-009 requires the deletion. The file is read during the inventory and the carry-forward. It is deleted only after `specs/003-ranch-today/spec.md` and the pass record exist, so the citations have somewhere to point in the working tree until that moment. Git history keeps the old text. The working tree does not.

`README.md` currently says to see `PRD.md` for the design. That sentence points at `specs/003-ranch-today/spec.md` instead. It does not name a file that was just deleted as a place to keep reading.

Active and deferred issues stop citing `PRD.md`. Finished issues are not rewritten to chase the deleted path. A `## Parent PRD` line in `issues/done/` is historical prose.

The glossary, accepted ADRs, `docs/architecture.md`, research notes, and the issue queue stay. They are not the old plan. The constitution still requires the glossary, accepted decisions, and the issue queue beside Spec Kit.

**Alternatives considered**:

- Deleting `PRD.md` before the current specification is written. Rejected. The conversion would lose its source.
- Also deleting `CONTEXT.md`, `docs/adr/`, `docs/architecture.md`, `docs/research/`, or `issues/`. Rejected. Those are names, decisions, the implementation description, evidence, and the shipping queue. The owner asked to remove the old planning document, not those authorities.

## Decision: Mark the remaining historical sources with a banner, not a move

**Rationale**: Finished issue write-ups and research notes stay findable and must be identifiable as historical. Moving them would break links. Editing all 49 finished issue bodies would rewrite history the feature says to keep.

The mark is one banner:

- On each of the three files in `docs/research/`.
- Once for the finished archive, in a new `issues/done/README.md`, so the archive is marked as a class.

Files moved into `issues/done/` during this pass also get a one-line shipped note or closed-without-building note, so a reader can tell those apart from work that shipped as written.

`specs/001-render-domain-seam/` is the accepted record of shipped render-domain work (FR-011). It gets no banner and no edit.

**Alternatives considered**:

- A banner on every file in `issues/done/`. Rejected. The archive is one class, and a directory mark keeps the old write-ups intact.
- Marking `specs/001-render-domain-seam/spec.md` historical. Rejected. That would rewrite the record FR-011 says to leave in place.

## Decision: Classify issues by the observable promise, not the original checklist

**Rationale**: US2 and the edge case about a shipped issue whose checklist names files that never landed. An active issue is judged by whether its completion promise is true on `main`, compared with the code, the accepted ADRs, and `docs/architecture.md`. Ticking the original checklist is not the test.

Classes:

- **still to do** — the promise is not true, the owner has not deferred it, and later work has not replaced it. The file stays in `issues/` and cites one owning spec.
- **already satisfied** — the promise is true on `main`, including when a different approach landed. Move the file to `issues/done/` and add the shipped note.
- **superseded** — later work replaced the issue or its premise was wrong, and the original promise is not the work to do. If the outcome is nevertheless true, classify it as already satisfied instead. Otherwise add it to the index's closed-without-building list and move the file to `issues/done/` with the closed-without-building note.
- **deferred** — the owner has deferred it. The two icebox files stay in `issues/icebox/` and stay off the current specification's requirements.

The index's priority queue and architecture queue are both active. Both may contain only still-to-do issues after the pass. Do not collapse those two subsections.

Known input the implementation must classify, not a result decided here: `issues/README.md` lists 046 as closed without implementation, and `issues/done/046-timer-expiry-service-extraction.md` also exists. The pass record must show which statement was checked against `main`, and the lower-authority line must be corrected or both statements must appear on the unresolved list.

Active files at planning time: `031`, `051`, `056`, `059`, `060`, `061`, `062`. Icebox: `021`, `022`. Closed-without-building notes already in the index: `042`, `045`, `046`, `047`. Those notes stay closed unless a check shows the outcome actually shipped, in which case the note is corrected so a reader can see that it shipped.

**Alternatives considered**:

- Re-checking every acceptance checkbox against the original file list. Rejected. That recreates the stale-plan failure the feature describes.
- A new `issues/closed/` directory. Rejected. The index already has a closed-without-building list, and a new directory would be a second queue.

## Decision: One owning spec, and a follow-on only when the gap is real

**Rationale**: FR-006 and FR-007. A remaining issue stops citing `## Parent PRD` and cites either `specs/003-ranch-today/spec.md` or one follow-on spec. It keeps a single completion promise.

Gap test, applied only after the current specification exists, and only to an issue classified still to do:

1. The promise describes behavior a person using the ranch can observe.
2. The current specification does not already state that outcome, as a capability or as a limit.
3. The promise is not only an internal seam, a rename, or a regression fix of a behavior the current specification already states.

Create a follow-on `spec.md` only when all three are true. The directory number is the next free prefix at the time of creation, never a number reserved in advance other than `003` for the product spec. The follow-on file is Draft, uses the Spec Kit spec shape, and has no `plan.md` or `tasks.md`. Internal slices cite the current specification section they serve, plus an accepted ADR when the issue already cites one.

The pass does not replace the issue queue with `tasks.md`.

**Alternatives considered**:

- One follow-on spec per remaining issue. Rejected by the feature assumptions.
- Leaving `## Parent PRD` in place and adding a second parent. Rejected. The issue would still point at a historical document as its authority.

## Decision: Settled glossary words, banner corrected in place

**Rationale**: FR-003 and FR-008. `CONTEXT.md` stays the glossary. The product specification uses settled entries and does not redefine them. The banner that says the language is still under review is the edge case in the feature spec. Implementation reads each entry. If every entry is settled, replace the banner with the settled banner in [contracts/source-marks.md](contracts/source-marks.md). If an entry is not settled, mark that entry **Unsettled** and use the banner that says settled entries govern.

Retired words are the entry's `_Avoid_` line. The product specification uses the settled word for that idea. **Pig** remains the Species name and the concrete sprite name. It is not the word for the general Animal. **Limit** is the word for the soft threshold the PRD calls a cap. **Gather** is the word for bringing Animals back. **Settings** is the word for stored configuration. **Preferences** remains only the macOS menu label.

**Alternatives considered**:

- Copying the glossary into the product spec and deleting `CONTEXT.md`. Rejected by FR-008.
- Leaving the review banner untouched. Rejected. The feature's edge case says the banner is corrected or the unsettled entries are called out.

## Decision: Correct the lower authority, or write both statements down

**Rationale**: FR-010 and the constitution's authority order. When two inventoried documents disagree about current behavior:

1. Constitution.
2. Accepted ADR (`docs/adr/0001`, `0002`, `0003` are all Accepted).
3. The current product specification, once it states the revalidated behavior. While it is Draft, do not use it to overrule the constitution or an accepted ADR. Do use it as the requirements document the architecture description, README, and issues must stop contradicting.
4. An issue.
5. Historical sources never win: `issues/done/**` and `docs/research/**`. `PRD.md` is read, then deleted, so it cannot remain a competing statement.

The lower document is edited. If the pass cannot tell which statement is true on `main`, both statements go on the unresolved list and neither is silently dropped.

`docs/architecture.md` is the implementation description. It is corrected where it claims a current behavior the constitution, an accepted ADR, or the revalidated product spec does not hold. It is not converted into a specification.

`README.md`, `CHANGELOG.md`, and `CLAUDE.md` are not converted. A sentence that contradicts the current specification is corrected or listed. The expected README settlement is the sentence that sends the reader to `PRD.md` for the design: point that sentence at `specs/003-ranch-today/spec.md`. Confirm the sentence still says that before editing.

**Alternatives considered**:

- Rewriting `docs/architecture.md` to mirror the product spec. Rejected. It describes implementation, and wholesale conversion is out of scope.
- Keeping the PRD until the owner approves the Draft spec. Rejected. The owner asked for it to be removed. The Draft status stays on the product spec and in the pass record so Principle VI stays visible.

## Decision: Proof is a reader check plus an empty product diff

**Rationale**: SC-001 through SC-006 are document outcomes. [quickstart.md](quickstart.md) is the validation guide. `task check` runs once after the document edits. A diff that touches `src/`, `src-tauri/`, `crates/`, manifests, or generated types fails the pass even if the gate is green, because FR-012 forbids a behavior change.

**Alternatives considered**:

- New Vitest or Rust tests for prose. Rejected. There is no code contract to execute.
- Skipping `task check`. Rejected. The constitution requires the gate after synchronization with `main`, and a green gate is the evidence the document pass did not break the build.
