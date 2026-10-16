# Feature Specification: Spec Kit Docs Pass

**Feature Branch**: `002-speckit-docs-pass`

**Created**: 2026-10-15

**Status**: Draft

**Input**: User description: "converting our PRD, issues and docs into the speckit format and doing a pass over it"

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Read the product as it is now (Priority: P1)

The owner, or anyone they hand the project to, can read one current specification and learn what the ranch does today, what it will not do, and which words to use. They do not have to rebuild that picture from the product requirements document's phase history, from finished issue write-ups, or from research notes.

**Why this priority**: Every later slice is wrong if the current product is still described in several places that disagree. One current specification is the point of the conversion.

**Independent Test**: Hand a reader who has not seen the repository only the current specification. They can name the product's current capabilities, its limits, and its explicit non-goals, and they never need the old product requirements document to do it.

**Acceptance Scenarios**:

1. **Given** the product requirements document, the domain glossary, accepted decision records, and the behavior already shipped, **When** the pass finishes, **Then** one current specification states today's capabilities, limits, and non-goals in the glossary's settled words.
2. **Given** the old product requirements document describes finished phases or capabilities the product no longer has, **When** a reader opens the current specification, **Then** those phases and retired capabilities are absent from its requirements.
3. **Given** the render-domain work already has an accepted specification and has shipped, **When** the pass finishes, **Then** that specification remains the record of that work and is not rewritten as a second specification of the same outcome.

---

### User Story 2 - Trust the work still to do (Priority: P2)

Someone choosing the next slice sees only work that is still undone. Each remaining slice points at the current specification, or at a follow-on specification when the current one does not already cover that outcome. Shipped, superseded, and deferred items are not sitting in the active queue.

**Why this priority**: A converted specification that leaves a stale queue still sends the next person after work that already shipped or was rejected.

**Independent Test**: Compare the active queue with the product as it exists after the pass. Every active item is still undone, names one observable outcome, and cites the specification that owns that outcome. Deferred items are visibly deferred. Items closed without being built stay closed.

**Acceptance Scenarios**:

1. **Given** an issue whose observable outcome is already true in the product, **When** the pass classifies it, **Then** it leaves the active queue and a reader can see that it shipped, including when the original write-up described a different approach than the one that landed.
2. **Given** an issue the owner has deferred, **When** the pass finishes, **Then** it remains deferred and is not promoted into the active queue or into the current specification's requirements.
3. **Given** an issue that was closed because its premise was wrong or later work replaced it, **When** the pass finishes, **Then** it stays closed and is not copied in as a requirement.
4. **Given** an issue that is still undone, **When** the pass finishes, **Then** it keeps a single observable completion promise and names the specification that owns that promise.

---

### User Story 3 - Tell history from authority (Priority: P3)

A reader who opens an old issue, a research note, or the architecture description can tell whether they are looking at a current decision or at history. The old product requirements document is gone, so it cannot be mistaken for the current specification. Disagreements found during the pass are settled in the lower-authority document, or written down as unresolved. Nothing is silently left in conflict.

**Why this priority**: The conversion fails if a second planning document stays beside the specification, or if the remaining old documents stay unmarked and keep contradicting it.

**Independent Test**: Take each source the pass inventoried. A reader can say which role it has, and can find a written settlement for every place it disagreed with a higher authority.

**Acceptance Scenarios**:

1. **Given** the product requirements document, finished issue write-ups, and research notes, **When** the pass finishes, **Then** the product requirements document is gone from the working tree, and the finished write-ups and research notes remain findable and are marked so they cannot be mistaken for current requirements.
2. **Given** the domain glossary and an accepted decision record, **When** the pass finishes, **Then** both remain the authorities they already are, and the current specification uses them instead of replacing them.
3. **Given** two documents disagree about current behavior, **When** the pass settles the disagreement, **Then** the lower-authority document is corrected, or the disagreement is listed as unresolved with the two statements side by side.
4. **Given** the architecture description claims a behavior the current specification or an accepted decision no longer holds, **When** the pass finishes, **Then** the description matches the higher authority or the mismatch is on the unresolved list.

### Edge Cases

- The product requirements document narrates finished work, including capabilities that were later removed, in the same document as current requirements. The pass carries forward only what is still true, then deletes the document.
- An active issue describes a plan that already shipped by a different approach, so ticking its original checklist would describe files and steps that never landed.
- A glossary banner says the language is still under review while individual entries are already settled. Settled entries govern; the banner is corrected or the unsettled entries are called out.
- A research note is the only write-up of an idea that later became an accepted decision. The note stays historical; the accepted decision stays the authority.
- Deferred multi-monitor work is mentioned beside current requirements. It stays deferred.
- The install guide, release history, or working agreements contradict the current specification. Those documents are not converted wholesale; the contradicting sentence is corrected or listed.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The pass MUST inventory the product requirements document, the issue queue (active, deferred, finished, and closed without being built), the domain glossary, the architecture description, accepted decision records, research notes, and the already accepted render-domain specification, and MUST classify each source as an accepted decision, a verified observation of the product today, present direction, an unresolved assumption, or historical context.
- **FR-002**: The pass MUST produce one current product specification of what the ranch does today. Carried-forward statements MUST cite their source and record that they were checked against the product as it exists now.
- **FR-003**: The current specification MUST use the glossary's settled words. Where an older document uses a retired word for the same idea, the specification uses the settled word.
- **FR-004**: The current specification MUST state current capabilities, limits, and non-goals. It MUST NOT treat a history of finished phases as requirements.
- **FR-005**: The pass MUST classify every active issue as still to do, already satisfied, superseded, or deferred. Already satisfied and superseded issues MUST leave the active queue. Deferred issues MUST stay visibly deferred. Issues closed without being built MUST stay closed.
- **FR-006**: Each issue that remains to do MUST keep a single observable completion promise and MUST name the current specification, or a follow-on specification, that owns that promise.
- **FR-007**: A follow-on specification MUST be created only for undone, user-visible behavior that the current specification does not already cover. The pass MUST NOT replace the issue queue with a task list.
- **FR-008**: The domain glossary and accepted decision records MUST keep their current roles. The pass MUST NOT absorb them into the specification and remove them.
- **FR-009**: After the current specification exists, the pass MUST delete the product requirements document from the working tree. Finished issue write-ups and research notes MUST remain findable and MUST be marked as historical, so they are not current requirements.
- **FR-010**: Where two inventoried documents disagree about current behavior, the pass MUST correct the lower-authority document, or MUST list the disagreement as unresolved with both statements visible. The pass MUST NOT leave a silent conflict.
- **FR-011**: The already accepted render-domain specification MUST remain the record of that shipped work.
- **FR-012**: The pass MUST NOT change what a person can do with the ranch. No capability is added or removed except on paper.
- **FR-013**: The issue index MUST match the classification: active work, deferred work, and the notes for work closed without being built.

### Key Entities

- **Current product specification**: The single description of what the ranch does today, what it will not do, and the limits a reader can check. It replaces the product requirements document as the requirements authority.
- **Follow-on specification**: A separate description of undone, user-visible behavior that the current specification does not already cover. It exists only when that gap is real.
- **Issue**: One slice of work with one observable promise. Active issues are still to do. Deferred, finished, and closed-without-building issues are not active requirements.
- **Domain glossary**: The settled names for the ranch's ideas. The specification uses these names and does not redefine them.
- **Decision record**: An accepted choice about how the product is shaped. Only accepted records govern. Draft, rejected, and superseded records are context.
- **Historical source**: A finished issue write-up or a research note. Findable, and marked so it is not a current requirement. The old product requirements document is not kept.
- **Pass record**: The inventory, the classification of each source, and the list of disagreements that were corrected or left unresolved.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: A reader who has not seen the repository can list the product's current capabilities, limits, and non-goals from the current specification alone, without opening the old product requirements document.
- **SC-002**: 100% of issues that were active at the start of the pass are classified, and the active queue contains only issues that are still undone.
- **SC-003**: Every remaining active issue cites exactly one owning specification and states exactly one observable promise.
- **SC-004**: Zero silent conflicts remain between the current specification, the glossary, accepted decision records, and the architecture description. Every disagreement found is either corrected in the lower-authority document or present on a visible unresolved list.
- **SC-005**: The product requirements document is absent from the working tree. 100% of finished issue write-ups and research notes remain findable and are identifiable as historical.
- **SC-006**: After the pass, a person using the ranch can do the same things they could do before it. The pass changes documents only.

## Assumptions

- The project's adopted specification format is the one already in use: a current specification for user-visible intent, accepted decision records for durable choices, the glossary for names, and issues as the slices that get built. This pass uses that format. It does not invent a second one.
- One current product specification is the conversion of the product requirements document. Remaining issues stay issues. A follow-on specification is added only for a real gap, not one per issue.
- The product requirements document is the input to the current specification, then it is deleted. Spec Kit is the only planning format that remains. Finished issue write-ups stay in their archive. Research notes stay. The glossary, accepted decision records, the architecture description, and the issue queue stay, because they are not the old plan.
- The render-domain specification already accepted for that shipped work is left as that record.
- Install instructions, release history, and day-to-day working agreements are not converted into specifications. A sentence in them that contradicts the current specification is corrected or added to the unresolved list.
- Deferred work stays deferred, and work already closed without being built stays closed, unless the owner gives a new direction during the pass.
- Authority, when documents disagree, follows the order the project already uses: project rules, then accepted decisions, then the current specification, then an issue. Historical sources are outside that order.
- "Checked against the product as it exists now" means checked against the behavior and the accepted records on the current main line, not against an old plan that was never what landed.
