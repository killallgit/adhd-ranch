# Feature Specification: An Unheard Turn Stops Looking Busy

**Feature Branch**: `006-unheard-turn-ends`

**Created**: 2026-10-15

**Status**: Draft

**Input**: Issue 060, under accepted ADR-0002. An Agent Session whose turn ended without the ranch hearing about it stops being drawn as working, without restarting the app. Today the ranch keeps showing that session as working until it hears an end.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - The Animal rests when the turn is over (Priority: P1)

The person is watching an Agent Session Animal that is walking because the session is working. The turn ends and the ranch never receives the ending Firing. Without a restart, the Animal stops being drawn as working.

**Why this priority**: A walking Animal means the agent is mid-turn. Leaving it walking after the turn is over is a false glance.

**Independent Test**: Hold a session in the working state, drop the ending Firing, and let the ranch notice the turn is over. The Animal rests. The app was not restarted.

**Acceptance Scenarios**:

1. **Given** an Agent Session is shown as working, **When** its turn has ended and no ending Firing arrived, **Then** the Animal stops being shown as working without an app restart.
2. **Given** an Agent Session is shown as working, **When** the turn is still going, **Then** the Animal keeps being shown as working.
3. **Given** an Agent Session is already resting, **When** the ranch looks for unheard endings, **Then** that session stays resting.
4. **Given** the ranch has no way to tell whether the turn ended, **When** it looks, **Then** it leaves the session as it is rather than forcing it to rest.

### Edge Cases

- Agents as Animals is turned off. No session Animal is shown, and this check does not create one.
- The ranch does not remove the session just because the turn went quiet. Removal remains the ending Firing or turning the setting off.
- This feature does not read the text of a session transcript.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: An Agent Session shown as working MUST stop being shown as working, without an app restart, when its turn has ended and the ranch did not receive the ending Hook Firing.
- **FR-002**: A session whose turn is still going MUST keep being shown as working.
- **FR-003**: A session the ranch cannot judge MUST be left unchanged.
- **FR-004**: This check MUST NOT delete the session, MUST NOT read transcript text, and MUST NOT run in a way that delays accepting a Hook Firing.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: After an unheard end, the Animal is resting and the app was not restarted.
- **SC-002**: A session that is still working is not forced to rest.
- **SC-003**: A session the ranch cannot judge is unchanged.
