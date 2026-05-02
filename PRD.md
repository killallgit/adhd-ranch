# PRD — adhd-ranch

**Status:** Draft v1.2
**Owner:** ryan
**Last updated:** 2026-05-02

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

## Goals (v1.2)

1. Pixel pig sprites roam a fullscreen transparent overlay — one pig per Focus.
2. Clicking a pig shows its name and Task list in a small popover. Tasks can be cleared from the popover.
3. Manual Focus creation via menu bar item (simple native-style list). No agent flow in v1.2.
4. Markdown is the source of truth — user can hand-edit any Focus file; pig count updates live via file watcher.
5. Hard caps (5 Focuses, 7 Tasks per Focus) with overload alerts.

## Non-goals (v1.2)

- Agent proposals / `/checkpoint` flow — deferred to v1.3.
- Notification hook forwarding — deferred.
- Auto-completion of Tasks (`complete_task`) — deferred.
- Auto-merge of Focuses (`merge_focus`) — deferred.
- Cross-platform (Linux/Windows). macOS only.
- Multi-user / sync / cloud.
- Fancy menu bar UI — native NSMenu list is sufficient.

## User stories

- **US1.** I glance at my screen and see three pixel pigs wandering in the corners. I know immediately: three things are on my plate. I don't have to open anything.
- **US2.** I click a pig. A small dark card appears near the pig: its name and a short task list. I tap `✗` next to a task. It disappears. Card closes when I click elsewhere.
- **US3.** I finish a Focus. I open the menu bar item, find it in the list, and delete it. Pig disappears from the screen.
- **US4.** I add a Focus: click the menu bar item → "+ New Focus" → enter name + description. A new pig spawns and starts wandering.
- **US5.** I hand-edit `~/.adhd-ranch/focuses/customer-x-bug/focus.md` in vim, append `- [ ] release staging`. Save. Pig's task list reflects it within seconds.
- **US6.** I have 6 Focuses. The menu bar icon shows a red badge. I delete a stale Focus.

## Functional requirements

### FR1 — Focus storage

Unchanged. Each Focus is a directory under `~/.adhd-ranch/focuses/<slug>/` containing `focus.md` with YAML frontmatter (`id`, `title`, `description`, `created_at`) and a body of `- [ ]` bullets. Plain text only.

### FR2 — Transparent overlay window

- Full-screen transparent Tauri window: no decorations, always-on-top, covers the primary monitor.
- Click-through when not hovering a pig: Rust polling thread reads `NSEvent.mouseLocation` every 16ms, compares against pig bounding boxes (sent from frontend), calls `window.set_ignore_cursor_events(!is_over_pig)`.
- Pigs receive click events normally; transparent background passes clicks to whatever is beneath.
- File watcher (`notify`) on `~/.adhd-ranch/focuses/`; pig count re-renders on disk changes.

### FR3 — Pig UI

- One `PigSprite` per Focus, positioned at the sprite's current (x, y) on the overlay.
- Pigs wander the full screen: slow drift (~35 px/s), smooth random direction changes every 3–8 s, gentle boundary steering (40px margin from edges).
- Animation: 4 frames per direction (left/right), ticked at ~150ms (≈6.7fps).
- Sprite: real pixel-art sprite sheet when assets are ready (4 directions × 4 frames in one PNG); placeholder CSS/SVG pig until then.
- Clicking a pig opens `PigDetail` popover near the pig (edge-clamped): Focus title + task list + `✗` per task.
- `PigDetail` closes on click-outside.

### FR4 — Menu bar item

- Tray icon in the macOS menu bar.
- Native NSMenu with:
  - List of current Focuses (each as a menu item showing title).
  - Clicking a Focus item → brings the pig into view / opens its detail (TBD).
  - Separator.
  - "+ New Focus" → opens a small webview popover for title + description input.
  - Separator.
  - "Quit" → Cmd-Q.
- Red badge on tray icon when over-cap.

### FR5 — HTTP API

Localhost-only, ephemeral port. Retained for `/checkpoint` flow (v1.3). No changes to routes.

### FR6 — Caps

- `MAX_FOCUSES = 5`, `MAX_TASKS_PER_FOCUS = 7` (configurable in `settings.yaml`).
- Writes exceeding caps succeed but flip over-cap flag.
- Tray icon shows red badge while over.
- macOS notification fires once per `under → over` transition.

### FR7 — Configuration

`~/.adhd-ranch/settings.yaml`:
```yaml
caps:
  max_focuses: 5
  max_tasks_per_focus: 7
alerts:
  system_notifications: true
widget:
  always_on_top: true
```

### FR8 — Audit log

Retained. Every accepted/rejected proposal appended to `~/.adhd-ranch/decisions.jsonl`. Unused in v1.2 but preserved for v1.3.

## Non-functional requirements

- **Latency:** pig count reflects file changes within 1s of save.
- **Click-through:** hit-test polling at ~60fps (16ms); transition latency < 32ms.
- **Reliability:** atomic writes (tmpfile + rename), per-file `flock`. No partial-write corruption.
- **Footprint:** Tauri release build < 20 MB on disk; idle RAM < 50 MB.
- **Privacy:** zero outbound network. No telemetry.

## Success metrics (v1.2)

- Author can name their active Focuses by glancing at the screen without opening any tool.
- Pigs are visible and not disruptive — clicks pass through to other apps with < 32ms latency.
- Click-a-pig → task card round trip feels instant (< 100ms perceived).

## Out of scope / v1.3+

- `/checkpoint` slash command + agent proposal flow.
- `merge_focus` and `complete_task` proposal kinds.
- Menu bar Focus detail (clicking Focus in menu → highlight pig, open detail).
- Notification-hook forwarding.
- Linux + Windows ports.
- External aggregators: Jira, GitHub, Linear.
- Multi-machine sync.

## Open questions / risks

- **R1.** Click-through latency: 16ms Rust poll + IPC round-trip should feel transparent, but needs real-device testing.
- **R2.** ~~NSEvent.mouseLocation coordinate space~~ — resolved. Polling thread subtracts `window.outer_position()` from cursor before hit-test; pig rects are in webview-local physical pixels.
- **R3.** Multiple monitors: pigs spawn on primary monitor only. Tracked in issue #11 (017 — configurable display spanning).
- **R4.** Pig positions on resize: if screen resolution changes (external monitor connect/disconnect), pigs reset to safe positions.
- **R5.** Always-on-top + fullscreen apps: at kCGFloatingWindowLevel (3), pigs disappear behind fullscreen apps. Acceptable for v1.2.

## Implementation phases

1. **Phase 0 (done):** Tauri skeleton, storage, HTTP API, markdown read/write, caps, file watcher, proposals queue.
2. **Phase 1 (done):** Custom titlebar, app menu, always-on-top, regular Mac app.
3. **Phase 2 (done):** Transparent fullscreen window, click-through Rust polling thread, `PigSprite` placeholder, `usePigMovement`, `PigDetail` popover, tray icon + live focus list, typed errors, structured logging.
4. **Phase 3 (in progress):** ~~New-focus creation from tray (issue #7/014)~~, ~~delete from tray (issue #8/015)~~; remaining: configurable display spanning (issue #11/017), real sprite sheet (issue #9/016 — needs asset).
5. **Phase 4 — Agent flow (v1.3):** Restore `/checkpoint` command + proposal queue UI (tray submenu or modal).
