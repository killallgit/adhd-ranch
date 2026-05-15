# 053 — Task timer expiry workflow

## Parent PRD

PRD.md §FR3 (AnimalDetail timers) and issue 052.

## What to build

Extend timer expiry handling so Task timers have an explicit product behavior when they reach zero. Today Task timers are persisted and displayed in AnimalDetail, but the background 1 Hz expiry workflow only transitions Focus-level `timer.json` to `Expired` and only Focus timers drive notifications/tray expired state.

Before implementation, decide the Task-timer semantics:

- Whether expired Task timers should persist `status: Expired`.
- Whether Task timer expiry should fire macOS notifications.
- Whether expired Task timers should affect animal rendering, tray state, or only AnimalDetail display.
- Whether adding/updating/completing a Task should clear an expired Task timer.

## Completion promise

Task timers have a defined expiry behavior that is persisted, tested, and reflected consistently in AnimalDetail without confusing Focus-level expired animal behavior.

## Acceptance criteria

- [ ] Product semantics for Task timer expiry are documented in PRD.md and CONTEXT.md.
- [ ] Pure expiry detection covers Task timers without regressing Focus timer detection.
- [ ] Storage can update a Task timer status atomically by Focus ID + Task index.
- [ ] AnimalDetail renders expired Task timers from persisted state, not only elapsed wall-clock inference.
- [ ] Notification behavior is either implemented or explicitly documented as intentionally absent.
- [ ] Tests cover running, expired, missing, deleted-task, and out-of-range Task timer cases.
- [ ] `task check` green.

## Blocked by

052.

## User stories addressed

- "When a Task timer reaches zero, the app behaves predictably and does not leave the timer in a half-running state."
