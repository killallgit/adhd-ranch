# 005 — `/checkpoint` slash command + proposal queue

## Parent PRD

`PRD.md`

## What to build

Wire the in-session capture path end-to-end. The slash command at `~/.claude/commands/checkpoint.md` instructs the agent to read the catalog, decide a kind (`add_task` | `new_focus` | `discard`), and POST a proposal. App appends to `~/.adhd-ranch/proposals.jsonl` and exposes pending proposals via `GET /proposals`. Widget shows a collapsed `📥 N pending` tray with the badge count; expanding shows summary + suggested target + reasoning behind `?`. Accept/reject buttons render but are no-ops (next slice).

See `PRD.md` FR4 (`/proposals` routes), FR5 (slash command), `CONTEXT.md` single-stage flow.

Proposal storage behind a `ProposalQueue` trait. JSONL append is atomic.

## Acceptance criteria

- [ ] `~/.claude/commands/checkpoint.md` installed by `task install-skill` (or equivalent task target); contents documented.
- [ ] Slash command errors clearly when port file missing or `/health` fails: "adhd-ranch not running, please start the app".
- [ ] `POST /proposals` validates payload shape, appends to `proposals.jsonl`, returns the proposal id within 50ms (PRD NFR).
- [ ] `GET /proposals` returns pending proposals.
- [ ] Widget shows collapsed pending tray with badge count; expanding shows summary, target Focus title, and reasoning behind `?`.
- [ ] Run `/checkpoint` in a real Claude Code session → proposal appears in widget within 1s.
- [ ] Integration test posts a proposal and asserts it appears in `GET /proposals` and on disk.
- [ ] `task check` green.

## Blocked by

- Blocked by `issues/004-http-api-skeleton.md`

## User stories addressed

- User story 2
- User story 3
