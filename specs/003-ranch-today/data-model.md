# Data Model: Ranch Today

**Status**: Draft description of current implementation, checked 2026-10-17. No new schema, stable Task identity, database, or migration.

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
- **Focus projection:** running Focus Timer drives growth; Expired drives resting/ghostly/away appearance and tray Expired membership. Task Timer expiry changes neither projection nor tray membership.
- **Limits:** crossing above a threshold produces one transition and an allowed notification, not repeated notifications while above. Agent activity cannot change it.
- **Agent:** Start establishes a session; Working/Idle update activity; End removes it. Disabling Agents hides/clears observed sessions and leaves the plugin installed. No activity state uses Focus Timer expiry.

These relationships preserve Constitution I–IV without authorizing new writes or installation.
