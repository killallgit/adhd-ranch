# 009 — Edit-proposal modal + empty state polish

## Parent PRD

`PRD.md`

## What to build

Final UX polish before v1 ship. The build/dmg pipeline already exists from slice 002 — this slice is purely UI.

- Edit-proposal modal: small dialog lets the user override target Focus or task text before accepting (`CONTEXT.md` widget flow). On confirm, accept proceeds with the edited payload.
- Empty / first-run state: hero card with `+ New Focus` and one-line tip ("create a bucket; run `/checkpoint` in any session").
- Delete-Focus confirm dialog (referenced by 007) gets a proper styled modal if it's still a `window.confirm` shim.
- Decisions audit log surfaced behind a debug toggle (developer affordance, optional).
- README v1 install + usage docs.

## Completion promise

On `main`, a fresh-install end-to-end demo passes: empty-state hero → create Focus → run `/checkpoint` → edit proposal in modal → accept → bullet visible in widget.

## Acceptance criteria

- [ ] Edit modal lets the user change `target_focus_id` and `task_text` (for `add_task`) or `new_focus.{title,description}` (for `new_focus`) before accepting.
- [ ] Edited accept records the override in `decisions.jsonl` so routing accuracy can be measured (PRD success metric).
- [ ] Empty-state hero shows when no Focuses exist; vanishes once any Focus is present.
- [ ] Delete-Focus confirm is a styled modal, not `window.confirm`.
- [ ] README explains: install dmg, install slash command, run `/checkpoint`, accept proposals.
- [ ] `task check` green.
- [ ] Manual end-to-end: fresh launch on a clean machine → empty state → create Focus → run `/checkpoint` → edit proposal → accept → bullet appears. All flows pass.

## Blocked by

- Blocked by `issues/008-caps-and-overload-alerts.md`

## User stories addressed

- User story 3
