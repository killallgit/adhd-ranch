# Agent prompt — pick up issue 024

You are implementing issue **024 — Display subsystem: coordinate bugs + module refactor** in the `adhd-ranch` Tauri desktop app at the repository root.

---

## Your job

Replace the broken `app/overlay_manager.rs` + `app/pig_hittest.rs` with a clean `display/` module tree that fixes multi-monitor coordinate bugs. The spec is in `issues/024-display-subsystem.md` — read it first. The workflow rules are in `issues/README.md`. Programming rules are in `CLAUDE.md`. All must be followed.

---

## Context you need

### Why this exists

Multi-monitor spanning has never worked. Root cause: `overlay_manager::apply` mixes **logical positions** (macOS reports `Monitor::position()` in points) with **physical sizes** (`Monitor::size()` in raw pixels). At 2× DPR, `position.x = 1920` (logical) + `size.width = 3840` (physical) = `5760` — a nonsense value. The window is smaller than the actual display span and positioned inside it, so pigs hit a false barrier in the middle and walk off the true outer edges. The hit-test polling thread has the same mismatch. A 270°-rotated portrait monitor is also not handled.

### Current code map

Read these files before touching anything:

| File | What it does |
|---|---|
| `src-tauri/src/app/overlay_manager.rs` | Window creation, bounding-box calc, hit-test thread — all in one file. **Delete this.** |
| `src-tauri/src/app/pig_hittest.rs` | `PigHitTester` struct + `is_hit`. **Move to `display/hit_test.rs`.** |
| `src-tauri/src/app/mod.rs` | Composition root. Enumerates monitors, wires state. Update imports only. |
| `src-tauri/src/app/tray.rs` | Builds display submenu. Uses `MonitorInfo` name field. Update to use `LogicalMonitor.label`. |
| `src-tauri/src/ui_bridge/mod.rs` | `PigHitState` + `update_pig_rects` command. **Do not touch.** |
| `crates/domain/src/` | `DisplayConfig`, `PigRect`, `RectUpdater` trait. **Do not touch.** |

### New module tree to create

```text
src-tauri/src/display/
  mod.rs        — DisplayManager (public surface; replaces OverlayManager; impl RectUpdater)
  monitor.rs    — pure types + math: LogicalMonitor, compute_span, disambiguate_names
  overlay.rs    — window lifecycle: create, resize, show, destroy
  hit_test.rs   — PigHitTester + polling thread (move from app/pig_hittest.rs)
```

Add `mod display;` in `src-tauri/src/lib.rs` or `main.rs`.

### The coordinate fix (critical)

`LogicalMonitor::from_tauri` must divide physical values by `scale_factor`:

```rust
pub fn from_tauri(index: usize, m: &tauri::Monitor) -> Self {
    let sf = m.scale_factor();
    Self {
        index,
        label: m.name().unwrap_or_default().to_string(),
        scale_factor: sf,
        position: (m.position().x as f64 / sf, m.position().y as f64 / sf),
        size: (m.size().width as f64 / sf, m.size().height as f64 / sf),
    }
}
```

`compute_span` works entirely in logical space and returns `SpanBounds { x, y, width, height }` (all `f64` logical).

Window calls use `LogicalSize` / `LogicalPosition` so Tauri handles physical conversion internally:

```rust
window.set_size(tauri::LogicalSize::new(bounds.width, bounds.height))?;
window.set_position(tauri::LogicalPosition::new(bounds.x, bounds.y))?;
```

Hit-test thread: `outer_position()` returns physical — divide by `window.scale_factor()` to get logical before subtracting from cursor. OR use `outer_position().to_logical(sf)`.

### What the DisplayManager must do

- Same public interface as current `OverlayManager`:
  - `apply<R>(app, monitors: &[LogicalMonitor], config: &DisplayConfig)` — creates/resizes spanning window
  - `impl RectUpdater` — `update_rects(label, rects)` delegates to `PigHitTester`
- Same Tauri state wrapper: `pub struct DisplayManagerState(pub DisplayManager)`
- In `app/mod.rs`: replace `OverlayManagerState` with `DisplayManagerState`; `MonitorsState` now wraps `Vec<LogicalMonitor>`

---

## TDD — write tests first for pure logic

Tests go in `src-tauri/src/display/monitor.rs` (inline `#[cfg(test)]` module). Write RED test, then implement GREEN, repeat.

Required test cases for `compute_span`:

1. Single landscape monitor at origin → bounds equal monitor bounds
2. Two landscape monitors side-by-side → width = sum, x = leftmost x
3. Monitor at negative x (secondary left of primary) → min_x is negative, total width correct
4. Portrait monitor (height > width, as from a 270° rotation) left of landscape → correct origin and total width

Required test cases for `disambiguate_names`:

1. Two monitors with identical names → each gets `" (x, y)"` suffix
2. Monitors with unique names → labels unchanged

---

## Critical constraint

**Do not mark this done without confirming the app launches at runtime without crashing.** `task check` passing is necessary but not sufficient — Tauri apps can typecheck and still panic on startup. Run `cargo tauri dev` or the equivalent and verify the overlay appears.

---

## Workflow

```bash
git checkout main && git pull
git checkout -b feat/024-display-subsystem
# implement + tests
task check   # must be green
# confirm app launches at runtime
git add <files> && git commit -m "feat: 024 — display/ module tree, logical coordinate fix"
git push -u origin feat/024-display-subsystem
gh pr create --title "feat: 024 — display subsystem refactor + coordinate fix" --body "..."
```

PR body must quote the completion promise from the issue file verbatim and link `issues/024-display-subsystem.md`.

---

## Definition of done

- [ ] `display/` module tree exists; `app/overlay_manager.rs` + `app/pig_hittest.rs` deleted
- [ ] `compute_span` and `disambiguate_names` covered by 6 unit tests (all passing)
- [ ] Both monitors enabled: pigs roam full span, no barrier in the middle, no open edges
- [ ] Portrait/rotated monitor included correctly in the span
- [ ] Monitor names disambiguated in tray
- [ ] Click-through hit-testing works across the full spanning window
- [ ] App launches at runtime without crash
- [ ] `task check` green
