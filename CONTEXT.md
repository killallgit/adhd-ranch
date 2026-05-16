# Context

## Guiding metaphor

**A ranch you can see from the corner of your eye.** Pixel pig sprites roam the screen — one per Focus. They walk slowly in the background. No interruption; no modal. You glance, you know what's on your plate. Click a pig to see its tasks. Clear a task from that card. The pigs don't demand attention; they just exist.

## Ubiquitous Language

### Focus

A top-level item the user is paying attention to. Represents a real-world goal (e.g. "Customer X bug"). Created manually by the user (via menu bar or hand-edit), or in v1.3+ via an accepted `new_focus` proposal. Owns a flat list of Tasks. Rendered in the UI as a Pig.

### Pig

The visual representation of a Focus. A pixel-art sprite that wanders the screen. One pig = one Focus. Clicking a pig opens its detail card. Pigs are not stored — they are ephemeral projections of Focus state.

### RanchAnimal

The broader visual projection family for Focuses. Pig is the only implemented RanchAnimal today, but future versions may render other animals with different sprites or movement feel. Shared movement and display rules should use animal-neutral names where practical; Pig-specific names should remain only at the current sprite/UI Adapter.

### DisplaySpace

The normalized layout of enabled monitors within the spanning overlay window. Defines the overlay span, the primary spawn region, and the actual visible monitor regions where RanchAnimals may move. RanchAnimals are constrained to monitor regions, not the full rectangular span, so odd layouts do not allow invisible wandering through gaps between displays. If a RanchAnimal is outside every movement region after a display change, it moves to the nearest valid point in any enabled movement region. At movement-region edges, RanchAnimals soft-steer before impact for a natural wandering feel, then hard-clamp and reflect velocity if they still cross outside a valid region.

### Task

A child item under a Focus. Single sentence. Created by user action or (v1.3+) by an accepted `add_task` proposal. Tree is capped at two levels — Focus → Task. No sub-tasks. Removed only by user action.

### Proposal

A pending suggestion from the in-session agent at `/checkpoint` time. Three kinds: `add_task`, `new_focus`, `discard`. Held in a queue until the user accepts or rejects. **Deferred to v1.3** — not part of the current UI.

### FocusTimer

An optional countdown attached to a Focus or Task. Stores `duration_secs`, `started_at` (unix timestamp), and `status` (`Running` | `Expired`).

Focus-level timers drive pig scale growth from 1.0× at creation to 3.0× at expiry, plus expired-focus alerts.

Task-level timers expire independently. The background expiry workflow persists `status: Expired`, `AnimalDetail` displays the Task timer as Expired, and the `task_timer_expired` notification source can emit through the platform notification sink. Task timer expiry does not affect animal rendering, tray expired state, or Focus timer status.

Timers persist across restarts.

### TimerPreset

A named duration choice offered for Focus and Task timers: 2m, 4m, 8m, 16m, 32m, or Custom (free integer minutes). Maps to `duration_secs` in `FocusTimer`.

### TaskText

Newtype wrapper around a non-empty `String`. Construct via `TaskText::new(text)` — returns `DomainError::EmptyTaskText` for blank input. The only way `Commands::append_task` passes text into the storage layer.

### DomainError

Errors raised by validated domain constructors (`NewFocus::new`, `TaskText::new`). Variants: `EmptyTitle`, `EmptyTaskText`. Maps to `CommandError::BadRequest` at the commands layer. Domain owns the invariants; the commands layer owns the protocol mapping.

## Persistence

Focus = a self-contained directory under `~/.adhd-ranch/focuses/<slug>/`:

```
focuses/<slug>/
  focus.md            # frontmatter (id, title, description, created_at) + Tasks body
  timer.json          # optional Focus timer
  task-timers.json    # optional Task timers, indexed to task order
```

Top-level state:

```
~/.adhd-ranch/
  proposals.jsonl     # pending proposals; consumed when user accepts/rejects
  decisions.jsonl     # global audit log of accept/reject decisions
  settings.yaml
  run/port
  focuses/...
```

`focus.md` shape:

```
---
id: <uuid>
title: Customer X bug
description: short paragraph of intent (read by routing agent)
created_at: 2026-04-30T12:00:00Z
---
- [ ] add persistence field in compute api
- [ ] update and test sdk
```

- Tasks = top-level checkbox bullets in body. One bullet = one Task. Plain text only — no metadata fields.
- `description` is the load-bearing field for routing — agent reads it to decide if a summary belongs.
- `timer.json` sidecar: `{ "duration_secs": N, "started_at": T, "status": "Running"|"Expired" }`. Written atomically; if write fails during focus creation, focus dir is rolled back. Loaded alongside `focus.md` on every `list()` call.
- `task-timers.json` sidecar: JSON array of optional `FocusTimer` values. Array index matches the parsed task index; deleting a Task removes the matching timer entry. Expiry writes update the indexed timer to `status: Expired` atomically. Corrupted or missing task timer sidecars degrade to no task timers.
- User hand-edits anywhere; file watcher reflects changes.
- Atomic write via tmpfile + rename. `flock` per file.

## Architecture principles

- Functional core, imperative shell. Pure logic; I/O at edges.
- Single source of truth = the per-focus dirs + `proposals.jsonl`. Widget renders a projection.
- **No static binding from agent context to Focus.** Focus is a mental anchor identified by `title` + `description`. The in-session agent decides routing at `/checkpoint` time using the catalog returned by the app.
- **App holds zero LLM logic.** All reasoning happens in the agent already running in the user's session.

## Core interaction loop (v1.2)

Steps 1–8 are implemented. Display spanning uses a Rust-emitted DisplaySpace model so monitor geometry policy is local to the display module, while RanchAnimal movement consumes normalized visible monitor regions instead of the raw overlay span.

1. **Pigs roam the screen.** One pig per Focus, wandering at 60px/s with random direction changes; minimum velocity floor so pigs never look frozen. 4-direction pixel-art sprite sheet (016). Hit-box is 16px larger than sprite (018).
2. **Click a pig.** Pig freezes. `AnimalDetail` card opens near the pig (340px, opaque dark background, 16px padding): Focus title + scrollable Task list with `✗` per Task + "Add task…" input at bottom. Clock/time controls on the Focus title and each Task open compact timer dropdowns. Enter appends a task inline. Click-outside or Escape closes; pig resumes (019, 052).
3. **Drag a pig.** Click-and-hold then move > 4px enters drag mode — pig follows cursor. Release sends pig flying in that direction; friction decelerates it; bounces at screen edges. Pure click (< 4px movement) still opens AnimalDetail (020).
4. **Clear a task.** Tap `✗` → `delete_task` Tauri command → markdown updated → pig's task list reflects change.
5. **Add a task.** Type in "Add task…" input in AnimalDetail → Enter → `append_task` Tauri command → markdown updated.
6. **Create a Focus.** *(014)* Menu bar item → "+ New Focus" → small webview form → `create_focus` → new pig spawns. Timer dropdown (No timer / 2m / 4m / 8m / 16m / 32m / Custom) optionally attaches a `FocusTimer` (028). Focus and Task timers can later be started or cleared from `AnimalDetail` (052).
7. **Delete a Focus.** *(015)* Menu bar item → Focus submenu → "Delete…" → `delete_focus` → pig disappears. (Optional confirmation tracked in issue `#027`.)
8. **Configure displays.** *(017, 049)* Tray Displays section — check/uncheck monitors. Enabled monitors share one spanning overlay window; RanchAnimals spawn in the primary display region and move only inside normalized visible monitor regions. Persists in `settings.yaml`. The display module owns monitor geometry, and React owns movement over the emitted DisplaySpace model.

## Agent proposal flow (v1.3 — deferred)

1. User runs `/checkpoint` in a Claude Code session.
2. Agent reads focus catalog, composes summary, POSTs proposal.
3. App enqueues in `proposals.jsonl`.
4. User reviews proposals via tray menu submenu or modal; accepts/rejects.
5. Decisions appended to `decisions.jsonl`.

## LLM

The app does **not** call any LLM. The only model in the loop is whatever Claude Code (or future agent host) is already running in the user's session. The slash command is a prompt; quality of routing = quality of that prompt template + the model the user is already using.

No provider config. No API keys.

## Distribution

Two artifacts, macOS-only for v1, no user PATH management:

1. **`Adhd Ranch.app`** — Tauri v2 macOS bundle. Drag to `/Applications`.
2. **`adhd-ranch` skill** — deferred v1.3 proposal-flow artifact. It unpacks to `~/.claude/skills/adhd-ranch/` (global, every project) with `SKILL.md`. The slash command lives at `~/.claude/commands/checkpoint.md`.

App and skill are independent: the v1.2 app works UI-only without the skill; in v1.3 the skill is useless without the running app (the slash command will error if `~/.adhd-ranch/run/port` is missing or the health check fails — "adhd-ranch not running, please start the app").

## Application

Single Tauri v2 desktop app written in Rust (core + frontend webview). Two surfaces:

1. **Transparent overlay window** — fullscreen, always-on-top, no decorations. Renders pixel pig sprites via React. Click-through for non-pig areas via a Rust polling thread (`NSEvent.mouseLocation` at 16ms) that toggles `window.set_ignore_cursor_events`.
2. **Menu bar item (tray)** — native NSMenu with Focus list, new-focus creation, quit. Red badge when over-cap.

Responsibilities owned by the app:

- Read/write per-Focus markdown files (frontmatter + body).
- Watch `~/.adhd-ranch/focuses/` via `notify` crate; reflect external edits live (pig count updates within 1s).
- Serve the localhost HTTP API (for v1.3 `/checkpoint` flow).
- Enforce caps + emit overload alerts (`tauri-plugin-notification`).
- Poll mouse position in a background thread; maintain shared pig bounding boxes; toggle click-through.

No separate CLI, no shell scripts. Rust core is the single implementation of read/write/cap logic.

## Writers

Current v1.2 writers:

1. **User** — pig detail UI, preferences UI, tray/new-focus UI, or hand-edit markdown.

Deferred v1.3 writer:

2. **In-session agent via `/checkpoint`** — HTTP POST `/proposals`, then user accepts/rejects.

That's it. No hooks, no fallbacks.

## IPC

App exposes a localhost HTTP API on `127.0.0.1:<ephemeral-port>`. Port written to `~/.adhd-ranch/run/port` on bind; clients read it. HTTP only — no file fallback. Auth: none in v1 (localhost-only bind). In the deferred v1.3 `/checkpoint` flow, the slash command should error clearly if the app is down.

## Caps & overload alerting

Hard limits:

- `MAX_FOCUSES` (default: 5)
- `MAX_TASKS_PER_FOCUS` (default: 7)

App enforces caps at write time. When a write would exceed a cap, it succeeds but flips an over-cap flag; widget reflects badge; system notification fires once per `under → over` transition. Goes away on transition back to under.

## Configuration

`~/.adhd-ranch/settings.yaml`. Single user-editable file. Defaults applied if missing or partial.

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

Settings changed through the app are persisted immediately. Manual file edits are picked up on app restart.

## Scope

- **v1.2**: macOS only. User creates Focuses from the app or hand-edit. Pigs roam the overlay, show Tasks, and reflect markdown changes. Agent proposals are not part of the current UI.
- **v1.3 (deferred)**: `/checkpoint` slash command + proposal queue UI; accepted proposals may add Tasks or create Focuses.
- **v2 (deferred)**: external aggregators (Jira, GitHub), additional triggers (slash command for new-focus, hooks, scheduled), cross-platform, `merge_focus` and `complete_task` proposal kinds, optional `/usr/local/bin` symlink, schema migrations.
