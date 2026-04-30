# 006 — Accept/reject proposals: mutation + decisions audit

## Parent PRD

`PRD.md`

## What to build

Make proposal accept/reject actually mutate state. `POST /proposals/{id}/accept` dispatches by `kind`:

- `add_task` → append `- [ ]` bullet to target Focus body.
- `new_focus` → create `~/.adhd-ranch/focuses/<slug>/focus.md` with frontmatter + empty body.
- `discard` → no mutation.

`POST /proposals/{id}/reject` drops the proposal with no mutation. Both paths append a row to `~/.adhd-ranch/decisions.jsonl` with `{ts, proposal_id, decision, reasoning, target}` (FR8).

Mutation uses atomic write (tmpfile + rename) and `flock` per file (FR-NFR reliability). Each mutation kind is a separate strategy/handler — programs to a `ProposalApplier` trait.

Widget `✓ / ✗` buttons in the proposals tray now wired to the endpoints.

## Acceptance criteria

- [ ] Accepting `add_task` appends a bullet to the target Focus; widget reflects within 1s via watcher.
- [ ] Accepting `new_focus` creates a new Focus dir + `focus.md` with valid frontmatter and the proposed title/description.
- [ ] Accepting `discard` removes the proposal without mutation.
- [ ] Reject removes the proposal without mutation.
- [ ] Every accept/reject appends a row to `decisions.jsonl`; rows parseable.
- [ ] Atomic writes verified by a test that simulates a crash mid-write (no partial file visible to readers).
- [ ] Each kind dispatched via a strategy trait, not a `match` in the handler.
- [ ] `task check` green.

## Blocked by

- Blocked by `issues/005-checkpoint-slash-command-and-proposals.md`

## User stories addressed

- User story 2
- User story 3
