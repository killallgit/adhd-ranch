# Feature Specification: One Hook Firing Arrives Once

**Feature Branch**: `007-one-hook-delivery`

**Created**: 2026-10-15

**Status**: Draft

**Input**: Issue 062, under accepted ADR-0002. A hook entry the ranch wrote can be removed by the ranch, and one Hook Firing is delivered once. Today a new install uses the Claude plugin and a stable client, and an older direct entry in Claude's settings is left in place (`specs/003-ranch-today/spec.md` FR-015).

## User Scenarios & Testing *(mandatory)*

### User Story 1 - One firing, one delivery (Priority: P1)

The person has Agents as Animals on. One thing happening in Claude Code produces one Hook Firing in the Agent Hooks window, not a second copy that changed nothing.

**Why this priority**: A doubled Firing makes the journal lie and will double any later reaction to a change.

**Independent Test**: Trigger one Claude Code event that the ranch handles. The Agent Hooks window shows one Firing for it.

**Acceptance Scenarios**:

1. **Given** a single Claude Code event the ranch handles, **When** it is delivered, **Then** the Agent Hooks window shows one Firing.
2. **Given** an older direct hook entry the ranch wrote for its own socket, **When** the person installs or repairs hooks, **Then** that entry is removed and a current install remains.
3. **Given** a hook entry that names a different socket or a different program, **When** the person installs or repairs hooks, **Then** that entry is left alone.

### Edge Cases

- Windows has no hook delivery. This feature does not add one.
- Turning Agents as Animals off still leaves the Claude plugin installed.
- Two different Ranch data directories are not a supported setup. This feature does not delete a hook aimed at some other socket.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: One handled Claude Code event MUST produce one Hook Firing.
- **FR-002**: An install or repair MUST remove an older direct hook entry the ranch wrote for its own socket, and MUST leave an entry that names a different socket or a different program.
- **FR-003**: The client path a new install uses MUST NOT change because the app was launched from a different directory.
- **FR-004**: This feature MUST NOT write a hook entry as a side effect of launching the app. Install and repair remain an explicit person action.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: One handled event shows one Firing in Agent Hooks.
- **SC-002**: After install or repair, an older direct entry for this ranch's socket is gone, and an entry for a different socket is still there.
- **SC-003**: Launching the app from another directory does not create a second client path in the hook entry.
