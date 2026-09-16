# 054 — Timers module keyed by Timer Owner

## Parent PRD

PRD.md §FR3 (timers) and issues 028, 044, 052, 053. Architecture review 2026-09-15, candidate 1.

## What to build

Today a Timer is understood by reading about eight modules, and the Focus and Task paths are written twice at every tier: `Commands::start_timer` / `start_task_timer`, the two arms of `TimerExpiryWorkflow::run_once`, and four `FocusStore` methods. The Revive rule ("adding a Task revives its Focus") is hidden inside `MarkdownFocusStore::append_task`.

Collapse all of it behind one module keyed by **Timer Owner**.

### Target shape

- `TimerOwner` in `domain`: `Focus { focus_id }` | `Task { focus_id, index }`. Tasks stay matched by position; see 056.
- `commands::Timers`, built once in `app::run`, holding the store, clock, settings provider and `NotificationSink`:
  - `start(owner, preset)`
  - `clear(owner)`
  - `revive_if_expired(owner)`
  - `expire_due(now_secs)` — persists Expired and notifies through the enabled notification sources
- `domain::tick` stays as the pure core inside `expire_due`. `TimerExpiryWorkflow` is deleted, and the 1s host loop calls the single long-lived `Timers` instead of rebuilding a workflow and a sink every tick.
- Narrow `TimerStore` trait in `storage`: `list` + `write_timer(owner, Option<FocusTimer>)`. `MarkdownFocusStore` implements it, and a `pub` in-memory adapter next to it replaces the hand-written `unimplemented!()` stubs in the `commands` tests.
- `FocusStore` loses `update_timer`, `clear_timer`, `update_task_timer` and `clear_task_timer`.
- `Commands::append_task` calls `timers.revive_if_expired(Focus)`. Storage stops doing it.
- `create_focus` builds its initial running Timer through the same `Timers` helper as `start`.
- IPC collapses from four commands to `start_timer(owner, preset)` and `clear_timer(owner)`, with `TimerOwner` exported to TypeScript. `FocusWriter` follows.
- Delete `NewFocus::with_timer_preset` / `timer_preset()` (never read) and `domain::growth_factor` (called only from its own tests; the frontend computes scale itself).

### Out of scope

- Changing the `task-timers.json` format or how Task Timers bind to Tasks (056).
- `FocusStore`'s remaining methods and the load/save repository idea.
- Frontend timer helpers; `expired` and scale move in 055.

## Completion promise

Starting, clearing, reviving and expiring a Timer all go through one module keyed by Timer Owner, with one storage write path, and no Focus/Task duplication left at the commands, storage or IPC tiers.

## Acceptance criteria

- [x] `TimerOwner` exists in `domain` and is the only way callers name a Timer's owner
- [x] `Timers` exposes `start`, `clear`, `revive_if_expired` and `expire_due`; `TimerExpiryWorkflow` is gone
- [x] `Timers` is constructed once in `app::run`; the 1s loop does not rebuild it or its sink per tick
- [x] `TimerStore` has one write method (`focuses` + `write_timer`); `FocusStore` no longer has any timer method
- [x] An in-memory `TimerStore` adapter replaces the `unimplemented!()` stubs in `commands` tests
- [x] Revive lives in `Commands::append_task`; `MarkdownFocusStore::append_task` no longer touches `timer.json`
- [x] Two Tauri timer commands, with `TimerOwner` in `src/types/generated/`
- [x] Existing expiry, notification, revive and task-timer-sidecar tests pass through the new interface
- [x] `with_timer_preset`, `timer_preset()` and `growth_factor` are deleted
- [x] `task check` green

## Blocked by

None. Ships after 055.

## User stories addressed

- "When I change how Timers behave, I change one module, and Focus and Task Timers stay consistent by construction."
