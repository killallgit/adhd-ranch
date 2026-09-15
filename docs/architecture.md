# Architecture

Implementation notes moved out of `CONTEXT.md`, which now holds only domain language. This file will be regrouped by domain area once those areas are settled.

## Architecture principles

- Functional core, imperative shell. Pure logic; I/O at edges.
- Single source of truth = the per-focus dirs on disk. The overlay and tray render projections.
- **App holds zero LLM logic.** The app does not call any model. No provider config. No API keys.

## Persistence

Data root is `~/.adhd-ranch/` (`%APPDATA%\adhd-ranch\` on Windows).

Focus = a self-contained directory under `focuses/<slug>/`:

```
focuses/<slug>/
  focus.md            # frontmatter (id, title, description, created_at) + Tasks body
  timer.json          # optional Focus timer
  task-timers.json    # optional Task timers, indexed to task order
```

Top-level state:

```
~/.adhd-ranch/
  focuses/...
  settings.yaml
```

`focus.md` shape:

```
---
id: <uuid>
title: Customer X bug
description: short paragraph of intent
created_at: 2026-04-30T12:00:00Z
---
- [ ] add persistence field in compute api
- [x] update and test sdk
```

- Tasks = top-level checkbox bullets in body. One bullet = one Task. `- [ ]` is open, `- [x]` is done. Plain text only — no metadata fields.
- `timer.json` sidecar: `{ "duration_secs": N, "started_at": T, "status": "Running"|"Expired" }`. Written atomically; if write fails during focus creation, focus dir is rolled back. Loaded alongside `focus.md` on every `list()` call.
- `task-timers.json` sidecar: JSON array of optional `FocusTimer` values. Array index matches the parsed task index; deleting a Task removes the matching timer entry. Expiry writes update the indexed timer to `status: Expired` atomically. Corrupted or missing task timer sidecars degrade to no task timers.
- User hand-edits anywhere; file watcher reflects changes.
- Atomic write via tmpfile + rename, serialized by an exclusive lock file per target.
- First launch seeds one example Focus. A marker file stops it from coming back after the user clears every Focus.

## Domain types

- `FocusTimer` stores `duration_secs`, `started_at` (unix timestamp), and `status` (`Running` | `Expired`).
- `TaskText` is a newtype wrapper around a non-empty `String`. Construct via `TaskText::new(text)` — returns `DomainError::EmptyTaskText` for blank input. The only way `Commands::append_task` passes text into the storage layer.
- `DomainError` covers errors raised by validated domain constructors (`NewFocus::new`, `TaskText::new`). Variants: `EmptyTitle`, `EmptyTaskText`. Maps to `CommandError::BadRequest` at the commands layer. Domain owns the invariants; the commands layer owns the protocol mapping.

## Display spanning

Display spanning uses a Rust-emitted DisplaySpace model so monitor geometry policy is local to the display module, while RanchAnimal movement consumes normalized visible monitor regions instead of the raw overlay span.

## Core interaction loop

1. **Pigs roam the screen.** One pig per Focus, wandering at 60px/s with random direction changes every 3–8 s; minimum velocity floor so pigs never look frozen. 4-direction pixel-art sprite sheet (016). Hit-box is 16px larger than sprite (018).
2. **Click a pig.** Pig freezes. `AnimalDetail` card opens near the pig: editable Focus title, duplicate and delete buttons, scrollable Task list (check off, edit text, `✗` to clear), and an "Add task…" input at the bottom. Clock/time controls on the Focus title and each Task open compact timer dropdowns. Click-outside or Escape closes; pig resumes (019, 052).
3. **Drag a pig.** Click-and-hold then move > 4px enters drag mode — pig follows cursor. Release sends pig flying in that direction; friction decelerates it; bounces at region edges. Pure click (< 4px movement) still opens AnimalDetail (020).
4. **Clear a task.** Tap `✗` → `delete_task` Tauri command → markdown updated → pig's task list reflects change.
5. **Add a task.** Type in "Add task…" input in AnimalDetail → Enter → `append_task` Tauri command → markdown updated.
6. **Create a Focus.** *(014)* Tray → "+ New Focus" → small webview form → `create_focus` → new pig spawns. Timer dropdown (No timer / 2m / 4m / 8m / 16m / 32m / Custom) optionally attaches a `FocusTimer` (028). Focus and Task timers can later be started or cleared from `AnimalDetail` (052).
7. **Delete a Focus.** *(015, 027)* Tray → Focus submenu → "Delete…", or the delete button in `AnimalDetail` → `delete_focus` → pig disappears. Asks for confirmation when `widget.confirm_delete` is on.
8. **Configure displays.** *(017, 049)* Preferences → Displays — check/uncheck monitors. Enabled monitors share one spanning overlay window; RanchAnimals spawn in the primary display region and move only inside normalized visible monitor regions. Persists in `settings.yaml`. The display module owns monitor geometry, and React owns movement over the emitted DisplaySpace model.
9. **Change settings.** *(026, 032, 050)* Tray → "Settings…" or app menu → "Preferences…" opens the Preferences window. `update_settings` runs the settings workflow: persist `settings.yaml`, commit the in-memory settings, apply widget changes, reapply overlays when displays changed, refresh long-lived settings consumers, and rebuild the tray. Caps and notification toggles take effect without a restart.

## Application

Single Tauri v2 desktop app written in Rust (core + frontend webview). macOS is the primary target; the release workflow also builds unsigned Windows and Linux packages. Surfaces:

1. **Transparent overlay window** — spans the enabled displays, always-on-top, no decorations. Renders pixel pig sprites via React. Click-through for non-pig areas via a Rust polling thread (Tauri `cursor_position()` every 16ms) that toggles `window.set_ignore_cursor_events`.
2. **Tray menu** — Gather Pigs, "+ New Focus", one submenu per Focus (with Delete), an Expired submenu, Settings…, and Quit (plus Open Overlay DevTools in debug builds). The tray icon turns red when over-cap.
3. **New Focus and Preferences windows** — small webviews opened from the tray or app menu.

Responsibilities owned by the app:

- Read/write per-Focus markdown files (frontmatter + body).
- Watch `~/.adhd-ranch/focuses/` via `notify` crate; reflect external edits live (pig count updates within 1s).
- Run a 1s timer-expiry loop that persists expired timers and sends notifications. The UI picks up the change through the focuses watcher.
- Enforce caps + emit overload alerts (`tauri-plugin-notification`).
- Poll mouse position in a background thread; maintain shared pig bounding boxes; toggle click-through.

No network API, no separate CLI, no shell scripts. The UI reaches Rust only through Tauri IPC. Rust core is the single implementation of read/write/cap logic.

## Writers

1. **User** — pig detail UI, Preferences UI, tray/new-focus UI, or hand-edit markdown.
2. **Timer expiry loop** — marks timers `Expired`.

## Caps & overload alerting

Hard limits:

- `MAX_FOCUSES` (default: 5)
- `MAX_TASKS_PER_FOCUS` (default: 7)

App enforces caps at write time. When a write would exceed a cap, it succeeds but flips an over-cap flag; the tray icon turns red; system notification fires once per `under → over` transition. Goes away on transition back to under.

## Agents as Animals

An opt-in overlay mode that adds one pig per running Claude Code session alongside the Focus pigs. It is off by default and controlled by `agents.enabled` in `settings.yaml`.

- **Toggle.** The tray's "Agents as Animals" check item flips `agents.enabled` through the settings workflow (persist, commit, effects, tray rebuild). The workflow's agents effect runs only when the value changes.
- **Hook install.** When agents are enabled at startup, or turned on from the tray, the app writes `~/.adhd-ranch/hooks/claude-session.sh` and adds `SessionStart` / `SessionEnd` command hooks for it to `~/.claude/settings.json` (or `$CLAUDE_CONFIG_DIR/settings.json`). Registration is idempotent: it matches on the exact command, keeps the settings file's key order and permissions, writes through a symlink, and leaves unparseable settings untouched. Turning agents off never removes the hook; it only hides agent pigs.
- **Session files.** On `SessionStart` the script writes the hook payload to `~/.adhd-ranch/sessions/claude/<session_id>.json`; on `SessionEnd` it deletes that file. Session ids that aren't letters, digits, and dashes are ignored.
- **Projection.** A watcher on the sessions directory emits `animals-changed`; the overlay calls `list_animals`, which reads the session files (one Animal per file, named after the session's working directory) and returns an empty list while `agents.enabled` is false. Toggling also emits `animals-changed` so pigs appear or vanish immediately.
- **Overlay.** Agent pigs use `agent:<session_id>` ids, have no detail card, and never enter the selected state. Sessions that crash without `SessionEnd` leave their file behind until it is deleted.

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
agents:
  enabled: false
```

Settings changed through the app are persisted and applied immediately. Manual file edits are picked up on app restart.

## Scope

- **Current**: User creates Focuses from the app or by hand-editing. Pigs roam the overlay, show Tasks, and reflect markdown changes. Focus and Task timers, caps, and multi-display overlay work. Optional agent pigs for running Claude Code sessions.
- **Not built**: external aggregators (Jira, GitHub), sync, schema migrations.
