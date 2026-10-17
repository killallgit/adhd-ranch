# Feature Specification: Ranch Today

**Feature Branch**: `003-ranch-today`

**Created**: 2026-10-15

**Status**: Draft

**Input**: This specification converts the product requirements document into the current description of what the ranch does today. It stays Draft until the owner approves it.

## Clarifications

### Session 2026-10-17

- Q: Should known bugs and limitations in Ranch Today be observations that later features may fix, or required behavior that future changes must preserve? → A: Record known limitations as observations of the current product; follow-on specifications may fix them.

## Constitution Compliance Check

This check applies to the scope and requirements of this Draft baseline; it does not certify that every current runtime path conforms.

| Principle | Scope check |
|---|---|
| I — User-owned local state | FR-001–FR-003 preserve canonical local files, explicit user edits, safe optional sidecars, and timer-only expiry writes; FR-012 excludes outbound product network, telemetry, and model calls. |
| II — Independent domains | FR-005, FR-007, and FR-010 keep Agent Sessions separate from Focus editing, Timers, and Limits; domains meet through the accepted flat Animal render contract. |
| III — Functional core, typed edges | The baseline retains the existing pure domain rules, I/O adapters, Tauri IPC, and generated Rust-to-TypeScript contracts without proposing implementation changes. |
| IV — Explicit integrations | FR-010–FR-011 require opt-in visibility and explicit installation/repair; configuration health and observed delivery are separate evidence. |
| V — Observable completion | The plan and tasks validate existing behavior through current automated gates and native observations; failed or unverified criteria cannot support a completion claim. |
| VI — Evidence and authority | This spec remains Draft. Source observations and known limitations do not override the constitution or Accepted ADRs; Draft follow-ons are context, not additional requirements. |

ADR-0002's accepted transport direction is not fully implemented today. FR-014 records that gap, not an approved alternative or exception; follow-on work must reconcile lower-level artifacts with accepted decisions before implementation.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Glance and know what is open (Priority: P1)

A person looks at the screen and sees one Animal for each Focus, wandering slowly on the enabled displays. Pig is the only Species. The Animals do not interrupt other work. A glance is enough to know what is open.

**Why this priority**: The ranch is a peripheral reminder. If a glance does not show the open Focuses, the product is not doing its job.

**Independent Test**: Create two Focuses and look at the overlay. Two Pig Animals wander. Clicking empty overlay space reaches the app underneath.

**Acceptance Scenarios**:

1. **Given** two Focuses exist, **When** the person looks at the overlay, **Then** two Animals of Species Pig are wandering on the enabled displays.
2. **Given** the pointer is not over an Animal, **When** the person clicks, **Then** the click reaches whatever is underneath the overlay.
3. **Given** the Animals are wandering, **When** the person is working in another app, **Then** the ranch does not raise a window or demand attention on its own, except for a notification the person has allowed.

---

### User Story 2 - Work a Focus from its Animal (Priority: P1)

The person clicks a Focus Animal and gets a card: the Focus title, its Tasks, and its Timers. They can rename, duplicate, or delete the Focus, and add, edit, Complete, or Delete Tasks. They can start, restart, or Clear a Timer on the Focus or on a Task. Clicking elsewhere or pressing Escape closes the card.

**Why this priority**: The glance only helps if the person can change the work from the Animal they clicked.

**Independent Test**: Click one Focus Animal, add a Task, Complete it, start a Timer, and close the card. The markdown for that Focus matches what the card showed.

**Acceptance Scenarios**:

1. **Given** a Focus Animal, **When** the person clicks it, **Then** a card opens beside it with the title, the Tasks, and the Timer controls.
2. **Given** the card is open, **When** the person adds, edits, Completes, or Deletes a Task, **Then** the Focus file changes to match.
3. **Given** a running Focus Timer, **When** time passes, **Then** that Animal grows from its base size toward a larger size, and when the Timer is Expired the Animal rests, looks ghostly, faces away, and appears in the tray's Expired section.
4. **Given** an Expired Focus Timer, **When** the person adds a Task, **Then** the Focus is revived and the Animal returns to normal.
5. **Given** a Task Timer Expires, **When** the person looks at the ranch, **Then** that Task shows Expired and a notification can fire, and the Animal's size, motion, and the tray's Expired section do not change because of that Task Timer.

---

### User Story 3 - Run the ranch from the tray (Priority: P2)

The person uses the tray to create a Focus, delete one, Gather Animals back to the primary display, turn Agents as Animals on or off, open Settings, and quit. The tray icon turns red while the ranch is Over Limit.

**Why this priority**: Creation, deletion, and Gather are how the person keeps the ranch matched to the work they mean to hold.

**Independent Test**: From the tray, create a Focus, Gather, delete the Focus, and open Settings. Each action does only that thing.

**Acceptance Scenarios**:

1. **Given** the tray menu, **When** the person chooses New Focus and enters a title, **Then** a new Animal appears.
2. **Given** Animals have been dragged off the primary display, **When** the person chooses Gather, **Then** they return to the primary display.
3. **Given** delete confirmation is on, **When** the person deletes a Focus, **Then** the ranch asks first and the Animal disappears only after confirmation.

---

### User Story 4 - Hand-edit the markdown (Priority: P2)

The person edits a Focus file in their own editor. The overlay catches up within about a second. Tasks are checkbox lines. An optional Timer sidecar sits beside the Focus file.

**Why this priority**: Markdown is the canonical state. The app is a projection of those files.

**Independent Test**: Append a checkbox line to a Focus file and save. The card shows the new Task without using the app to add it.

**Acceptance Scenarios**:

1. **Given** a Focus file, **When** the person adds or Completes a checkbox line and saves, **Then** the Animal's Task list matches the file.
**Current limitation:** Task Timers are stored by Task position, so a hand-edit that reorders or inserts Tasks can move a Timer onto another Task. This records current behavior; a follow-on specification may fix it.

---

### User Story 5 - See an agent without tracking it as work (Priority: P3)

The person can turn on Agents as Animals. While enabled, each ongoing Claude Code Agent Session the ranch has observed through Hook Firings appears as an Animal. Those Animals are not Focuses: the person did not create them, they are not editable, they do not count toward Limits, and they disappear from the screen when the setting is turned off. The Claude plugin stays installed.

**Current limitation:** A session already running when the ranch starts listening stays unseen until it sends a Hook Firing. The ranch does not enumerate running sessions at startup. This is an observation that a follow-on specification may change.

**Why this priority**: Agent activity is a second thing the person may want to see. It must not become user-owned work.

**Independent Test**: With the ranch listening and Agents as Animals on, have one ongoing Claude Code session emit a Hook Firing that establishes or updates the session. Its Animal appears and cannot be opened as a Focus. Turn the setting off. That Animal is gone and the Focus Animals remain.

**Acceptance Scenarios**:

1. **Given** Agents as Animals is off, **When** a Claude Code session is running, **Then** no Agent Session Animal is shown.
2. **Given** the person turns Agents as Animals on and the plugin is missing, **When** the tray offers Install Hooks, **Then** nothing is installed until the person chooses that action.
3. **Given** an Agent Session Animal is showing, **When** the person clicks it, **Then** no Focus card opens.
4. **Given** Agents as Animals is on, **When** the person opens Agent Hooks, **Then** they can see whether hooks are in place, which sessions the ranch is holding, and recent Hook Firings, including a Firing that changed nothing.

### Edge Cases

- The first launch creates one example Focus so the ranch is not empty. A marker stops that example from returning after the person deletes every Focus.
- A missing or corrupt Timer sidecar is treated as no Timer. It does not block the Focus file.
- Going Over Limit still saves the Focus or Task. The tray icon turns red. If the relevant notification is enabled, it fires once for that crossing; if disabled, no notification fires.
- Only the primary display is enabled until the person enables more in Settings. Animals do not wander through gaps between displays.
- Settings changed in the app apply without a restart. A hand-edit of the settings file applies on the next launch.
- On Windows the ranch builds and stores data under the platform data directory. It does not receive Hook Firings there.
- A late Hook Firing is not ignored because of its age. Stop and a failed stop are not shown as different kinds of ending. The overlay learns that sessions changed by reading the list again, not by receiving the change itself.
- The ranch does not remove an older direct hook entry it once wrote into Claude's settings. A new install uses the Claude plugin and a stable local client.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The ranch MUST keep each Focus as a directory of markdown under the Ranch data directory, with Tasks as checkbox lines in the Focus file. Optional sidecars MUST hold the Focus Timer and the Task Timers. The person MUST be able to create and change a Focus or Task from the app or by editing those files. Timer expiry MUST update only Timer state.
- **FR-002**: Completing a Task MUST keep it in the Focus, shown as done. Deleting a Task MUST remove it. Clear MUST apply to a Timer, not to a Task or a Focus.
- **FR-003**: A Timer MUST belong to exactly one Focus or one Task. The person MUST be able to start one from the built-in durations of 10, 30, or 60 minutes, or add a custom duration as a saved chip, and MUST be able to restart or Clear it. Saved chips MUST be shared across timer pickers and persisted in Settings; removing or clearing saved chips MUST leave the built-ins available and MUST NOT clear a running Timer. An Expired Focus Timer MUST be cleared when a Task is added. A Task Timer MUST NOT be revived.
- **FR-004**: The ranch MUST show one Animal per Focus on the enabled displays, using Species Pig. The person MUST be able to drag an Animal and toss it. Gather MUST bring Animals back to the primary display. The overlay MUST let clicks through everywhere except on an Animal.
- **FR-005**: Clicking a Focus Animal MUST open a card for that Focus and MUST NOT open a card for an Agent Session Animal. The card MUST close when the person clicks outside it or presses Escape.
- **FR-006**: The tray MUST offer New Focus, delete, Gather, Agents as Animals, Settings, and quit. Delete MUST ask first when confirmation is on. The tray icon MUST turn red while the ranch is Over Limit.
- **FR-007**: A Limit MUST be a soft threshold. The defaults MUST be 5 Focuses and 7 Tasks in one Focus, and the person MUST be able to change them in Settings. Crossing a Limit MUST NOT block the write. Agent Sessions MUST NOT count toward a Limit.
- **FR-008**: Settings MUST be a file the app edits from the Preferences window, labeled Preferences only as the macOS menu name. Changes made in the app MUST apply without a restart.
- **FR-009**: DisplaySpace MUST be the enabled displays. Animals MUST NOT move through the gaps between them. The primary display MUST be the one enabled until the person enables others.
- **FR-010**: Agents as Animals MUST be off until the person turns it on. An Agent Session MUST be observed from Claude Code, never created by the person, never stored as a Focus, and never editable. Turning the setting off MUST hide those Animals and MUST leave the Claude plugin installed.
- **FR-011**: Installing or repairing the Claude plugin MUST happen only after an explicit Install Hooks action. A Hook Firing MUST be recorded even when it changed nothing. The Agent Hooks window MUST show wiring, live sessions, and recent Firings.
- **FR-012**: The running ranch MUST make no outbound network call, expose no network API, collect no telemetry, and call no language model. The only frontend-to-core boundary MUST be Tauri IPC.

### Limits

FR-013 through FR-015 describe current limitations as observations. Follow-on specifications may change this behavior.

- **FR-013**: Task Timers are currently indexed to Task position. A hand-edit that reorders or inserts Tasks can move a Timer onto another Task. That drift is not repaired today.
- **FR-014**: A Hook Firing that arrives late is currently eligible to apply. The ranch does not yet tell a turn that ended from a turn that died as two different Firings. A session change is learned by reading the session list again.
- **FR-015**: The ranch currently leaves an older direct hook entry in Claude's settings when a previous build wrote it. New hook delivery goes through the Claude plugin and the stable client under the Ranch data directory.
- **FR-016**: macOS MUST be the design target. Windows and Linux packages MUST still build. Hook delivery MUST NOT be available on Windows.
- **FR-017**: The ranch MUST NOT demand attention except through a notification the person has turned on, or through a window the person opened.

### Non-goals

- **FR-018**: The ranch MUST NOT sync, host multiple users, or import work from Jira, GitHub, or Linear.
- **FR-019**: The ranch MUST NOT offer a network API, a slash command, or any writer of Focuses and Tasks other than the person and Timer expiry.
- **FR-020**: The ranch MUST NOT complete Tasks on its own, merge Focuses on its own, or forward notifications into a Harness.
- **FR-021**: The ranch MUST NOT enable every display on first launch, and MUST NOT offer Wrangle. Gather is the action that brings Animals back.
- **FR-022**: The ranch MUST NOT ship a second Species, and MUST NOT read a Harness other than Claude Code.

### Key Entities

- **Focus**: User-owned work, stored as markdown. Shown as an Animal.
- **Task**: One checkbox line in a Focus.
- **Timer**: A countdown on one Focus or one Task.
- **Animal**: The sprite that wanders. Focus and Agent Session each project their own.
- **Agent Session**: One observed Claude Code run. Not user-owned work.
- **Limit**: A soft threshold on open Focuses or Tasks.
- **Settings**: The stored configuration.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: A person can name their open Focuses by looking at the overlay, without opening another tool.
- **SC-002**: A person can add, Complete, and Delete a Task, and start or Clear a Timer, from the Focus Animal's card, and the Focus file matches.
- **SC-003**: A hand-edit of a Focus file shows on the overlay within about a second.
- **SC-004**: Going past a Limit still saves and turns the tray icon red. If the relevant notification is enabled, it fires once for that crossing; if disabled, no notification fires. Agent Sessions do not count toward Limits.
- **SC-005**: With Agents as Animals off, no Agent Session Animal is shown. With it on, ongoing sessions observed through Hook Firings are visible and cannot be edited as Focuses.
- **SC-006**: The running app makes no outbound product network call.

## Sources

Checked against `main` on 2026-10-15.

- FR-001: source `.specify/memory/constitution.md` (User-Owned Local State) and `docs/architecture.md` (Persistence) — checked against `main` on 2026-10-15. Focus directories, checkbox Tasks, and Timer sidecars are the stored state.
- FR-002: source `CONTEXT.md` (Complete, Delete, Clear) — checked against `main` on 2026-10-15. The card still completes a Task in place and deletes a Task by removing its line.
- FR-003: source `docs/architecture.md` (Timers), `CONTEXT.md` (Timer, Revive), and PR #89 (`8fa695d`) — checked against `main` on 2026-10-15. Built-ins are 10, 30, and 60 minutes; custom chips persist in `timers.saved_minutes`, and timer edits preserve other windows' saves. `Timers` starts, clears, revives, and expires. Adding a Task revives an Expired Focus Timer. A Task Timer is not revived.
- FR-004: source `docs/adr/0003-rust-owned-render-scene.md` and `docs/architecture.md` (Renderer read model, Core interaction loop) — checked against `main` on 2026-10-15. Rust owns the scene and the Pig profile. Gather and drag-and-toss still behave as the interaction loop describes.
- FR-005: source `docs/adr/0001-two-apps-over-one-renderer.md` — checked against `main` on 2026-10-15. Selection resolves only to a Focus. An Agent Session Animal has no card.
- FR-006: source `docs/architecture.md` (Application) and `README.md` (Day-to-day usage) — checked against `main` on 2026-10-15. The tray still offers those actions, and the icon turns red while Over Limit.
- FR-007: source `.specify/memory/constitution.md` (Limits) and `docs/architecture.md` (Limits) — checked against `main` on 2026-10-15. Defaults remain 5 and 7. Writes past a Limit succeed.
- FR-008: source `CONTEXT.md` (Settings) and `docs/architecture.md` (Configuration) — checked against `main` on 2026-10-15. The Preferences window persists Settings and applies them without a restart.
- FR-009: source `CONTEXT.md` (DisplaySpace) and `docs/architecture.md` (Display spanning) — checked against `main` on 2026-10-15. The primary display is the default enabled display. Animals stay inside visible regions.
- FR-010: source `docs/adr/0001-two-apps-over-one-renderer.md` and `docs/architecture.md` (Agents as Animals) — checked against `main` on 2026-10-15. The setting is off by default. Sessions are in memory. Turning the setting off drops them and leaves the plugin.
- FR-011: source `.specify/memory/constitution.md` (Explicit, User-Controlled Integrations) and `docs/architecture.md` (Hook install, Journal, Agent Hooks window) — checked against `main` on 2026-10-15. Install is an explicit action. The journal keeps Firings that changed nothing.
- FR-012: source `.specify/memory/constitution.md` (User-Owned Local State, product constraints) — checked against `main` on 2026-10-15. No Ranch network API is present. IPC is Tauri.
- FR-013: source `docs/architecture.md` (Timers) and `crates/storage/src/focus_store.rs` — checked against `main` on 2026-10-15. Task Timers are stored by index. No reorder repair is present.
- FR-014: source `docs/adr/0002-agent-hook-transport.md` and `docs/architecture.md` (Events, Delivery) — checked against `main` on 2026-10-15. Stop and a failed stop both become idle. No late-frame stamp is applied. The change event does not carry the change.
- FR-015: source `docs/architecture.md` (Stable client, Hook install) and `src-tauri/src/app/claude_hook.rs` — checked against `main` on 2026-10-15. The app copies a stable client and does not write Claude's settings file. It also does not remove older direct entries.
- FR-016: source `.specify/memory/constitution.md` (Product and Architecture Constraints) and `docs/architecture.md` (Windows) — checked against `main` on 2026-10-15. Windows still compiles. The hook transport is not available there.
- FR-017: source `.specify/memory/constitution.md` (ambient and peripheral) — checked against `main` on 2026-10-15. Notifications are settings-gated. The overlay does not steal focus.
- FR-018: source `PRD.md` (Out of scope) — checked against `main` on 2026-10-15. No sync and no external aggregators are present. `PRD.md` was removed after this check.
- FR-019: source `.specify/memory/constitution.md` and `docs/architecture.md` (Writers) — checked against `main` on 2026-10-15. The proposal queue and localhost HTTP API are gone. Writers are the person and Timer expiry.
- FR-020: source `PRD.md` (Non-goals) — checked against `main` on 2026-10-15. No auto-complete and no notification forwarding are present. `PRD.md` was removed after this check.
- FR-021: source `issues/icebox/021-all-monitors-default.md` and `issues/icebox/022-wrangle.md` — checked against `main` on 2026-10-15. Both remain deferred. Gather exists. All-displays default and Wrangle do not.
- FR-022: source `CONTEXT.md` (Species, Harness) and `docs/adr/0001-two-apps-over-one-renderer.md` — checked against `main` on 2026-10-15. Pig is the only Species. Claude Code is the only Harness the ranch reads.

The record of the shipped render-domain work is `specs/001-render-domain-seam/spec.md`. This specification does not restate it.
