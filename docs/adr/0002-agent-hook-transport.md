# ADR-0002: Carry state over the agent hook transport, not edges

**Status:** Accepted
**Date:** 2026-09-21 (proposed)
**Accepted:** 2026-09-21
**Deciders:** Ryan (project owner)

## Context

ADR-0001 moved the seam between the two halves down to the renderer and left `Motion`
open for extension on the agent side. This is the other half of that sentence. What the
agent half can *say* is capped by what reaches it, and what reaches it today cannot
express a transition, cannot survive a loss, and cannot be told apart from a duplicate.

### The path a firing takes

```
Claude Code event
  → `adhd-ranch-hook <socket> <verb>`   one process per event, the agent's JSON on stdin
  → AF_UNIX stream                      one connection per firing, one frame: verb \n payload
  → HookAction::from_verb               5 events → 4 verbs
  → Announcing → HookJournal → LiveSessions    BTreeMap<session_id, AgentSession>
  → emit("agent-sessions-changed", ())  the payload is the unit type
  → frontend invoke("list_agent_sessions")     the whole list, every time
```

Every piece of that is defensible on its own. Together they make a transport that only
works because the thing it carries is too simple to break.

### 1. The ranch asks for almost nothing, then throws half of it away

The repo's own research already inventoried the surface
(`docs/research/claude-codex-session-terminology.md`): Claude Code exposes **33 hook
events** across eight categories, and a firing carries a common set of fields —
`session_id`, `prompt_id`, `transcript_path`, `cwd`, `permission_mode`, `hook_event_name`,
plus `agent_id`/`agent_type` inside a subagent.

The ranch registers **5** of those events and reads **2** of those fields.

What gets discarded is not exotic — it is exactly what the agent half is missing:

- **`prompt_id`** — "a UUID v4 identifier linking all events produced while processing a
  single user prompt." A correlation key, on the wire, unread. Every ordering problem
  below is one this field was designed to answer.
- **`hook_event_name`** — the event that fired. `Stop` and `StopFailure` both become the
  verb `idle`, so "the turn died on an API error" is erased *at the client*, before
  anything can decide whether it is worth drawing.
- **`transcript_path`** — a real file, handed to us, whose mtime is a liveness signal for
  a session we would otherwise only hear from when it chooses to speak.
- **`Notification` with `notification_type: permission_prompt`** — "Claude needs approval
  and the prompt has waited about six seconds." This is the single most animation-worthy
  state the agent has, and it is unreachable because the event is not registered.

The information narrows at every hop, and the first hop is the lossy one: 33 events → 5
registered → 4 verbs → 2 `SessionActivity` values → 2 `Motion` values. Two of the four
verbs land on the same state — `Start` and `Idle` both reach `record(…, Idle)` — so the
wire spends distinct words on a distinction the receiver cannot make.

### 2. The wire carries edges, the store holds a level, nothing reconciles them

`HookEventSink::apply(action, payload)` is a fold over arrival order. Each message mutates
the map; the map is the only record. No frame carries a time, a sequence, or a "here is my
whole current state." A lost message is therefore permanent, and so is a reordered pair.

Losses are routine by design, and correctly so — the client runs inside someone's turn and
is silent by construction (`.ok()?` at every step, exit 0). These are all reachable, and
all permanent:

| Cause | Result |
|---|---|
| `agents.enabled` off → on | `clear()` empties the map and the hooks come out. Running sessions are invisible until they next speak — and an idle session never speaks again. |
| A client that connects and stalls | `handle` reads on the accept thread with a 250 ms timeout. The firing is dropped outright, and every firing behind it waits. |
| A payload over `MAX_FRAME` | Truncated mid-JSON, parse fails, firing dropped. `UserPromptSubmit` carries whatever the user just pasted. |
| A missed `Stop`/`StopFailure` | The session stays `Working` for as long as it lives. |
| A missed `SessionEnd` | A ghost animal until the app restarts. |

The design's own defence — nothing is written down, so nothing goes stale — is true of
*disk* and false of memory. The `BTreeMap` is exactly the accumulated state that has to be
reconciled against reality, and nothing reconciles it.

### 3. Every firing is already delivered twice

Not a hypothetical. From `~/Library/Logs/com.adhd-ranch.app/Adhd Ranch.log`:

```
26 firings logged   ·   13 with changed=false   ·   exactly half
```

`client_bin` is derived from `current_exe()`, and the app has been run from two checkouts,
so `~/.claude/settings.json` carries two entry sets pointed at the same socket:

```
SessionStart  '…/adhd-ranch/.claude/worktrees/animals-by-sessions-7d5c3c/target/debug/adhd-ranch-hook' … start
SessionStart  '…/adhd-ranch/target/debug/adhd-ranch-hook'                                              … start
```

All five events carry both. Uninstall matches the command string character for character,
so the entries naming a checkout you no longer run can never be removed by the app that
wrote them.

The doubling is harmless *today* only because `record` is idempotent and `forget` on an
unknown id returns false. That is a property of levels, not of the transport. The moment a
frame means "a turn just ended" — precisely what a richer `Motion` needs — every such edge
fires twice.

Parsing cost tells the same story: `LiveSessions` parses the payload once, then
`HookJournal` parses it twice more to recover the session id and pen that `LiveSessions`
had already computed. Three parses per firing, six per real event.

### 4. `apply` computes the change and discards it at the return statement

`apply` returns `bool`. It knows which session moved, from what to what, and whether it
appeared or vanished — and returns one bit. Everything downstream re-derives what it threw
away: the journal re-parses the payload, and the frontend refetches the entire session list
to find out what moved.

`emit(app, AGENT_SESSIONS_CHANGED_EVENT)` sends `()`. So a push transport terminates in a
notify-then-refetch, and every consumer sees only states, never changes. `usePigMovement`
already pays for this, detecting roster changes by hashing a `rosterKey` string.

**This is the cap on ADR-0001.** That ADR says `Motion` is open for extension and the Focus
half need not change. True at the type level, and empty in practice: nothing downstream has
a transition to bind a new `Motion` to. Anything with a duration, a one-shot, or a
direction — "just went idle" as against "is idle" — has no data to compute from.

### 5. The closed event table is a transport decision in a domain costume

`events.rs` refuses `PreToolUse`/`PostToolUse` because they "report a state the session is
already in." That is correct *for a two-state model*, and whether the model stays
two-state is the entire question. Under this transport, admitting them costs one process,
one connection, one full-list IPC and one re-render per tool call. So the vocabulary
ceiling and the transport cost are the same constraint, and the comment's closing promise
— "an agent's event table then gains a row rather than a rewrite" — holds only for rows
that fit the two states already there.

### Not in question

Push over polling. A socket with no on-disk state. A dependency-free client that fails
silently and fast. Mode `0600` as the whole authorisation story. The decorator chain, which
is the one place this design already separated concerns properly. Stale-socket reclaim.
None of this changes.

## Decision

**Make each frame describe a state rather than announce an edge, give the store a way to
notice what it has lost, and let the change event say what changed.** Three slices, each
shippable and worth having alone.

**1 — The frame is self-describing.** It carries the source event name and the moment the
client sent it, alongside the verb:

```
adhd-ranch/1 <verb> <rfc3339-stamp> <hook_event_name>
<payload verbatim>
```

The store resolves by stamp rather than arrival order, and drops a frame older than what it
already holds for that session. The client is a fresh process with no memory, so it cannot
hold a sequence — but two hook processes started in emission order by the same parent will
stamp in that order, which is strictly better than arrival order and costs one syscall.
`prompt_id` comes out of the payload for free and correlates a turn's events.

This changes the wire, not the command string, so no hook entry is orphaned by it.

**2 — The store converges.** Two different problems, and only the first is worth solving now:

- *Stuck state* — we hold the session and its level is wrong. The payload hands us
  `transcript_path`; keep it, and on a slow sweep check whether the file has moved. A
  session we believe is `Working` whose transcript has been still for minutes is not
  working. This reads a path the agent gave us, not a layout we reverse-engineered.
- *Cold start* — we never heard of the session at all, because we were not listening when
  it began. Solving this needs a census of `~/.claude/projects/<cwd-slug>/<id>.jsonl`,
  which is an undocumented private layout. Deferred, and noted under revisit.

On push-not-poll: push stays the transport. This is a repair pass off the hot path, run at
start-of-listening and on a slow cadence. It is not a marker file we invented and asked the
agent to write to — the distinction that principle was about.

**3 — The change event carries the change.**

```rust
enum SessionChange {
    Appeared(AgentSession),
    Activity { id: String, prompt_id: Option<String>, from: SessionActivity, to: SessionActivity },
    Gone { id: String },
}
```

`apply` returns this instead of `bool`; `Announcing` emits it; the journal stops re-parsing
to recover what it was handed. The frontend keeps `invoke` for the first read and for
recovery, and applies changes in between — so a one-shot `Motion` finally has an edge to
hang on.

## Options Considered

### Option A: Status quo — add rows to the event table as needs arrive

| Dimension | Assessment |
|---|---|
| Effort now | None |
| Fixes drops / reordering | No |
| Unblocks richer `Motion` | No |
| Duplicate-safe | Only while every frame is idempotent |
| Reversibility | Decreasing — each new row inherits the same transport |

**Pros:** free; the two-state model genuinely does survive on this transport today.
**Cons:** every reachable failure in §2 stays permanent; the first non-idempotent event
turns the live duplicate delivery from invisible into visible; ADR-0001's open `Motion` has
nothing to consume it.

### Option B: Harden the frame and converge (recommended)

| Dimension | Assessment |
|---|---|
| Effort now | Medium — 3 slices, Rust plus a small frontend change |
| Blast radius | `hook-client`, `server/unix.rs`, `LiveSessions`, `HookJournal`, `Announcing`, `tauriReader` |
| Fixes drops / reordering | Reordering yes; stuck state yes; cold start deferred |
| Unblocks richer `Motion` | Yes — directly |
| Reversibility | High — no storage, no settings-file change, the command string is untouched |

**Pros:** keeps everything the design got right; each slice ships alone; uses fields that
are already on the wire rather than inventing a protocol; does not orphan a single hook
entry.
**Cons:** a versioned frame is a compatibility surface that did not exist before; the
staleness sweep needs a threshold that will be wrong the first time.

### Option C: Hooks become a doorbell; the transcript is the truth

Every firing says only "something changed, go look." The ranch reads Claude Code's own
transcripts for state.

| Dimension | Assessment |
|---|---|
| Effort now | High |
| Fixes drops / reordering | Completely — convergent by construction |
| Unblocks richer `Motion` | Yes, and per-tool granularity comes free |
| Reversibility | Low — the reader becomes load-bearing |

**Pros:** no dropped frame can cause a permanent wrong state; the transcript already
records every tool call, so the event table stops being the ceiling.
**Cons:** the docs call the transcript format "internal to Claude Code and changes between
versions." And the decisive one: transcripts are the user's conversation. The journal
already took care not to keep `UserPromptSubmit`'s text; reading whole transcripts to drive
an animation walks that back and pulls prompt content into the ranch's process to decide
which way a pig should face. Rejected on that alone.

### Option D: One long-lived connection per session

The client daemonizes per session and holds a stream, giving ordering for free and a broken
stream as an end-of-session signal.

| Dimension | Assessment |
|---|---|
| Effort now | Very high |
| Fixes drops / reordering | Ordering yes; a wedged daemon is a new failure mode |
| Unblocks richer `Motion` | Yes |
| Reversibility | Low |

**Pros:** ordering and liveness become structural rather than inferred.
**Cons:** Claude Code's hook contract *is* process-per-event, so the spawn does not go away
— only the `connect` does, which is not what costs. In exchange it reintroduces exactly the
lifecycle state this design is proud of not having: daemons to reap, orphans after a crash,
and a second thing that can be wedged. Rejected.

## Trade-off Analysis

The real axis is **what a frame means**. A is "a frame is an edge, and edges are never
lost" — false, and only survivable while every edge happens to be idempotent. C is "a frame
means nothing; go read the truth elsewhere" — correct and convergent, at the price of
reading the user's conversation to animate a sprite. B is "a frame is a claim about a state
at a time, and the store is allowed to disbelieve it" — which is what the wire can actually
support.

B is also the only option that is *cheap because of work already done*. `prompt_id`,
`hook_event_name` and `transcript_path` are already in every payload; `apply` already
computes the change; the decorator chain already has the right shape to carry it. Most of
slice 3 is deleting a discard.

The honest cost is that B fixes reordering and stuck state but not cold start. A session
running before the ranch started listening stays invisible until it speaks. That is the
same hole as today, and B does not widen it — but it does not close it either, and the only
thing that closes it is the private-layout census this ADR declines to buy.

## Consequences

**Easier**
- A `Motion` with a duration or a one-shot becomes expressible — the thing ADR-0001
  promised at the type level and the transport withheld
- `Stop` and `StopFailure` stop being the same word, so "the turn died" is drawable
- The debug window can tell "heard and ignored" from "heard and could not read"
- Registering a richer event stops requiring a two-state justification
- Duplicate delivery becomes detectable rather than merely survivable

**Harder**
- A versioned frame means a client and a ranch from different builds can disagree, which
  they cannot today
- The staleness sweep introduces a clock into a half that ADR-0001 was pleased to note had
  none — a narrow one, for liveness only, never for growth or expiry
- Three parses per firing becomes one, but the frame gains a header to parse

**To revisit**
- Whether the cold-start census is worth its coupling, once slices 1–3 have shipped
- Whether `Notification`/`permission_prompt` earns its own `Motion` — the strongest
  candidate for the first state that is neither working nor resting
- Whether subagents get their own animal, which is the point at which `SubagentStart`/
  `SubagentStop` and `agent_id` stop being noise
- Whether a second harness (Codex, whose hook payloads copy these field names and add
  `turn_id`) fits this frame unchanged

## Action Items

1. [x] Issue 059: versioned frame carrying stamp and `hook_event_name`; store resolves
       by stamp; journal records the source event, not just the verb.
2. [x] Issue 060: keep `transcript_path` per session; slow sweep downgrades a `Working`
       session whose transcript has been still past a threshold.
3. [x] Issue 061: `apply` returns `SessionChange`; `agent-sessions-changed` carries it;
       the frontend applies changes instead of refetching the list.
4. [x] Issue 062: stop deriving `client_bin` from `current_exe()`, and reclaim the
       orphaned entry sets already in `~/.claude/settings.json`. Independent of the frame
       work, and the only item here with a visible bug attached.
5. [ ] Add **Hook Firing** and **Session Activity** to CONTEXT.md's **Agents** section; the
       glossary names neither, and ADR-0001 showed what an unnamed concept turns into.
6. [ ] Confirm against a live payload that `prompt_id` and `hook_event_name` arrive as the
       research doc records, before slice 1 designs around them. Carried as 059's first
       acceptance criterion.
7. [ ] Revisit the cold-start census after slice 2 ships.
