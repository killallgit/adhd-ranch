# 028 — Focus timer: domain types + full-stack creation

## Parent PRD

PRD.md §FR3 (pig UI) + §FR7 (configuration)

## What to build

Add timer support to focuses end-to-end: domain types, persistence, and the creation UI.

### Domain (`crates/domain/src/timer.rs`)

```rust
pub enum TimerStatus { Running, Expired }

pub enum TimerPreset { Two, Four, Eight, Sixteen, ThirtyTwo, Custom(u64) }
// duration_secs(): 120, 240, 480, 960, 1920, custom value

pub struct FocusTimer {
    pub duration_secs: u64,
    pub started_at: i64,   // unix timestamp (secs)
    pub status: TimerStatus,
}

pub fn timer_remaining_secs(timer: &FocusTimer, now_secs: i64) -> Option<u64>
// Returns None when expired, Some(remaining) while running.

pub fn pig_scale(elapsed_secs: u64, duration_secs: u64) -> f32
// Returns 1.0..=3.0 clamped. Linear interpolation over full timer window.
```

- `Focus` gains `timer: Option<FocusTimer>`
- `NewFocus` gains `timer_preset: Option<TimerPreset>`
- Both serialise/deserialise correctly through `settings.yaml` and the HTTP API

### Backend

- `create_focus` command/handler accepts `timer_preset: Option<TimerPreset>`
- When present: constructs `FocusTimer { started_at: now, duration_secs, status: Running }`
- Timer persisted alongside focus

### Frontend

- `NewFocusForm` gains a timer dropdown: **None / 2m / 4m / 8m / 16m / 32m / Custom**
- Custom shows a number input (minutes)
- Selected preset sent with focus creation request

## Completion promise

User can create a focus with a timer preset; the timer is stored and readable via the API with correct remaining time.

## Acceptance criteria

- [ ] `TimerPreset`, `FocusTimer`, `TimerStatus` defined in `crates/domain/src/timer.rs`
- [ ] `timer_remaining_secs` returns correct value at t=0, mid-duration, and past expiry
- [ ] `pig_scale` returns 1.0 at t=0, ~2.0 at half duration, 3.0 at/past full duration
- [ ] Unit tests cover both pure functions including boundary values
- [ ] `Focus` and `NewFocus` carry optional timer fields; round-trips through serde
- [ ] `NewFocusForm` renders timer dropdown; Custom shows number input
- [ ] Creating a focus with a preset stores a `FocusTimer` with correct `started_at` and `duration_secs`
- [ ] `task check` green

## Blocked by

None — can start immediately.
