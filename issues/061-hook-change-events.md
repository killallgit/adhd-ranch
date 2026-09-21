# 061 — The change event carries the change

## Parent PRD

PRD.md §FR3 (Pig UI) and §Phase 7, and ADR-0002's third slice.

## What to build

`HookEventSink::apply` works out which session moved, from what to what, and whether it appeared
or vanished — then returns `bool`. Everything downstream re-derives what it threw away:
`HookJournal` re-parses the same payload twice to recover the session id and pen, and
`emit(app, AGENT_SESSIONS_CHANGED_EVENT)` sends the unit type, so the frontend answers it by
re-invoking `list_agent_sessions` for the whole list.

The cost is not the round trip. It is that **no consumer downstream of the socket can see a
transition** — only two snapshots, which is why `usePigMovement` detects roster changes by
hashing a `rosterKey` string. ADR-0001 left `Motion` open for extension; this is what it has to
bind to. "Just went idle" and "is idle" are the same fact today.

### Target shape

`crates/domain/src/session/` gains:

```rust
pub enum SessionChange {
    Appeared(AgentSession),
    Activity { id: String, prompt_id: Option<String>, from: SessionActivity, to: SessionActivity },
    Gone { id: String },
}
```

- `apply` returns `Option<SessionChange>`; `None` is today's `false`.
- `OnChange` becomes `Arc<dyn Fn(SessionChange) + Send + Sync>`; `server/unix.rs` passes it through.
- `Announcing` emits the change as the event payload. The event keeps its `*_EVENT` const so
  `lint:ipc` still sees it.
- `HookJournal` reads the session and pen off the change and only falls back to parsing the
  payload when there is no change to read them from — the unreadable and repeat cases, which are
  exactly the ones the debug window exists for.
- `task gen-types` is re-run and `src/types/generated` committed; the gate fails on drift.

Frontend, without disturbing the other readers:

```ts
export interface PolledReader<T, C = never> {
  read(): Promise<T>;
  subscribe?(onChange: (change: C | null) => void): Unsubscribe | Promise<Unsubscribe>;
  apply?(current: T, change: C): T;
}
```

`usePolledReader` applies the change when both the change and `apply` are present, and refetches
otherwise. The Focus and Settings readers pass neither and behave exactly as they do now. The
first read and every recovery path still go through `invoke`.

### Out of scope

- Any new `Motion`. This slice gives a transition something to hang on; spending it is a later
  slice, and ADR-0002 names `Notification`/`permission_prompt` as the first candidate.
- Registering new hook events.
- Rewriting `usePigMovement`'s `rosterKey` hash. It can go once something better exists; that is
  051's territory.

## Completion promise

A consumer can tell that a session just changed activity, rather than inferring it by comparing
two lists.

## Acceptance criteria

- [ ] `apply` returns `Option<SessionChange>`; no `bool` survives on that path
- [ ] `agent-sessions-changed` carries the change, and `lint:ipc` still resolves the event
- [ ] `SessionChange` is exported by ts-rs and `src/types/generated` is committed
- [ ] The journal parses the payload only when there is no change to read it from
- [ ] Test: an `Appeared`, an `Activity` and a `Gone` each reach the emitter intact
- [ ] Test: a repeat of what the store already holds produces `None`
- [ ] `usePolledReader` applies changes for the session reader and refetches for readers that
      supply no `apply`; the Focus and Settings readers are untouched
- [ ] Test: a dropped or malformed change still leaves the frontend able to recover by reading
- [ ] `task check` and `task check:windows` green

## Blocked by

059 — `prompt_id` comes from the payload work there, and both slices change the same `apply`
signature.

## User stories addressed

- "When my agent finishes a turn, the pig can do something *at that moment*, not just look
  different afterwards."
- "When the debug window shows a firing, the ranch did not parse the same payload three times to
  describe it."
