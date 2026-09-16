# 056 — Task Timers drift when Tasks are hand-edited

## Parent PRD

PRD.md §FR3 (Task timers), issues 052 and 053. Raised while designing 054.

## What to build

`task-timers.json` is a JSON array whose index matches the Task's position in `focus.md`. The app keeps the two in step: `delete_task` removes the matching entry. A user editing `focus.md` by hand does not. Inserting a bullet above a timed Task, or reordering bullets, silently moves a Timer onto a different Task.

Hand-editing is a supported way to use the app, so the binding should survive it. Nothing tests this today.

Options to weigh:

- Store each Timer's Task text alongside it and re-match on load, falling back to position when text is ambiguous or changed.
- Give Tasks stable ids in the markdown. Changes the document format and what "plain text only" means.
- Accept the drift and document it.

### Out of scope

- 054, which keeps position matching as-is.

## Completion promise

A Task Timer stays attached to the Task the user attached it to, across hand edits that reorder or insert Tasks — or the drift is a documented, tested behaviour.

## Acceptance criteria

- [ ] A decision is recorded (ADR if the format changes)
- [ ] Tests cover reordering, inserting above a timed Task, and editing a timed Task's text
- [ ] `task check` green

## Blocked by

054

## User stories addressed

- "When I reorder my tasks in the markdown by hand, my timers stay on the right tasks."
