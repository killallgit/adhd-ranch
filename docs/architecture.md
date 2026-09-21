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

## Timers

Every Timer rule lives in one `Timers` module in `crates/commands`, built once in `app::run`.

- **Owner as data.** `TimerOwner` is `Focus { focus_id }` or `Task { focus_id, index }`, so Focus and Task Timers take the same path through `start`, `clear`, `revive_if_expired` and `expire_due`. Tasks are still matched by position (056 covers the drift when a user reorders bullets by hand).
- **Storage seam.** `TimerStore` has three methods: `focuses()`, `timer(owner)` and `write_timer(owner, Option<FocusTimer>)`. A missing Focus or a corrupt sidecar reads as no Timer, so an unrelated broken `focus.md` cannot block a Revive. `MarkdownFocusStore` writes the sidecars; `InMemoryTimerStore` is the adapter the use-case tests run against. `FocusStore` holds no Timer methods.
- **Expiry.** The 1s host loop calls `expire_due(now)`, which runs the pure `domain::tick`, persists `status: Expired` and notifies through the enabled notification sources. Persisted Expired status is the dedup lock, so a Timer fires once.
- **Revive.** `Commands::append_task` revives the Focus after writing the Task, reading just that Focus's Timer: an expired Focus Timer is cleared so its Animal goes back to normal. Best effort — a failed revive logs and leaves the Task written. Task Timers are never revived.
- **IPC.** Two commands: `start_timer(owner, preset)` and `clear_timer(owner)`.

## Domain types

- `FocusTimer` stores `duration_secs`, `started_at` (unix timestamp), and `status` (`Running` | `Expired`).
- `TaskText` is a newtype wrapper around a non-empty `String`. Construct via `TaskText::new(text)` — returns `DomainError::EmptyTaskText` for blank input. The only way `Commands::append_task` passes text into the storage layer.
- `DomainError` covers errors raised by validated domain constructors (`NewFocus::new`, `TaskText::new`). Variants: `EmptyTitle`, `EmptyTaskText`. Maps to `CommandError::BadRequest` at the commands layer. Domain owns the invariants; the commands layer owns the protocol mapping.

## Display spanning

Display spanning uses a Rust-emitted DisplaySpace model so monitor geometry policy is local to the display module, while Animal movement consumes normalized visible monitor regions instead of the raw overlay span.

## Core interaction loop

1. **Pigs roam the screen.** One pig per Focus, wandering at 60px/s with random direction changes every 3–8 s; minimum velocity floor so pigs never look frozen. 4-direction pixel-art sprite sheet (016). Hit-box is 16px larger than sprite (018).
2. **Click a pig.** Pig freezes. `AnimalDetail` card opens near the pig: editable Focus title, duplicate and delete buttons, scrollable Task list (check off, edit text, `✗` to clear), and an "Add task…" input at the bottom. Clock/time controls on the Focus title and each Task open compact timer dropdowns. Click-outside or Escape closes; pig resumes (019, 052).
3. **Drag a pig.** Click-and-hold then move > 4px enters drag mode — pig follows cursor. Release sends pig flying in that direction; friction decelerates it; bounces at region edges. Pure click (< 4px movement) still opens AnimalDetail (020).
4. **Clear a task.** Tap `✗` → `delete_task` Tauri command → markdown updated → pig's task list reflects change.
5. **Add a task.** Type in "Add task…" input in AnimalDetail → Enter → `append_task` Tauri command → markdown updated.
6. **Create a Focus.** *(014)* Tray → "+ New Focus" → small webview form → `create_focus` → new pig spawns. Timer dropdown (No timer / 2m / 4m / 8m / 16m / 32m / Custom) optionally attaches a `FocusTimer` (028). Focus and Task timers can later be started or cleared from `AnimalDetail` (052).
7. **Delete a Focus.** *(015, 027)* Tray → Focus submenu → "Delete…", or the delete button in `AnimalDetail` → `delete_focus` → pig disappears. Asks for confirmation when `widget.confirm_delete` is on.
8. **Configure displays.** *(017, 049)* Preferences → Displays — check/uncheck monitors. Enabled monitors share one spanning overlay window; Animals spawn in the primary display region and move only inside normalized visible monitor regions. Persists in `settings.yaml`. The display module owns monitor geometry, and React owns movement over the emitted DisplaySpace model.
9. **Change settings.** *(026, 032, 050)* Tray → "Settings…" or app menu → "Preferences…" opens the Preferences window. `update_settings` runs the settings workflow: persist `settings.yaml`, commit the in-memory settings, apply widget changes, reapply overlays when displays changed, refresh long-lived settings consumers, and rebuild the tray. Caps and notification toggles take effect without a restart.

## Application

Single Tauri v2 desktop app written in Rust (core + frontend webview). macOS is the primary target; the release workflow also builds unsigned Windows and Linux packages. Surfaces:

1. **Transparent overlay window** — spans the enabled displays, always-on-top, no decorations. Renders pixel pig sprites via React. Click-through for non-pig areas via a Rust polling thread (Tauri `cursor_position()` every 16ms) that toggles `window.set_ignore_cursor_events`.
2. **Tray menu** — Gather Pigs, "Agents as Animals", "+ New Focus", one submenu per Focus (with Delete), an Expired submenu, Settings…, and Quit (plus Open Overlay DevTools in debug builds). The tray icon turns red when over-cap.
3. **New Focus and Preferences windows** — small webviews opened from the tray or app menu.
4. **Agent Hooks window** — an always-on-top webview opened from Window → "Agent Hooks…"; shows the agent hook wiring, the live Agent Sessions, and recent hook firings.

Responsibilities owned by the app:

- Read/write per-Focus markdown files (frontmatter + body).
- Watch `~/.adhd-ranch/focuses/` via `notify` crate; reflect external edits live (pig count updates within 1s).
- Run a 1s timer-expiry loop (`Timers::expire_due`) that persists expired timers and sends notifications. The UI picks up the change through the focuses watcher.
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

An opt-in overlay mode that draws one Animal per running Claude Code session alongside the Focus pigs. It is off by default and controlled by `agents.enabled` in `settings.yaml`.

- **Toggle.** The tray's "Agents as Animals" check item flips `agents.enabled` through the settings workflow (persist, commit, effects, tray rebuild). The agents effect reconciles the hooks only when the value changed, and emits `agent-sessions-changed` either way — what the overlay draws has changed even when the hooks could not be brought into line.
- **Hook install.** Enabling agents at startup, or from the tray, registers command hooks in `~/.claude/settings.json` (or `$CLAUDE_CONFIG_DIR/settings.json`). Each entry runs `adhd-ranch-hook` — a compiled client bundled beside the app's own executable, not a shell script — with the socket path and one verb: `'<client>' '<socket>' working`. Both paths are settled at install time and written verbatim, so the client carries no configuration and reads nothing to find the app.
- **Events.** Five, in `crates/domain/src/agents/claude_code/hooks/events.rs`: `SessionStart` → start, `UserPromptSubmit` → working, `Stop` → idle, `StopFailure` → idle, `SessionEnd` → end. No `PreToolUse`/`PostToolUse` and no subagent events: they fire on every tool call in every session and report a state the turn is already in. `StopFailure` is registered because a turn cut short by an API error never reaches `Stop`, and the session would otherwise stay working for as long as it lives.
- **Settings file.** The file belongs to Claude Code and to whatever else the user has pointed at it. Install is idempotent, keeps key order and permissions, writes through a symlink, and re-reads immediately before the rename so a concurrent write is retried rather than clobbered (three attempts, then an error). A missing file is created. JSON the app cannot parse is left exactly as found and reported as `SettingsNotUnderstood`; it is never replaced with something the app invented.
- **Hook uninstall.** Turning agents off uninstalls the entries and clears the in-memory sessions. An entry is the app's own only if its command matches one of this version's commands character for character; groups emptied that way are dropped, and anything else stays as found. Leaving the entries behind would keep firing the client on every session on the machine with nothing listening for it.
- **Socket.** `~/.adhd-ranch/agent-hooks.sock`, mode 0600 — the file mode is the whole authorisation story. A socket left behind by a crash makes the address look taken; nobody answering on it means debris, so it is removed and rebound. Accept is polled every 25ms rather than blocked on, so shutdown never depends on the socket file still being there, and dropping the server removes it.
- **Client contract.** The client runs inside someone's turn, so every failure path exits 0 and says nothing: no app listening, no socket, unreadable stdin — all silent. It is dependency-free because it is spawned several times per turn. One frame per connection: the verb on its own line, then the agent's JSON verbatim, since the app is the only reader. 250ms write timeout, 1MiB payload ceiling, and a 5s timeout on the installed entry as a ceiling on damage.
- **Sessions.** State lives in memory in `LiveSessions`; nothing about a session is written to disk. A session exists only as long as the process running it, so there is nothing worth surviving a restart, and anything persisted would have to be reconciled against reality on the way back up. A session the app has never heard of is taken at its word, so hooks installed mid-flight still draw what is already running. A payload without a `session_id` or a `cwd` is not the app's to draw.
- **Pens.** One Pen per repository checkout, taken from the session's `cwd`. A `.claude/worktrees/<name>` tail folds back to the checkout it branched from, so two worktrees of one repo share a Pen (`crates/domain/src/session/pen.rs`). Sessions are listed ordered by pen then session id, so animals keep a stable order as sessions come and go.
- **Delivery.** A push, not a watcher: the listener applies the firing in memory and calls back, and the callback emits `agent-sessions-changed`. The overlay re-invokes `list_agent_sessions`, which returns an empty list while `agents.enabled` is false. Nothing is on disk to watch, to go stale, or to clean up when the hooks are removed.
- **Journal.** `HookJournal` decorates the sink and keeps the last 200 firings — what arrived, and whether the app looked any different for it. The payload itself is deliberately not retained: `UserPromptSubmit` carries the text the user just typed, and a debug window is exactly the wrong place for it to resurface.
- **Agent Hooks window.** Window → "Agent Hooks…" opens an always-on-top webview showing the wiring (agents enabled, hooks installed, and the socket, client and settings paths), the live sessions, and every firing including the ones that changed nothing. Every part of the hook path fails silently by design, so this is the one place that tells silence apart from breakage.
- **Overlay.** `projectAnimals` concatenates Focus Animals and Session Animals into one list. Session Animals take `agent:<session_id>` ids, are never selectable and have no detail card, take no clock, and rest while the session is idle. They roam only their Pen; a Focus Animal has the run of the DisplaySpace. Selection is derived from that list, so an Animal that disappears closes its card and narrows the overlay's hit rect in the same render.
- **Windows.** No Unix socket there, so nothing is installed and nothing listens. Inert implementations keep the platform out of every caller rather than leaving a command in a settings file that could only ever fail.

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
  always_on_top: false
  confirm_delete: true
displays:
  enabled: 0
agents:
  enabled: false
pens:
  max_size: 320
```

Settings changed through the app are persisted and applied immediately. Manual file edits are picked up on app restart.

## Scope

- **Current**: User creates Focuses from the app or by hand-editing. Pigs roam the overlay, show Tasks, and reflect markdown changes. Focus and Task timers, caps, and multi-display overlay work. Optional agent pigs for running Claude Code sessions.
- **Not built**: external aggregators (Jira, GitHub), sync, schema migrations.
