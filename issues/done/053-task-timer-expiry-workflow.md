# 053 — Task timer expiry workflow

## Parent PRD

PRD.md §FR3 (AnimalDetail timers) and issue 052.

## What to build

Extend timer expiry handling so Task timers have an explicit product behavior when they reach zero. Before this slice, Task timers were persisted and displayed in AnimalDetail, but the background 1 Hz expiry workflow only transitioned Focus-level `timer.json` to `Expired` and only Focus timers drove notifications/tray expired state.

Implemented Task-timer semantics:

- Expired Task timers persist `status: Expired`.
- Task timer expiry emits through a platform-neutral notification sink when the `task_timer_expired` source is enabled.
- Expired Task timers affect only `AnimalDetail` display and the task timer notification source; they do not affect animal rendering, tray expired state, or Focus timer status.
- Adding/updating/completing a Task does not clear an expired Task timer. Deleting a Task removes its indexed timer entry; users can clear or restart the timer explicitly.

## Completion promise

Task timers have a defined expiry behavior that is persisted, tested, and reflected consistently in AnimalDetail without confusing Focus-level expired animal behavior.

## Acceptance criteria

- [x] Product semantics for Task timer expiry are documented in PRD.md and CONTEXT.md.
- [x] Pure expiry detection covers Task timers without regressing Focus timer detection.
- [x] Storage can update a Task timer status atomically by Focus ID + Task index.
- [x] AnimalDetail renders expired Task timers from persisted state, not only elapsed wall-clock inference.
- [x] Notification behavior is implemented through the platform-neutral notification sink.
- [x] Tests cover running, expired, missing, deleted-task, and out-of-range Task timer cases.
- [x] `task check` green.

## Blocked by

052

## User stories addressed

- "When a Task timer reaches zero, the app behaves predictably and does not leave the timer in a half-running state."
