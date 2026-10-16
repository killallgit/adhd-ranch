# Feature Specification: Task Timers Survive Hand Edits

**Feature Branch**: `004-task-timer-hand-edits`

**Created**: 2026-10-15

**Status**: Draft

**Input**: Issue 056. A Task Timer stays on the Task the person attached it to when they reorder or insert Tasks by editing the Focus file. The current product records the opposite limit in `specs/003-ranch-today/spec.md` FR-013.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - The Timer stays with the Task (Priority: P1)

The person attaches a Timer to a Task, then edits the Focus file by hand and moves that Task or inserts another Task above it. The Timer is still on the original Task.

**Why this priority**: Hand-editing is a supported way to change Tasks. A Timer that silently moves is the person losing track of the work they timed.

**Independent Test**: Start a Timer on the second Task. Insert a new Task above it in the file and save. The Timer is still on the Task that had it, not on the new line.

**Acceptance Scenarios**:

1. **Given** a Timer on a Task, **When** the person inserts a Task above it and saves, **Then** the Timer remains on the original Task.
2. **Given** a Timer on a Task, **When** the person reorders the checkbox lines and saves, **Then** the Timer remains on the original Task.
3. **Given** two Tasks whose text is identical and one has a Timer, **When** the person reorders them, **Then** the ranch does not pretend to know which line the Timer belongs to. The behavior for that case is stated and tested.

### Edge Cases

- Editing the text of the timed Task.
- Deleting the timed Task from the file by hand.
- A corrupt Task Timer sidecar still degrades to no Timers and does not block the Focus.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: After a hand-edit that inserts or reorders Tasks, a Task Timer MUST remain attached to the Task the person attached it to.
- **FR-002**: The Focus file MUST stay plain checkbox lines. This feature MUST NOT add a network API or a second writer of Focuses.
- **FR-003**: If the attachment cannot be known, as with two identical Task lines, the ranch MUST follow one stated rule and MUST test that rule.
- **FR-004**: A decision that changes the Focus file format MUST be an accepted decision record before the format changes. A repair that does not change the file format does not need one.

### Non-goals

- **FR-005**: This feature MUST NOT revive Task Timers, change Focus Timer growth, or change which Animal is drawn.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Inserting a Task above a timed Task leaves the Timer on the timed Task.
- **SC-002**: Reordering checkbox lines leaves each Timer on the Task it was attached to, except for the stated ambiguous case.
- **SC-003**: A person can still hand-edit a Focus file and see the overlay catch up.
