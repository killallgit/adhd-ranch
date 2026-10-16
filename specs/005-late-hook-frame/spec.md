# Feature Specification: A Late Hook Frame Does Not Rewrite the Past

**Feature Branch**: `005-late-hook-frame`

**Created**: 2026-10-15

**Status**: Draft

**Input**: Issue 059, under accepted ADR-0002. A Hook Firing that arrives late does not change the Agent Session, and the Agent Hooks window can tell a turn that ended from a turn that died. Today both failures are a limit in `specs/003-ranch-today/spec.md` FR-014.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - An old frame changes nothing (Priority: P1)

A Hook Firing that describes an older moment than the one the ranch already applied does not move the Agent Session Animal and does not replace the session the ranch is holding.

**Why this priority**: A late Firing that overwrites a newer one makes the Animal lie about the session.

**Independent Test**: Apply a newer Firing, then deliver an older one for the same session. The Animal and the held session stay on the newer Firing.

**Acceptance Scenarios**:

1. **Given** the ranch has applied a Firing for a session, **When** an older Firing for that session arrives, **Then** the session and its Animal stay as the newer Firing left them.
2. **Given** a Firing is the newest for that session, **When** it arrives, **Then** the ranch applies it.

---

### User Story 2 - Ended and died look different (Priority: P2)

In the Agent Hooks window, a turn that ended and a turn that died are two different Firings. The person can tell which one happened.

**Why this priority**: Both are an ending, and today the ranch cannot show which.

**Independent Test**: Produce one normal end and one failed end. The window shows them as different Firings.

**Acceptance Scenarios**:

1. **Given** a turn ends normally, **When** the person opens Agent Hooks, **Then** that Firing is marked as an end.
2. **Given** a turn dies without a normal end, **When** the person opens Agent Hooks, **Then** that Firing is marked as a failed end, not as the same kind of Firing as a normal end.

### Edge Cases

- A Firing the ranch could not read is still shown.
- The client still exits quietly when the ranch is not listening.
- No new Harness event is added by this feature.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: A Hook Firing older than the one already applied for that Agent Session MUST NOT change that session.
- **FR-002**: A normal end and a failed end MUST be distinguishable in the Agent Hooks window.
- **FR-003**: An Agent Session MUST NOT gain a stored transport field, and this feature MUST NOT add a Harness event.
- **FR-004**: The hook client MUST stay dependency-free and MUST still exit without output when the ranch is not listening.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: An older Firing does not change the Animal or the held session.
- **SC-002**: A person can tell a normal end from a failed end in Agent Hooks without reading the raw payload.
