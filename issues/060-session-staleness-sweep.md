# 060 — A session that stops speaking stops working

## Parent PRD

PRD.md §FR3 (Pig UI) and §Phase 7, and ADR-0002's second slice: the store gets a way to notice
what it has lost.

## What to build

`LiveSessions` believes the last thing it was told, forever. A missed `Stop` or `StopFailure`
leaves a session `Working` for as long as it lives, and the only cure is restarting the app.
Every drop path is reachable by design — the client is silent by construction, `handle` drops a
firing whose client stalls past the 250 ms read timeout, and a payload over `MAX_FRAME` is
truncated into a parse failure.

Claude Code hands us the repair in the payload. `transcript_path` is a real file that grows while
a turn runs. A session we believe is `Working` whose transcript has been still for minutes is not
working.

### Target shape

- `LiveSessions` keeps `transcript_path` per session, alongside the stamp 059 added. Not on
  `AgentSession` — it is a path into the user's conversation history and has no business crossing
  into the overlay's types.
- A sweep, off the hot path, downgrades `Working` → `Idle` for any session whose transcript has
  not moved in `STALE_AFTER`. Start at 5 minutes and say why in the const: a turn appends as it
  runs, so stillness that long is a turn that is over.
- The clock and the mtime lookup are injected, as `HookJournal` already injects its `Clock`. The
  sweep's tests need no real filesystem and no real time.
- A downgrade is a change, so it goes through the same path a firing does and the overlay
  updates.

### What this deliberately does not fix

**Ghost sessions.** Transcripts survive the session that wrote them — that is what `--resume`
reads — so a missing file is not evidence a session ended. A missed `SessionEnd` therefore still
leaves an idle animal until the app restarts. A walking pig that never stops is the visible bug;
a resting pig that overstays is not, and inventing a reaping rule for it would be guessing.

**Cold start.** A session running before the ranch started listening stays invisible until it
speaks. Closing that needs a census of Claude Code's private transcript layout, which ADR-0002
declined to buy and parked under revisit.

### On push-not-poll

Push stays the transport. This is a repair pass on a slow cadence, reading a path the agent
handed us — not a marker file we invented and asked it to write to, which is what that rule is
about.

### Out of scope

- Removing sessions on any staleness rule.
- Reading transcript *contents*. The journal already takes care not to keep `UserPromptSubmit`'s
  text; this slice stats a file and never opens it.
- The cold-start census.

## Completion promise

A session whose turn ended without the ranch hearing about it stops being drawn as working,
without the app restarting.

## Acceptance criteria

- [ ] `transcript_path` is held inside `LiveSessions` and appears on no exported type
- [ ] Test: a `Working` session whose transcript is older than `STALE_AFTER` becomes `Idle`
- [ ] Test: a `Working` session whose transcript is still moving is left alone
- [ ] Test: an `Idle` session is never touched by the sweep
- [ ] Test: a session with no readable `transcript_path` is left alone, not downgraded
- [ ] The sweep takes its clock and its mtime lookup as parameters; no test reads a real clock
- [ ] The sweep never runs on the accept thread
- [ ] A downgrade reaches the overlay the same way a firing does
- [ ] `task check` and `task check:windows` green

## Blocked by

059 — which introduces the per-session bookkeeping this extends, and reads the payload fields it
needs.

## User stories addressed

- "When a turn dies quietly, the pig settles down instead of walking forever."
- "When I have been away, what the ranch shows me is not just the last thing it happened to hear."
