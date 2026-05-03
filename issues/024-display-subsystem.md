# 024 — Display subsystem: coordinate bugs + module refactor

## Parent PRD

PRD.md §FR2 (transparent overlay window)

## Problem

Multi-monitor spanning has never worked correctly. Confirmed runtime behaviour:

1. Single monitor selection works — pig appears on the selected monitor.
2. With both selected, pigs hit a **barrier in the middle** (not a monitor edge) when walking or being dragged.
3. Pigs **walk off the right edge** of one display and **off the left edge** of the other — disappear, no boundary.
4. Display names in tray are **identical** — no disambiguation.
5. Hardware includes a **270° rotated portrait monitor** to the left of the main display.

## Root cause

### Mixed-unit bounding box

`overlay_manager::apply` mixes logical positions with physical sizes:

```rust
// WRONG — position is logical (macOS points), size is physical pixels
let max_x = enabled.iter().map(|m| m.position.x + m.size.width as i32).max();
```

At 2× DPR: `position.x = 1920` (logical) + `size.width = 3840` (physical) = `5760` (nonsense).
Window ends up smaller than the actual span and offset inside it → open outer edges, barrier in the middle.

### Rotated monitor

270°-rotated monitor reports swapped logical dimensions and may anchor position at a non-top-left corner. Current code ignores orientation entirely.

### Hit-test thread mismatch

`outer_position()` may return logical, `cursor_position()` returns physical. `local_x = cursor.x - origin.x` is in mixed units → click-through fires at wrong positions.

### Single file doing too much

`overlay_manager.rs` conflates monitor math, window lifecycle, hit-test state, and polling threads.

## What to build

Replace `app/overlay_manager.rs` + `app/pig_hittest.rs` with a `display/` module tree:

```text
src-tauri/src/display/
  mod.rs        — DisplayManager (public surface; re-exports; impl RectUpdater)
  monitor.rs    — pure types + math: LogicalMonitor, compute_span, disambiguate_names
  overlay.rs    — window lifecycle: create, resize, show, destroy
  hit_test.rs   — PigHitTester + polling thread
```

### `display/monitor.rs` (pure, testable)

```rust
pub struct LogicalMonitor {
    pub index: usize,
    pub label: String,       // disambiguated
    pub scale_factor: f64,
    pub position: (f64, f64),  // logical pixels, top-left
    pub size: (f64, f64),      // logical pixels
}

pub struct SpanBounds { pub x: f64, pub y: f64, pub width: f64, pub height: f64 }

pub fn compute_span(monitors: &[LogicalMonitor]) -> SpanBounds;
pub fn disambiguate_names(monitors: &mut [LogicalMonitor]);
```

All arithmetic in logical pixels. Window calls use `LogicalSize`/`LogicalPosition` — Tauri handles physical conversion per DPR.

### Key fixes

- `LogicalMonitor::from_tauri`: divide `m.size()` and `m.position()` by `m.scale_factor()` to get true logical values.
- `compute_span`: union bounding box entirely in logical space.
- Hit-test thread: convert `outer_position()` to logical before subtraction from (logical) cursor position.
- `disambiguate_names`: append `" (x, y)"` when two monitors share a name.

### What moves / stays

| Symbol | From | To |
|---|---|---|
| `PigHitTester` | `app/pig_hittest.rs` | `display/hit_test.rs` |
| `OverlayManager` | `app/overlay_manager.rs` | `display/mod.rs` as `DisplayManager` |
| `MonitorInfo` | `app/overlay_manager.rs` | `display/monitor.rs` as `LogicalMonitor` |
| Bounding-box math | `app/overlay_manager.rs` | `display/monitor.rs::compute_span` |
| Window lifecycle | `app/overlay_manager.rs` | `display/overlay.rs` |
| `MonitorsState` | `app/mod.rs` | stays, wraps `Vec<LogicalMonitor>` |
| `DisplayConfigState` | `app/mod.rs` | stays |
| Tray display menu | `app/tray.rs` | stays, uses `LogicalMonitor.label` |
| `PigHitState`, `update_pig_rects` | `ui_bridge/mod.rs` | stays |
| `DisplayConfig`, `PigRect`, `RectUpdater` | `crates/domain` | stays |

## Completion promise

With two monitors enabled (including a rotated portrait monitor), pigs roam the full logical span with no barrier in the middle and no open outer edges.

## Acceptance criteria

- [ ] `display/` module tree exists; `app/overlay_manager.rs` + `app/pig_hittest.rs` deleted
- [ ] `compute_span` and `disambiguate_names` covered by unit tests (6 cases)
- [ ] Both monitors enabled: pigs roam full span, no barrier, no open edges
- [ ] Portrait/rotated monitor included correctly in the span
- [ ] Monitor names disambiguated in tray
- [ ] Click-through hit-testing works across the full spanning window
- [ ] Drag works anywhere on the spanning window
- [ ] App launches at runtime without crash
- [ ] `task check` green

## Blocked by

None.
