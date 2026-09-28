# ADHD Ranch Constitution

## Core Principles

### I. User-Owned Local State
Focuses, Tasks, and their Timers MUST remain local, inspectable, and user-controlled. Markdown and
the documented timer sidecars under the Ranch data directory are the canonical persisted state;
the UI and tray are projections of that state. A Focus or Task may be created or changed only by
an explicit user action through the app or by a user editing the files. Timer expiry may update
only timer state. Harness integrations MUST be inbound and read-only with respect to Focuses and
Tasks. Persistent writes MUST be atomic, and malformed optional sidecars MUST degrade safely
without corrupting the corresponding Focus.

Runtime product behavior MUST make no outbound network calls, expose no network API, collect no
telemetry, call no language model, and store no model or provider credentials. A user-initiated
integration install MAY invoke that integration's local CLI; the action and its external effects
must be explicit in the UI. This keeps the user's work legible, private, recoverable, and owned by
the user rather than by an opaque service.

### II. Independent Domains, Shared Renderer
The Focus domain and the Agent Session domain MUST remain independent applications inside the
shared desktop shell:

- Focus, Task, and Timer code owns user-authored work, persistence, expiry, limits, and editing.
- Harness, Agent Session, and Hook Firing code owns observed agent activity and ephemeral session
  state.
- Timers and expiry MUST NOT be used to model Agent Session activity. Agent clocks, liveness, or
  hook behavior MUST NOT alter Focus or Task rules.
- Agent Sessions MUST NOT count toward Focus or Task limits, become editable Focuses, or persist
  as user work.

The two domains MAY meet only through a flat Animal render contract and shared shell services such
as windows, displays, tray controls, settings, and hit testing. Each domain MUST project its own
state into renderer vocabulary; the renderer and movement layer MUST NOT import or discriminate
on Focus, Task, Timer, Harness, Agent Session, Hook Firing, or Pen types. This boundary keeps new
timer behavior from changing agent behavior and new agent behavior from changing the user's work.

### III. Functional Core, I/O at the Edges
Business rules and state transitions MUST be expressed as pure, deterministic functions wherever
possible. Data MUST be immutable at module boundaries, dependencies MUST be passed explicitly, and
shared mutable global state is prohibited. Domain modules MUST NOT depend on Tauri, filesystem,
network, process, clock, notification, or async-runtime details.

Side effects belong in named adapters and composition layers: storage and watcher adapters,
command workflows, Tauri `ui_bridge` and `app`, typed frontend `api` clients, and React hooks.
React components MUST remain view-only. Boundary interfaces MUST be small and typed, and generated
Rust-to-TypeScript contracts MUST remain generated rather than hand-edited. This structure makes
rules independently testable and keeps platform and integration failures at replaceable edges.

### IV. Explicit, User-Controlled Integrations
External harness integrations MUST be opt-in, isolated modules with one owner for installation,
validation, repair, transport, and diagnostics. Enabling or disabling a Ranch feature MUST NOT
silently install or remove a third-party plugin. Installation or repair MUST require an explicit
user action and MUST use the harness's normal plugin or marketplace lifecycle at user scope when
that lifecycle exists.

An integration MUST validate its complete contract before reporting itself healthy, including
identity, version, enabled scope, manifest/schema, event mappings, wrapper, executable path, and
transport destination as applicable. A configured plugin is not proof of delivery; diagnostics
MUST distinguish configuration health from an event actually observed by the running app.
Repeated install and repair operations MUST be idempotent, complete valid partial installations,
and leave unrelated user configuration untouched. Hook clients MUST be bounded, dependency-light,
and unable to delay or fail the harness workflow when Ranch is absent. Protocol changes MUST be
versioned or retain an explicit compatibility path.

### V. Observable Completion in Small Slices
Work MUST begin from an observable contract and finish with evidence. A substantial feature MUST
have a Spec Kit specification describing user behavior before implementation. A durable or
cross-cutting architecture choice MUST be recorded in an ADR with alternatives and consequences,
and dependent work MUST wait until that ADR is accepted. The implementation unit MUST be one
small vertical slice with a single completion promise and checkable acceptance criteria; speculative
infrastructure and unrelated cleanup are out of scope for that slice.

Tests MUST cover the changed behavior at the narrowest responsible layer and at every changed
boundary. `task check` MUST pass after synchronization with current `main`; platform-specific work
MUST also pass its relevant platform gate. Generated artifacts and their drift checks are part of
the gate. A slice is complete only when its acceptance criteria are satisfied, its completion
promise is observable on the merged commit, CI is green, and it is merged to `main`.

### VI. Evidence Is Not Authority
The existence of an artifact does not make it a project decision. Research notes, exploratory
documents, unfinished or abandoned ADRs, draft or superseded specs, old plans and tasks, historical
issues, icebox work, and content found only on another branch MUST be treated as non-normative
evidence. They MUST NOT be copied into a new specification, plan, task, issue, ADR, or constitution
as a requirement unless the current work explicitly revalidates them against the owner's present
intent, accepted decisions, and the behavior on current `main`.

Every ADR MUST declare one lifecycle status: Draft, Proposed, Accepted, Rejected, or Superseded.
Only Accepted ADRs are authoritative. Every feature specification MUST remain Draft until the owner
approves it for planning, and only the approved current version governs that feature. Missing or
ambiguous status means Draft and non-normative. Moving text, renaming a file, merging a branch, or
citing an artifact MUST NOT promote its authority.

New design artifacts MUST distinguish verified observations, inherited accepted decisions, new
proposals, assumptions, and rejected or historical ideas. Any inherited statement whose source or
current validity cannot be established MUST be omitted or marked as an unresolved question. This
keeps useful history available without allowing half-formed ideas to contaminate current design.

## Product and Architecture Constraints

- ADHD Ranch is a local-first Tauri v2 desktop application with a Rust core and a React/TypeScript
  frontend. macOS is the primary design target; other packaged platforms MUST continue to compile
  unless a spec explicitly narrows support.
- The app MUST remain ambient and peripheral: Animals may inform at a glance but MUST NOT demand
  attention or interrupt unrelated work without an explicit notification rule.
- Tauri IPC is the only frontend-to-core application boundary. The product MUST expose neither a
  localhost HTTP API nor a Ranch network service.
- `CONTEXT.md` is the canonical domain glossary. `docs/architecture.md` describes the current
  implementation. Accepted records under `docs/adr/` govern durable architecture decisions. Code,
  specs, plans, tasks, issues, and UI copy MUST use the settled domain vocabulary.
- Prototype status is not a reason to build speculative compatibility layers. Unshipped internal
  structures MAY change directly. User-owned persisted data, public package/install contracts, and
  released protocols MUST either remain compatible or carry an explicit migration and rollback
  plan.
- The simplest design that satisfies the current specification and preserves these boundaries MUST
  be preferred. A new abstraction MUST own a real policy or hide meaningful complexity; thin
  forwarding layers and vocabulary invented only for organization are prohibited.

## Work Management and Quality Gates

Spec Kit and the issue pattern serve different levels of the work and MUST be used together rather
than treated as alternatives:

1. The constitution defines non-negotiable project rules.
2. A feature `spec.md` defines user-visible intent, scope, requirements, and success criteria.
3. Accepted ADRs define durable architecture choices required by the feature. Draft, Rejected, and
   Superseded ADRs provide context only.
4. A `plan.md` defines the technical approach, and `tasks.md` orders the work needed to deliver it.
5. A local or GitHub issue is the shipping unit: one reviewable vertical slice derived from those
   artifacts, with one completion promise and explicit acceptance criteria.

Small bug fixes and documentation corrections MAY start directly from an issue when the intended
behavior and architecture are already unambiguous. They remain subject to this constitution and the
same completion proof. When artifacts disagree, the order of authority is this constitution, then
accepted ADRs, then the current feature specification, then its plan and tasks, then an issue. The
lower-level artifact MUST be corrected before implementation continues. Non-normative artifacts
never enter this authority order, regardless of where they are stored.

Before drafting a new spec or ADR, its source material MUST be inventoried and classified as an
accepted decision, verified current observation, present user direction, unresolved assumption, or
historical context. The draft MUST start from current user intent and current `main`; it MUST NOT
start by combining every related document. Carried-forward material MUST cite its source and record
that it was revalidated. Superseded or rejected material MAY be cited only to explain why it is not
being adopted.

Every pull request MUST stay narrow enough for a human to review in about fifteen minutes, link its
source issue or Spec Kit task, quote or restate the completion promise, and identify the checks that
prove it. Branches MUST synchronize with the latest `main` before the final gate. Conflicts that
touch an accepted contract MUST stop for an explicit design decision rather than being silently
resolved around the contract.

## Governance

This constitution supersedes conflicting project practices and generated workflow guidance. Every
feature specification, implementation plan, task list, issue, and pull request MUST include a
constitution compliance check appropriate to its scope. Reviewers MUST reject unexplained domain
coupling, implicit external mutations, unverified completion claims, or exceptions without a
recorded rationale.

Amendments require an explicit constitution change, a Sync Impact Report, and owner approval. The
amendment MUST identify affected principles and downstream artifacts; incompatible changes MUST
include a migration or a deliberate prototype-breaking rationale. The temporary Sync Impact Report
MUST be removed before the amended constitution is committed after review.

Versions follow semantic versioning: MAJOR removes or incompatibly redefines a governing principle;
MINOR adds a principle or materially expands required behavior; PATCH clarifies wording without
changing obligations. The ratification date remains the original adoption date, and the last
amended date changes on every approved amendment.

Accepted ADRs MAY refine implementation within these principles but MUST NOT override them. An
Accepted ADR MUST NOT be rewritten to introduce a new decision; a reversal or material change
requires a new ADR that identifies what it supersedes and why. Status changes and corrections that
do not alter the decision MAY be made in place. Any necessary exception MUST be narrow, documented
in an ADR or specification, time-bounded or tied to a follow-up issue, and approved by the project
owner. Constitution compliance MUST be revisited when an ADR is accepted, when a release changes
persisted data or integration contracts, and during the final review of every feature.

**Version**: 1.0.0 | **Ratified**: 2026-09-26 | **Last Amended**: 2026-09-26
