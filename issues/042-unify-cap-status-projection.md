# 042 — Unify cap-state behind a single `CapStatus` projection

## Parent PRD

PRD.md §FR2 (caps + overload alerting)

## What to build

Cap state has two read paths today:

- `src-tauri/src/app/tray.rs:34,104` calls `cap_state(&focuses, settings.caps).any_over()` directly to drive the badge.
- `crates/commands/src/caps.rs` (`CapEvaluator`) wraps `OverCapMonitor` (`crates/domain/src/cap_monitor.rs`) for transition-based notifications.

Two consumers, two paths, one concept. The badge can show over-cap while the monitor's transition log says under-cap (or vice versa) — there is no shared truth. Apply the deletion test: deleting `OverCapMonitor` would only affect the notifier; the badge keeps working off `cap_state` directly. The seam is real (notifier needs dedup) but only one of two consumers uses it.

Introduce a single `CapStatus` projection that both consumers read. Tray badge and notifier agree by construction.

### `CapStatus` (`crates/domain/src/cap_status.rs`)

```rust
pub struct CapStatus {
    pub focuses_over: bool,
    pub tasks_over_per_focus: Vec<(String /* focus_id */, usize /* task_count */)>,
}

impl CapStatus {
    pub fn from_focuses(focuses: &[Focus], caps: Caps) -> Self { ... }
    pub fn any_over(&self) -> bool { ... }
}
```

- Replaces the ad-hoc `cap_state(...).any_over()` chain in `tray.rs`.
- `OverCapMonitor::observe(&CapStatus)` consumes the same projection and emits `CapTransition`s.

### Tray badge (`src-tauri/src/app/tray.rs`)

- Replace both `cap_state(&focuses, settings.caps).any_over()` calls with `CapStatus::from_focuses(&focuses, settings.caps).any_over()`.
- No behaviour change for badge.

### `CapEvaluator` (`crates/commands/src/caps.rs`)

- `evaluate()` builds `CapStatus` once, hands it to `OverCapMonitor::observe`, then to the notifier for any transitions.
- Notifier receives `CapTransition` (already does); no notifier change.

## Completion promise

Both badge and notifier derive from one `CapStatus` value; over-cap state cannot disagree across surfaces.

## Acceptance criteria

- [ ] `CapStatus` defined in `crates/domain/src/cap_status.rs`
- [ ] `tray.rs` uses `CapStatus::from_focuses(...).any_over()` (no direct `cap_state(...).any_over()` call)
- [ ] `CapEvaluator::evaluate()` builds `CapStatus` once and passes it to the monitor
- [ ] Domain test: badge state and monitor transitions agree across over → under → over sequence (one focus list seeded; assert `CapStatus::any_over` matches the monitor's running state at every step)
- [ ] Existing `caps.rs` tests still pass without behavioural change
- [ ] `task check` green

## Blocked by

None.

## User stories addressed

- "When I cross the focus cap, the tray badge and the system notification reflect the same state — they never diverge."
