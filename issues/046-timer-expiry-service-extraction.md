# 046 — Extract timer-expiry workflow out of `app/`

## Parent PRD

CLAUDE.md §Data/view separation — `app/` is the composition root that wires everything in `main.rs`; not domain workflow.

## What to build

`src-tauri/src/app/timer_expiry.rs` (added in issue 029) owns more than wiring. `run_once` walks `store.list()` → `tick()` → `store.update_timer()` → emit Tauri event → fire system notification per `NotificationSource` toggle. That is a workflow, not composition.

Move the workflow into a dedicated module/service. `app/` should only own the spawn site that injects dependencies and starts the loop.

### Target shape

```rust
// crates/commands/src/timer_expiry.rs (or new crate)
pub trait TimerExpiryNotifier: Send + Sync {
    fn emit_expired(&self, focus_id: &str, focus_title: &str);
    fn notify_expired(&self, focus_title: &str);
}

pub struct TimerExpiryService {
    store: Arc<dyn FocusStore>,
    notifier: Arc<dyn TimerExpiryNotifier>,
    notifications: Arc<dyn Fn() -> NotificationSettings + Send + Sync>,
    clock_secs: Arc<dyn Fn() -> i64 + Send + Sync>,
}

impl TimerExpiryService {
    pub fn tick(&self) -> Result<(), CommandError> { /* … */ }
}
```

- Tauri-specific bits (`AppHandle`, `Emitter`, `tauri_plugin_notification`) live in a thin `TauriTimerExpiryNotifier` in `src-tauri/src/app/` — same pattern as `TauriCapNotifier`.
- The async interval loop stays in `app/timer_expiry.rs`, but reduces to `spawn_blocking(|| service.tick())`.

### Why now

CodeRabbit flagged this on PR #53 (https://github.com/killallgit/adhd-ranch/pull/53). Deferred to keep that PR scoped to issue 029.

## Acceptance criteria

- [ ] `TimerExpiryService` exists in `crates/commands` (or a new dedicated crate); takes a `TimerExpiryNotifier` trait + `FocusStore` + clock + notification-settings provider via constructor injection.
- [ ] `TauriTimerExpiryNotifier` in `src-tauri/src/app/` implements `TimerExpiryNotifier` using `AppHandle::emit` + `tauri_plugin_notification`.
- [ ] `src-tauri/src/app/timer_expiry.rs` reduces to: spawn 1Hz interval, call `spawn_blocking(|| service.tick())`. No inline tick→persist→emit→notify logic.
- [ ] Service has unit tests using a recording notifier + in-memory `FocusStore` stub (mirrors `CapEvaluator` tests).
- [ ] `task check` green.

## Blocked by

- 029 (timer-expiry interface) — done

## Hands off to

None.
