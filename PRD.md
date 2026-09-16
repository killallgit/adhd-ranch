# PRD — adhd-ranch

**Status:** Living document — describes the app on `main` (latest release v0.1.2)
**Owner:** ryan
**Last updated:** 2026-09-14

---

## Problem

A developer working across many microservices, agents, repos, and tickets loses the thread. Tools that try to help (Jira, GitHub, Linear) demand structure and time the user can't spend mid-flight. Notes, todos, and ticket trees become more cognitive load, not less. Agentic tools accelerate this — Claude Code spawns a tree of in-flight work with no central place to track "what am I actually focused on right now?"

The problem isn't capacity. It's *signal compression*. The user needs a few buckets they can glance at and a way for those buckets to stay current without manual upkeep.

## Why now

Agentic coding is the multiplier. A user with 3+ Claude Code sessions running at once has exactly the context-fragmentation this addresses. The ambient pig overlay makes Focuses physically visible without demanding attention — the reminder is peripheral, not modal.

## Target user

Solo developer (initially: the author) who:
- Uses Claude Code daily across multiple repos.
- Has ADHD-flavored attention, or just runs more parallel work than working memory accommodates.
- Owns a Mac.
- Will hand-edit markdown if it's faster than clicking.

## Guiding metaphor

**A ranch you can see from the corner of your eye.** Pixel pig sprites roam the screen — one pig per Focus. They walk slowly. You don't have to look at them. But when you glance, you know what's on your ranch. Click a pig to see its tasks. Clear a task from that card. The pigs never interrupt; they just exist.

## Goals

1. Pixel pig sprites roam a transparent overlay spanning the enabled displays — one pig per Focus.
2. Clicking a pig shows its name and Task list in the `AnimalDetail` card. Tasks can be added, edited, checked off, and cleared from that card, and Focus/Task timers can be edited from clock/time controls.
3. Manual Focus creation from the tray menu.
4. Markdown is the source of truth — user can hand-edit any Focus file; pig count updates live via file watcher.
5. Hard caps (5 Focuses, 7 Tasks per Focus) with overload alerts.

## Non-goals

- Any agent, slash command, network API, or other external writer.
- Notification hook forwarding.
- Auto-completion of Tasks or auto-merge of Focuses.
- Platform parity. macOS is the primary target; Windows and Linux packages are built but are not a design driver.
- Multi-user / sync / cloud.
- Fancy menu bar UI — native tray menu is sufficient.

## User stories

- **US1.** I glance at my screen and see three pixel pigs wandering in the corners. I know immediately: three things are on my plate. I don't have to open anything.
- **US2.** I click a pig. A small dark card appears near the pig: its name and a short task list. I tap `✗` next to a task. It disappears. I click a clock/time control to start or clear a timer. Card closes when I click elsewhere.
- **US3.** I finish a Focus. I open the menu bar item, find it in the list, and delete it. Pig disappears from the screen.
- **US4.** I add a Focus: click the menu bar item → "+ New Focus" → enter name + description. A new pig spawns and starts wandering.
- **US5.** I hand-edit `~/.adhd-ranch/focuses/customer-x-bug/focus.md` in vim, append `- [ ] release staging`. Save. Pig's task list reflects it within seconds.
- **US6.** I have 6 Focuses. The tray icon turns red. I delete a stale Focus.

## Functional requirements

### FR1 — Focus storage

Each Focus is a directory under `~/.adhd-ranch/focuses/<slug>/` containing `focus.md` with YAML frontmatter (`id`, `title`, `description`, `created_at`) and a body of `- [ ]` / `- [x]` bullets. Plain text only. Optional `timer.json` and `task-timers.json` sidecars hold Focus and Task timers. On Windows the data root is `%APPDATA%\adhd-ranch\`.

### FR2 — Transparent overlay window

- Transparent Tauri overlay window: no decorations, always-on-top, covering the enabled display span.
- Click-through when not hovering a pig: Rust polling thread reads Tauri `cursor_position()` every 16ms, compares against pig bounding boxes (sent from frontend), calls `window.set_ignore_cursor_events(!is_over_pig)`.
- Pigs receive click events normally; transparent background passes clicks to whatever is beneath.
- File watcher (`notify`) on `~/.adhd-ranch/focuses/`; pig count re-renders on disk changes.

### FR3 — Pig UI

- One `PigSprite` per Focus, positioned at the sprite's current (x, y) on the overlay.
- Pigs wander the enabled display regions: ~60 px/s with a minimum speed floor, random direction changes every 3–8 s, soft steering near region edges.
- Animation: real pixel-art sprite sheet (4 directions × 4 frames in one PNG), ticked at ~150ms.
- Drag and toss: click-and-hold then move ≥ 4px drags the pig; release tosses it with friction.
- Clicking a pig opens the `AnimalDetail` panel near the pig (edge-clamped): editable Focus title, duplicate and delete actions, task list with check-off, edit, and `✗` per task, and an "Add task…" input.
- Focus and Task timer editing is accessed by clicking the clock icon or current remaining time. No timer renders as a small clock; a running/expired timer renders as its current time/expired status.
- `AnimalDetail` closes on click-outside or Escape.
- **Focus timer growth (028 + 030):** If a Focus has a `FocusTimer`, its current animal projection grows from 1× to 3× sprite size linearly over the timer window. Focuses without a timer stay at 1×. Expired animals become ghostly, stop moving, face away, and appear in the tray's Expired section. Adding a new task to an expired Focus clears the expired timer and revives the animal.
- **Task timers (052 + 053):** Task timers are independent per Task. When a Task timer expires, `status: Expired` is persisted, `AnimalDetail` renders the Task timer as Expired, and the `task_timer_expired` notification source can emit through the platform notification sink. Task timer expiry does not affect animal rendering, tray expired state, or Focus timer status.
- **Implementation status:** The only concrete animal today is still the pig sprite; the detail surface is animal-neutral as `AnimalDetail`.

### FR4 — Tray menu

- Tray icon in the menu bar.
- Native menu with:
  - "Gather Pigs" — pulls every pig back onto the primary display.
  - "+ New Focus" → opens a small webview window for title, description, and optional timer.
  - One submenu per Focus with "Delete…" (confirms when `widget.confirm_delete` is on).
  - Expired submenu listing expired Focuses; clicking one opens its detail card.
  - "Settings…" → opens the Preferences window.
  - "Open Overlay DevTools" (debug builds only).
  - "Quit".
- Tray icon turns red when over-cap.

### FR5 — Caps

- `MAX_FOCUSES = 5`, `MAX_TASKS_PER_FOCUS = 7` (configurable in `settings.yaml`).
- Writes exceeding caps succeed but flip over-cap flag.
- Tray icon turns red while over.
- System notification fires once per `under → over` transition.

### FR6 — Configuration

`~/.adhd-ranch/settings.yaml`:
```yaml
caps:
  max_focuses: 5
  max_tasks_per_focus: 7
notifications:
  timer_expired: true
  task_timer_expired: true
  focuses_over_cap: true
  tasks_over_cap: true
widget:
  always_on_top: true
  confirm_delete: true
displays:
  enabled: 0
```

Edited from the Preferences window (General, Widget, Displays, Notifications). Changes made there are persisted and applied without a restart (050). Manual file edits are picked up on app restart.

Timer presets available at Focus creation and in `AnimalDetail` clock dropdowns: No timer / 2m / 4m / 8m / 16m / 32m / Custom (free integer minutes). `AnimalDetail` allows start/restart/clear for the Focus timer and each Task timer.

## Non-functional requirements

- **Latency:** pig count reflects file changes within 1s of save.
- **Click-through:** hit-test polling at ~60fps (16ms); transition latency < 32ms.
- **Reliability:** atomic writes (tmpfile + rename) behind an exclusive lock file. No partial-write corruption.
- **Footprint:** Tauri release build < 20 MB on disk; idle RAM < 50 MB.
- **Privacy:** zero outbound network. No telemetry.

## Success metrics

- Author can name their active Focuses by glancing at the screen without opening any tool.
- Pigs are visible and not disruptive — clicks pass through to other apps with < 32ms latency.
- Click-a-pig → task card round trip feels instant (< 100ms perceived).

## Out of scope

- Agent integrations of any kind.
- Menu bar Focus detail for non-expired Focuses (clicking a Focus in the menu → highlight pig, open detail).
- Notification-hook forwarding.
- External aggregators: Jira, GitHub, Linear.
- Multi-machine sync.

## Open questions / risks

- **R1.** Click-through latency: 16ms Rust poll + IPC round-trip should feel transparent, but needs real-device testing.
- **R2.** ~~Cursor coordinate space~~ — resolved for single-monitor. `drag_active: AtomicBool` in hit-test thread prevents click-through race during drag. Real-device mixed-monitor drag still needs periodic validation.
- **R3.** ~~Multiple monitors: pigs spawn on primary monitor only.~~ 024 `display/` refactor fixed logical coordinate math and window sizing. 049 added DisplaySpace: Rust owns monitor geometry, React movement consumes normalized visible monitor regions, and Animals cannot wander into invisible gaps inside the overlay span. Only the primary display is enabled by default (icebox 021).
- **R4.** Pig positions on resize: if screen resolution changes (external monitor connect/disconnect), pigs reset to safe positions.
- **R5.** Always-on-top + fullscreen apps: at kCGFloatingWindowLevel (3), pigs disappear behind fullscreen apps. Acceptable for now.

## Implementation phases

1. **Phase 0 (done):** Tauri skeleton, storage, HTTP API, markdown read/write, caps, file watcher, proposals queue.
2. **Phase 1 (done):** Custom titlebar, app menu, always-on-top, regular Mac app.
3. **Phase 2 (done):** Transparent fullscreen window, click-through Rust polling thread, `PigSprite` placeholder, `usePigMovement`, animal detail card, tray icon + live focus list, typed errors, structured logging.
4. **Phase 3 (done):** New-focus creation from tray (014), delete from tray (015), configurable display spanning (017), real sprite sheet (016).
5. **Phase 3 polish (done):** Larger pig hitbox (018), AnimalDetail redesign (019), drag-and-toss physics (020), display subsystem refactor (024), pig freeze fix + keep-still (025), settings/preferences (026, 027, 032), DisplaySpace seam (049).
6. **Phase 4 — Timers (done):** Focus timers (028), timer expiry + notification sources (029), growth + expired tray list (030), Task timers + clock dropdowns (052), Task timer expiry workflow (053).
7. **Phase 5 — Architecture deepening (done):** IPC layer (033), domain invariants (034), store tests (035), ts-rs types (036), reader/writer collapse (037–041), domain timer ticker (044), focus document module (048), settings update workflow (050).
8. **Phase 6 — Distribution (done):** Focus duplication, first-launch example Focus, Windows data paths (#66); manual cross-platform release workflow (#72).
9. **Baseline cleanup (done):** removed the unused Proposal queue, decision log, localhost HTTP API, and orphaned frontend components.
10. **Open:** notification source registry (031), Timers module (054), Animal vocabulary for shared movement code (051).
11. **Icebox:** all-monitors default on first launch (021), wrangle pig / wrangle all (022).
