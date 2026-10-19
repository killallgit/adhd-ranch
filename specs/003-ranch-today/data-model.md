# Data Model: Ranch Today

**Status**: Draft description of the baseline and owner-selected startup correction, revalidated 2026-10-18. No new schema, stable Task identity, database, or migration.

## User-owned work

| Entity | Current fields / relationship | Validation and persistence |
|---|---|---|
| Focus | `id: FocusId(String)`, `title`, `description`, `created_at`, ordered `tasks`, optional `timer` | [Domain](../../crates/domain/src/focus.rs), [parser](../../crates/domain/src/parse.rs), [storage](../../crates/storage/src/focus_store.rs). Metadata and checkbox Tasks in `focus.md`; parser requires frontmatter with `id` and `title`. |
| Task | `id`, `text`, `done`, optional `timer`; belongs to one Focus | App writes reject blank text. Parser derives ID as `focus_id:index` from nonempty checkbox lines; no stable persisted Task ID. Complete checks the retained line; Delete removes it. |
| Timer | `duration_secs`, `started_at` (epoch seconds), `status: Running / Expired` | [Timer domain](../../crates/domain/src/timer.rs), [workflow](../../crates/commands/src/timers.rs). Focus Timer in `timer.json`; indexed Task Timers in `task-timers.json`. Missing/corrupt optional sidecars load as no Timer. |
| TimerOwner | Tagged `Focus { focus_id }` or `Task { focus_id, index }` | Selects one owner for start/restart/Clear. Positional addressing is an observation, not protection against hand-edit drift (FR-013). |
| Settings | `caps`, `notifications`, `widget`, `displays`, `agents`, `pens`, `timers` | [Types/defaults](../../crates/domain/src/settings.rs), [host workflow](../../src-tauri/src/app/settings_workflow.rs), `settings.yaml`. Saved custom minutes are positive, unique, and exclude built-ins 10/30/60. Frontend timer presets define the current picker; legacy backend preset variants do not define its menu. |
| Limit state | Focus count, over-Focus flag, IDs of Focuses above Task threshold | [Evaluator](../../crates/domain/src/caps.rs) reads Focuses, not Agent Sessions. Defaults 5/7. Completed Tasks count because all stored Tasks are counted; writes above a threshold remain permitted. |

The data root is `ADHD_RANCH_HOME` when set, otherwise `~/.adhd-ranch` on Unix, with platform path fallbacks on Windows. [Path resolution](../../src-tauri/src/app/paths.rs) is authoritative. No archive layout from draft spec 008 is adopted.

## Duplication contract

FR-002 / US2/AC6 use [the existing duplication workflow](../../crates/commands/src/focus.rs): create an independent Focus with newly assigned identity and creation time; copy the description and Task text/completion in source order; leave the source unchanged. The title is the source title plus “ copy”, with a numeric suffix starting at 2 when needed to avoid an identity collision. Neither the Focus Timer nor any Task Timer is copied. No new schema or stable Task identity is introduced.

## Startup loading and preservation

FR-001 / FR-019 and US1/AC4–AC5 require no seeded work: load existing Focuses, Tasks, Timers, and Settings; an empty work root remains empty. This changes startup behavior, not entities or storage formats. The deletion is implemented in the startup correction branch; automated and native proof status is recorded in [the pass record](validation/pass-record.md#startup-correction-execution--2026-10-18-t030t035).

Existing example Focuses remain ordinary persisted Focuses; do not detect or delete them by title or identity. `.example-focus-seeded` is a legacy artifact, not an active creation gate: leave it untouched and ignore its presence/absence. Directory setup may still create an empty storage directory; it must not create a Focus or Task.

Reloaded Timers retain `started_at`, `duration_secs`, and status. Normal expiry may change a due Timer to Expired, including one due while Ranch was closed, without resetting start/duration or changing Focus/Task text/completion. Saved Settings retain the current loader/default behavior. No durable Animal position, open-card, or Agent-session restoration is introduced. See [the startup decision](research.md#8-resume-saved-work-without-seeding).

## Observed activity

| Entity | Current fields | Ownership / lifecycle |
|---|---|---|
| Agent Session | `id`, `name`, `pen`, `activity: Working / Idle` | [Model](../../crates/domain/src/session/agent_session.rs), [live store](../../crates/storage/src/agent_session_store.rs). Observed from Claude hooks, in memory, never a Focus or Timer owner. |
| Pen | `id`, `name`, derived from repository checkout path | [Pen domain](../../crates/domain/src/session/pen.rs). Groups Agent Sessions, not user work. |
| Hook Firing | `at`, `action: Start / Working / Idle / End`, optional `session_id`, optional `pen`, `changed` | [Types](../../crates/domain/src/agents/hooks.rs), [journal](../../crates/storage/src/agent_hooks/journal.rs). Recent diagnostics retain unchanged Firings; no transcript archive. |
| Hook Wiring | `agent`, `settings_file`, `client_bin`, `socket_path`, `installed`, `enabled` | Same hook types. Validated configuration is distinct from observed delivery. |

Current hooks fold arrival order; Stop and StopFailure map to Idle. No source stamp or transport-change entity exists in today's exported model. This differs from ADR-0002's accepted target; see [research](research.md#5-record-accepted-decision-drift-without-enshrining-it).

## Shared render projection

[RenderScene](../../crates/commands/src/render_scene.rs) contains `animals`, `regions`, and `speciesProfiles`. Each Animal has `id`, `label`, `species`, `size`, optional `regionId`, and `motion`. Species is Pig; Motion is Walking or Resting. Size is fixed or linearly sampled over time. Region rectangles describe enabled display layout; [DisplaySpace](../../src/types/display.ts) supplies frontend geometry.

Each domain projects its state into this flat vocabulary. Movement and sprites consume neither domain types nor source discriminators. Selection resolves separately to a Focus; it cannot make an Agent Animal editable. Rust and [the wire contract test](../../crates/commands/tests/render_scene_contract.rs) own the shape; `src/types/generated/` stays generated.

## State transitions

- **Task:** add → unchecked; Complete → checked in place; undo → unchecked; Delete → removed. Editing text preserves completion. None is Timer Clear.
- **Timer:** start/restart → Running with fresh start time; due Running → Expired with timer-only persistence; Clear → absent. Adding a Task clears an Expired Focus Timer (Revive), never a Task Timer.
- **Focus projection:** absent or Running Focus Timer → Walking; Expired → Resting. A Running Focus Timer drives growth; Expired drives ghostly/away appearance and tray Expired membership. Task Timer state does not control Animal motion; Task Timer expiry changes neither projection nor tray membership.
- **Limits:** crossing above a threshold produces one transition and an allowed notification, not repeated notifications while above. Agent activity cannot change it.
- **Agent:** Start establishes a session; Working/Idle update activity; End removes it. Disabling Agents hides/clears observed sessions and leaves the plugin installed. No activity state uses Focus Timer expiry.

These documented relationships propose no new write path or installation. They do not certify current runtime compliance; the seed-writing policy is resolved and deletion is implemented, while native runtime verification remains pending.
