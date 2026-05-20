# 052 — Task timers and clock dropdown editing

## Parent PRD

PRD.md §FR3 (Pig UI / AnimalDetail) and §FR7 (Configuration timers).

## What shipped

Users can edit timers by clicking the clock/time control itself. Focus timers remain global to the Focus, while each Task can now carry its own independent timer.

Implementation notes:

- `Task` gains optional `timer: Option<FocusTimer>`.
- Focus timers continue to persist in `timer.json`.
- Task timers persist in `task-timers.json`, indexed to the parsed Task order.
- `Commands` and Tauri bridge expose start/clear operations for Focus timers and Task timers.
- `TimerDropdown` consolidates timer editing behind the clock/time control.
- `PigDetail` was renamed to `AnimalDetail`; CSS/test IDs now use `animal-detail`.

## Completion promise

AnimalDetail exposes compact clock/time dropdowns for Focus and Task timers, and Task timer state persists across restarts without changing the markdown Task syntax.

## Acceptance criteria

- [x] A Focus can still have one global timer.
- [x] Each Task can have its own independent timer.
- [x] No timer renders as a compact clock control.
- [x] Existing timers render as current time/expired status and open the same dropdown on click.
- [x] Timer dropdown supports preset and custom-minute starts/restarts.
- [x] Timer dropdown supports clearing existing timers.
- [x] Task timer storage does not alter `focus.md` checkbox syntax.
- [x] Deleting a Task removes the matching task timer entry.
- [x] `AnimalDetail` naming replaces `PigDetail` for the clicked-animal detail card.
- [x] Tests cover storage, commands, IPC wrappers, and AnimalDetail timer UI.

## Known follow-up

Task timers are persisted and displayed, but only Focus timers currently participate in the background expiry/notification workflow. See issue 053.

## Validation

- `task test`
- `npm run lint`
- `npm run typecheck`
- `npm run test -- AnimalDetail App`
