# 044 — Carve domain `TimerTicker` pure module

## Parent PRD

PRD.md §FR7 (timer expiry)

## What to build

Issue 029 (in-flight, scope-expanded) plans a Tokio interval that walks focuses, mutates timer status, persists, emits Tauri events, and fires notifications — all in `src-tauri/src/app/`. That entangles pure expiry-detection logic with the imperative shell. Same anti-pattern as if `cap_state` lived in `tray.rs`.

Land the pure piece **before** 029 so 029 shrinks to wiring.

### `TimerTicker` (`crates/domain/src/timer_ticker.rs`)

```rust
pub struct TimerTransition {
    pub focus_id: String,
    pub focus_title: String,
}

pub fn tick(now_secs: i64, focuses: &[Focus]) -> Vec<TimerTransition>;
```

Rules:

- Returns one `TimerTransition` per `Focus` whose `timer.status == Running` and `started_at + duration_secs <= now_secs`.
- `timer.status == Expired` → skipped (already transitioned).
- `timer.is_none()` → skipped.
- Pure: no I/O, no clock read, no allocation in the hot path beyond the result vector.
- Unlike `OverCapMonitor` (`crates/domain/src/cap_monitor.rs`), which holds a `Mutex<State>` to dedup transitions in-memory, `TimerTicker` is **stateless**: dedup lives in the persisted `timer.status == Expired`. Once a transition fires and the store records `Expired`, subsequent ticks skip the focus naturally.

### Out of scope for this slice

- No interval task.
- No `store.update_timer` call.
- No Tauri event emission.
- No notifier wiring.

All of the above belong to issue 029.

## Completion promise

`TimerTicker::tick(now, focuses) -> Vec<TimerTransition>` exists in `crates/domain` with full table-driven test coverage; no caller wires it yet.

## Acceptance criteria

- [ ] `crates/domain/src/timer_ticker.rs` exposes `tick` and `TimerTransition`
- [ ] `crates/domain/src/lib.rs` re-exports both
- [ ] Table-driven tests cover: no-timer focus skipped, `Running` not yet expired skipped, `Running` exactly at expiry transitioned, `Running` past expiry transitioned, already-`Expired` skipped, multiple transitions in one tick returned in input order
- [ ] No tokio, no async, no I/O imports in the new module
- [ ] `task check` green

## Blocked by

None.

## Unblocks (recommended)

Update issue 029 acceptance criteria to require the expiry detection loop consumes `TimerTicker::tick` rather than re-implementing it inline.

## User stories addressed

- "Expiry detection is unit-testable without a runtime, so future timer rules (snooze, regrowth) can be developed against the pure module."
