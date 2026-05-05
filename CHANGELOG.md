# Changelog

All notable changes to adhd-ranch. Follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

---

## [Unreleased]

### Added — 034 focus/task invariants in domain (PR #40, in flight)

- `crates/domain/src/error.rs` — new `DomainError` enum: `EmptyTitle`, `EmptyTaskText`
- `NewFocus::new(title, description) -> Result<Self, DomainError>` validated constructor
- `crates/domain/src/focus.rs` — `TaskText` newtype with validated `new` + `as_str`
- `From<DomainError> for CommandError` — maps to `BadRequest`
- Command-handler regression tests assert blank input → `BadRequest`

### Changed — 034

- `Commands::create_focus` and `Commands::append_task` no longer carry their own `trim().is_empty()` guards; they call `NewFocus::new` and `TaskText::new` instead
- Focus title and task text invariants now enforced once in `crates/domain/`

### Added — 035 unit tests for `MarkdownFocusStore` (PR #41, in flight)

- 8 direct unit tests in `crates/storage/src/focus_store.rs`: create/list roundtrip, timer sidecar present/absent, delete + delete-of-unknown, corrupted `timer.json`, append/delete task persistence
- Storage seam now independently trusted without going through `Commands`

### Changed — 035

- `MarkdownFocusStore::list()` degrades to `timer: None` when `timer.json` is corrupted instead of failing the whole load — UI keeps showing the focus, user can recreate the timer

### Changed — 033 pig/drag IPC moved into `api/` layer (PR #39, in flight)

- `src/api/pig.ts` — new typed wrappers: `setPigDragActive`, `updatePigRects`, `subscribeGatherPigs`, `subscribeDisplayRegion`
- `src/types/pig.ts` — shared `SpawnRegion`, `PigHitRect` (api/ no longer depends on hooks/)
- `src/components/App.tsx` and `src/hooks/usePigMovement.ts` no longer import `@tauri-apps/api/core` or `@tauri-apps/api/event`; both go through `src/api/pig.ts`
- Subscription effects wrap a no-op fallback so cleanup cannot raise an unhandled rejection on strict-mode unmount

### Fixed

- `Taskfile.yaml` — drop stale `ralph/` include (left dangling by `dabed05 remove ralph dir`); `task check` runs again
- `issues/` — archive pre-merge drafts of 013/016/018/019/020 and move 032 into `issues/done/` (PR #42)

### Added — 028 focus timer: domain types + full-stack creation (PR #32)

- `crates/domain/src/timer.rs` — pure domain: `FocusTimer`, `TimerPreset` (2/4/8/16/32m + Custom), `TimerStatus`, `timer_remaining_secs()`, `growth_factor()`
- `Focus.timer: Option<FocusTimer>` — timer persisted as `timer.json` sidecar alongside `focus.md`; loaded on `list()`
- `NewFocus.timer_preset: Option<TimerPreset>` — preset carried through creation path
- `Commands` gains injected `ClockSecs` for deterministic `started_at` in tests
- `ProposalLifecycle` also gains `ClockSecs`; builds `FocusTimer` when accepting `NewFocus` proposals with a preset
- `ServerDeps.clock_secs` exposed for API test determinism
- `NewFocusForm` — timer dropdown (No timer / 2m / 4m / 8m / 16m / 32m / Custom); Custom shows number input
- `src/types/timer.ts` — `TimerPreset`, `FocusTimer`, `TimerStatus` TypeScript types
- `focusWriter.ts` — `createFocus` accepts and forwards `timer_preset`
- `create_focus` atomic: rollback (remove focus dir) if `timer.json` write fails after `focus.md` committed

### Added — 024 display subsystem (WIP, PR #27 — cross-monitor drag still broken)

- `display/` module tree replaces `app/overlay_manager.rs` + `app/pig_hittest.rs`
  - `display/monitor.rs` — `LogicalMonitor`, `compute_span`, `disambiguate_names` (7 unit tests)
  - `display/overlay.rs` — window lifecycle; `ShowParams` struct; hit-test polling thread
  - `display/mod.rs` — `DisplayManager`, `PrimaryRegion`, `drag_active: Arc<AtomicBool>`
- `PrimaryRegion` event — Rust emits CSS offset+size of first enabled monitor on window show; React confines pig spawn to visible display
- `drag_active` AtomicBool — hit-test thread forces overlay interactive during JS drag, eliminating click-through race at monitor boundary
- `set_pig_drag_active` Tauri command — JS sets on pointer-down/up so flag is live before first 16ms poll
- "Gather Pigs" tray menu item — snaps all pigs to top-right of primary display; rescues pigs lost on secondary
- "Open Overlay DevTools" tray item (debug builds only)
- Red debug banner in overlay (dev mode): shows window size, focus count, pig count
- Log file at `~/Library/Logs/com.adhd-ranch.app/Adhd Ranch.log`
- `issues/027-confirm-delete-setting.md` — tracks adding a confirm-delete preference

### Fixed — 024 (CR review)

- `display/mod.rs`: filter enabled monitors by `LogicalMonitor.index` not enumerate position — wrong displays could be selected on toggle
- `display/mod.rs`: introduce `DisplayService` trait; `DisplayManagerState` holds `Arc<dyn DisplayService>`; tray + overlay use concrete `Wry` runtime
- `display/overlay.rs`: `Arc<AtomicBool>` stop flag per overlay entry — old poller thread exits before window is recreated, preventing dual-poller race on display toggle
- `display/monitor.rs`: `compute_span_portrait_above_landscape` test added (negative-y portrait above primary)
- `App.tsx`: Tauri listener cleanup uses Promise-chaining — prevents subscription leak when component unmounts before `listen()` resolves
- `PigSprite.tsx`: `onLostPointerCapture` guards on `startPosRef.current` not `isDraggingRef.current` — `drag_active` clears even when capture lost before drag threshold
- `PigSprite.tsx`: `invoke("set_pig_drag_active")` moved to `App.tsx` — component is now I/O-free per CLAUDE.md
- `usePigMovement.ts`: `gather()` wraps into columns when pig count exceeds display height — pigs no longer fall off short displays
- `styles.css` / `NewFocusWindow.tsx`: `.new-focus-form--window` modifier split out — `min-height: 100vh` no longer bleeds into non-window usages of the form

### Changed — 024

- Window builder uses `.inner_size().position()` instead of post-build `set_size()` — macOS WKWebView was overriding `set_size()` and resetting window to 800×600
- `from_tauri` divides monitor size (physical) and position (logical) each by `scale_factor` → uniform logical space for `compute_span`
- `compute_span` and `disambiguate_names` now in pure `display/monitor.rs` with tests
- `PIG_SPEED` raised 35 → 60 px/s; minimum velocity floor (35% of `PIG_SPEED`) so pigs never appear frozen
- `tickPig` uses `effectiveMaxY = primaryRegion.h` when pig is in primary display x-range — prevents pigs entering the dead zone below the main display on a multi-height span
- Drag: `startDrag` sends wide hit-rect immediately (was deferred up to 67ms); `endDrag` restores narrow rects immediately
- `PigSprite` adds `onPointerCancel` + `onLostPointerCapture` handlers to clean up stale drag state
- `gather()` places pigs relative to `primaryRegion` top-right instead of raw `screenW`
- New-focus window: dark opaque background (`rgba(22,22,26,0.97)`), larger padding, readable inputs
- Confirm-delete dialog removed from tray — deletes immediately; setting tracked in #027

## [1.2.1] — 2026-05-03 — Phase 3 polish

### Added

- `HITBOX_PADDING = 16` — pig hit-rect is 16px larger than sprite and centred; easier to click (018)
- `buildHitRects(pigs, dpr): PigHitRect[]` — exported pure function for rect calculation
- `PigDetail` add-task input — "Add task…" field at bottom of card; Enter appends task inline via `appendTask` (019)
- Drag-and-toss pig physics — click+drag moves pig under cursor; release sends it flying with velocity computed from last 80ms of pointer history; `FRICTION = 0.97` decelerates each frame; bounces at edges (020)
- `computeTossVelocity(samples, windowMs, now)` — exported pure function; velocity capped at `PIG_SPEED × 6`
- `DRAG_THRESHOLD = 4` — pointer must move ≥ 4px for drag; otherwise treated as click → opens PigDetail

### Changed

- `PigDetail` card: fully opaque background, `min-width: 340px`, `padding: 16px`, task list scrolls at `max-height: 240px`
- `usePigMovement` returns `{ pigs, startDrag, moveDrag, endDrag }` instead of bare `PigState[]`
- `tickPig` speed cap raised from `PIG_SPEED × 1.2` to `PIG_SPEED × 6` to allow post-toss deceleration

## [1.1.0] — 2026-05-01

Regular Mac app pivot. Replaces the tray-popover model with a draggable floating window, Dock icon, and standard app menu.

### Added
- Custom titlebar with drag region and close `×` button (hides window, keeps app running)
- App menu: `Adhd Ranch` / `File` / `Edit` / `Window` submenus
- `Window > Always on Top` checkbox — toggles window level and persists to `settings.yaml`
- `Window > Show Ranch` — re-shows and focuses the window
- Dock icon and `RunEvent::Reopen` handler (clicking Dock icon reopens window)
- `widget.always_on_top: bool` setting in `settings.yaml` (replaces `window_level` enum)
- Window position + size autosaved via `setFrameAutosaveName:` across launches
- `Settings::to_yaml()` + `storage::write_settings` for atomic settings persistence
- `File > Close` and `Cmd-W` hide the window; `Cmd-Q` quits

### Changed
- Window size: 400×600, resizable, minimum 320×400 (was 360×480, fixed)
- Activation policy: `Regular` (Dock icon) instead of `Accessory` (hidden from Dock)
- `WindowLevel` enum replaced with `always_on_top: bool`

### Removed
- Tray icon and `tray.rs`
- `window_level.rs` and `WindowLevel` / `parse_window_level` from domain
- `tauri-plugin-positioner` dependency
- `WindowEvent::Focused(false) → hide` auto-hide on focus loss
- `widget.window_level` settings key

---

## [1.0.0] — 2026-04 (tray-popover baseline)

### Added
- Tray icon toggles borderless popover window
- Focus CRUD: create, delete, task add/clear from widget
- Proposal queue: `/checkpoint` skill enqueues proposals; accept/reject from widget
- Decisions audit log (append-only `.jsonl`)
- Cap monitoring: max focuses + max tasks per focus with macOS notifications
- Edit proposal modal with focus-retarget support
- Localhost HTTP API (`/health`, `/focuses`, `/proposals`, `/caps`)
- Live file-watcher refresh (debounced, no polling)
- `settings.yaml`: caps, alerts, widget config
- `.app` + `.dmg` packaging; tag-driven GitHub releases
- CI: lint + typecheck + tests on push

