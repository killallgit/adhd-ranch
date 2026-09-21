# 059 — A hook frame that describes a state at a time

## Parent PRD

PRD.md §FR3 (Pig UI) and §Phase 7, and ADR-0002, which decides that a frame carries a claim about
a state rather than announcing an edge.

## What to build

The wire says `<verb>\n<payload>` and nothing else. There is no time on it, so `LiveSessions`
folds in arrival order and cannot tell a late frame from a current one; two hook processes racing
to `connect()` can invert a `Stop` and a `UserPromptSubmit`, and the session sticks `Working`.

The payload already carries what would fix this and the ranch reads none of it. Claude Code sends
`session_id`, `prompt_id`, `transcript_path`, `cwd`, `permission_mode` and `hook_event_name`;
`agent_session_from_payload` reads two of them. `crates/storage/tests/agent_hook_socket.rs:23`
already puts `hook_event_name` in its fixture, and nothing in production looks at it.

### Target shape

```
adhd-ranch/1 <verb> <epoch-millis>
<payload verbatim>
```

Two refinements on ADR-0002's sketch, both deliberate:

- **Epoch millis, not RFC3339.** The client is dependency-free on purpose — it is spawned several
  times per turn and its cost is what it has to load. `SystemTime::now().duration_since(UNIX_EPOCH)`
  is in std; hand-rolling RFC3339 to be parsed back immediately is not worth a line of code.
- **`hook_event_name` stays in the payload**, not the header. It is already there, and copying it
  up would give the frame two places to disagree.

Work:

- `crates/hook-client/src/main.rs` — stamp at send. Still silent, still exits 0, still no deps.
- `crates/storage/src/agent_hooks/server/unix.rs` — parse the versioned first line. A first line
  with no `adhd-ranch/1` prefix is the legacy form: accept it, stamp it on arrival. During an app
  upgrade `settings.json` can still name an older client, and a dropped firing there is a pig
  that stops moving for no reason the user can see.
- `crates/domain/src/agents/claude_code/hooks/payload.rs` — read `hook_event_name` and `prompt_id`.
- `crates/storage/src/agent_session_store.rs` — hold a per-session last-applied stamp beside the
  `AgentSession` and ignore a frame older than it. `AgentSession` itself does not gain a field:
  it is the projection the overlay draws, and transport bookkeeping is not part of it.
- `crates/domain/src/agents/hooks.rs` — `HookFiring` gains the source event, so the journal stops
  reporting `Stop` and `StopFailure` as the same word.

### Out of scope

- Registering any new hook event. The table stays at five rows; this slice makes a sixth
  affordable, it does not add one.
- Convergence and staleness (060).
- `SessionChange` and the frontend (061).
- The duplicate entries in `~/.claude/settings.json` (062). They are why half of all firings are
  already duplicates, and this slice makes that visible in the journal rather than fixing it.

## Completion promise

A frame that arrives late is ignored rather than applied, and the debug window can tell a turn
that ended from a turn that died.

## Acceptance criteria

- [ ] Confirmed against a live payload that `prompt_id` and `hook_event_name` arrive as
      `docs/research/claude-codex-session-terminology.md` records — ADR-0002 action item 6, and
      this slice is designed around it
- [ ] The client sends `adhd-ranch/1 <verb> <epoch-millis>` and stays dependency-free
- [ ] The server accepts both the versioned and the legacy first line
- [ ] Test: a frame stamped older than the session's last-applied stamp changes nothing
- [ ] Test: `Stop` and `StopFailure` produce distinguishable `HookFiring`s
- [ ] `AgentSession` gains no transport field, and `src/types/generated` is unchanged by it
- [ ] The client still exits 0 and prints nothing when no ranch is listening
- [ ] `task check` and `task check:windows` green

## Blocked by

None. ADR-0002 is Accepted.

## Blocks

060 and 061.

## User stories addressed

- "When my agent finishes a turn, the pig stops — even if two hooks fired at once."
- "When an animal is wrong, the debug window tells me which event the ranch actually heard."
