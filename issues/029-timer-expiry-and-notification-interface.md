# 029 — Timer expiry + NotificationSource interface

## Parent PRD

PRD.md §FR7 (configuration) + §FR3 (pig UI)

## What to build

Detect when a focus timer expires and introduce a composable notification interface that any subsystem can implement.

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
```

- `Settings` gains `notifications: NotificationSettings`
- `settings.yaml` gains a `notifications:` section; missing keys default to `true`

### Expiry detection

- Background task (or existing tick) calls `timer_remaining_secs` for each focus with a `Running` timer
- On transition to zero: sets `timer.status = Expired`, persists change
- Emits Tauri event `timer-expired` with payload `{ focus_id, focus_title }`
- If `NotificationSettings::is_enabled(TimerExpiredSource)` → fires system notification via Tauri

### `TimerExpiredSource`

```rust
pub struct TimerExpiredSource;
impl NotificationSource for TimerExpiredSource {
    fn key(&self) -> &'static str { "timer_expired" }
    fn label(&self) -> &'static str { "Timer expired" }
}
```

## Completion promise

When a focus timer reaches zero, `TimerStatus` transitions to `Expired`, a Tauri event fires, and a system notification appears (if enabled). Other subsystems can implement `NotificationSource` to plug into the same settings toggle.

## Acceptance criteria

- [ ] `NotificationSource` trait and `NotificationSettings` in `crates/domain/src/notification.rs`
- [ ] `Settings::notifications` field; round-trips through `settings.yaml`
- [ ] `TimerExpiredSource` implements `NotificationSource`
- [ ] Background task detects expiry and sets `TimerStatus::Expired` exactly once per timer
- [ ] Tauri event `timer-expired` emitted with `focus_id` and `focus_title`
- [ ] System notification fires when `timer_expired` source is enabled
- [ ] No notification when source disabled in settings
- [ ] `task check` green

## Blocked by

028 (FocusTimer domain types)
