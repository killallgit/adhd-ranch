# 029 — Timer expiry + unified NotificationSource interface

## Parent PRD

PRD.md §FR7 (configuration) + §FR3 (pig UI)

## What to build

Detect when a focus timer expires, introduce a composable notification interface that any subsystem implements, and migrate the existing cap notifications onto the same interface so the app has **one** notification system.

### `NotificationSource` trait (`crates/domain/src/notification.rs`)

```rust
pub trait NotificationSource {
    fn key(&self) -> &'static str;   // stable identifier used in settings
    fn label(&self) -> &'static str; // human-readable name for settings UI
}

pub struct NotificationSettings {
    pub sources: std::collections::HashMap<String, bool>,
}

impl NotificationSettings {
    pub fn is_enabled(&self, source: &dyn NotificationSource) -> bool {
        *self.sources.get(source.key()).unwrap_or(&true)
    }
}

pub fn all_sources() -> Vec<Box<dyn NotificationSource>> {
    vec![
        Box::new(TimerExpiredSource),
        Box::new(FocusesOverCapSource),
        Box::new(TasksOverCapSource),
    ]
}
```

- `Settings` gains `notifications: NotificationSettings`
- `settings.yaml` gains a `notifications:` section; missing keys default to `true`
- `Alerts` struct is **removed**; `Settings.alerts` field deleted
- `notifications.sources` is the only notification toggle surface

### Concrete sources

```rust
pub struct TimerExpiredSource;
impl NotificationSource for TimerExpiredSource {
    fn key(&self) -> &'static str { "timer_expired" }
    fn label(&self) -> &'static str { "Timer expired" }
}

pub struct FocusesOverCapSource;
impl NotificationSource for FocusesOverCapSource {
    fn key(&self) -> &'static str { "focuses_over_cap" }
    fn label(&self) -> &'static str { "Too many focuses" }
}

pub struct TasksOverCapSource;
impl NotificationSource for TasksOverCapSource {
    fn key(&self) -> &'static str { "tasks_over_cap" }
    fn label(&self) -> &'static str { "Too many tasks in a focus" }
}
```

### Cap notifier migration

- `CapEvaluator::new` takes `NotificationSettings` instead of the full `Settings.alerts`
- In `evaluate()`, gate each call:
  - `focuses_over_cap` / `focuses_under_cap` → `is_enabled(&FocusesOverCapSource)`
  - `task_over_cap` / `task_under_cap` → `is_enabled(&TasksOverCapSource)`
- `under_cap` recovery notifications follow the same per-source toggle (mute the source, mute its recovery too — one switch, one subsystem)

### Expiry detection

- Tokio interval task (1 Hz) in the composition root
- Each tick: `store.list()`, then `let transitions = adhd_ranch_domain::tick(now_secs, &focuses);`
- For each `TimerTransition { focus_id, focus_title }`:
  - Build new timer with `status: Expired`
  - `store.update_timer(&focus_id, &timer)` — single atomic write per timer lifetime
  - Emit Tauri event `timer-expired { focus_id, focus_title }`
  - If `notifications.is_enabled(&TimerExpiredSource)` → fire system notification
- Pure detection logic (which focuses transitioned) lives in `crates/domain::timer_ticker` (issue 044, merged). The interval task here owns I/O + emission only.

### `FocusStore` trait extension

```rust
fn update_timer(&self, focus_id: &str, timer: &FocusTimer) -> Result<(), FocusStoreError>;
```

- `MarkdownFocusStore`: atomic write of `timer.json`
- Test stubs in `commands` crate: implement (mutate cloned focus list)

### `settings.yaml` format

```yaml
notifications:
  timer_expired: true
  focuses_over_cap: true
  tasks_over_cap: true
```

Missing `notifications:` section → all sources default `true`. The old `alerts:` section is no longer parsed; existing users who had `system_notifications: false` re-toggle via Preferences (031). Clean break — single-user app, no migration shim.

## Completion promise

When a focus timer reaches zero, `TimerStatus` transitions to `Expired` (persisted exactly once), a Tauri event fires, and a system notification appears (if enabled). All notifications — timer expiry and cap transitions — flow through one `NotificationSource` interface, controlled by one `notifications.sources` map.

## Acceptance criteria

- [x] `NotificationSource` trait, `NotificationSettings`, `all_sources()` in `crates/domain/src/notification.rs`
- [x] `TimerExpiredSource`, `FocusesOverCapSource`, `TasksOverCapSource` implement `NotificationSource`
- [x] `Alerts` struct deleted; `Settings.alerts` field removed
- [x] `Settings.notifications` field; round-trips through `settings.yaml`
- [x] `FocusStore::update_timer` defined; `MarkdownFocusStore` writes atomically
- [x] `CapEvaluator` consults `NotificationSettings::is_enabled` per source (focuses, tasks gated independently)
- [x] Background tokio interval (1 Hz) consumes `adhd_ranch_domain::tick(now, &focuses)`; for each returned `TimerTransition`, sets `TimerStatus::Expired` exactly once per timer
- [x] Interval task contains no inline expiry arithmetic — detection is delegated to `tick`
- [x] Tauri event `timer-expired` emitted with `focus_id` and `focus_title`
- [x] System notification fires when `timer_expired` source is enabled; suppressed when disabled
- [x] Cap notifications fire/suppress per their respective source toggles
- [x] Existing cap tests adapted and green
- [x] `task check` green

## Blocked by

- 028 (FocusTimer domain types) — done
- 044 (`TimerTicker` pure module) — done
