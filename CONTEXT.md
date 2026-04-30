# Context

## Guiding metaphor

**Issue tracking for a 5 year old.** A small number of buckets the user cares about right now. When something new happens, an adult (the in-session agent) decides which bucket it goes in. The user clears bullets when they're done. No tickets, no hierarchies, no fields beyond what a five-year-old could parse.

## Ubiquitous Language

### Focus
A top-level item the user is paying attention to. Represents a real-world goal (e.g. "Customer X bug"). Created manually by the user, or via an accepted `new_focus` proposal. Owns a flat list of Tasks.

### Task
A child item under a Focus. Single sentence. Created either by user action or by an accepted `add_task` proposal. Tree is capped at two levels — Focus → Task. No sub-tasks. Removed only by user action.

### Proposal
A pending suggestion from the in-session agent at `/checkpoint` time. Three kinds: `add_task`, `new_focus`, `discard`. Held in a queue until the user accepts or rejects in the widget.

## Persistence

Focus = a self-contained directory under `~/.adhd-ranch/focuses/<slug>/`:

```
focuses/<slug>/
  focus.md            # frontmatter (id, title, description, created_at) + Tasks body
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
- User hand-edits anywhere; file watcher reflects changes.
- Atomic write via tmpfile + rename. `flock` per file.

## Architecture principles

- Functional core, imperative shell. Pure logic; I/O at edges.
- Single source of truth = the per-focus dirs + `proposals.jsonl`. Widget renders a projection.
- **No static binding from agent context to Focus.** Focus is a mental anchor identified by `title` + `description`. The in-session agent decides routing at `/checkpoint` time using the catalog returned by the app.
- **App holds zero LLM logic.** All reasoning happens in the agent already running in the user's session.

## Single-stage flow (v1)

1. **Capture + route (in-session, one shot at `/checkpoint`):**
   - User runs `/checkpoint` in their Claude Code session.
   - Slash command instructs the agent to:
     1. `GET http://127.0.0.1:<port>/focuses` → receive catalog `[{id, title, description}]`.
     2. Compose ONE short sentence (≤12 words) summarizing what was just done.
     3. Decide kind: `add_task` (fits an existing Focus) | `new_focus` (doesn't fit any) | `discard` (not worth tracking).
     4. `POST http://127.0.0.1:<port>/proposals` with `{ kind, target_focus_id?, task_text?, new_focus?, summary, reasoning }`.
   - App appends to `proposals.jsonl`. No app-side LLM call.
   - One proposal per `/checkpoint`. User runs it again if multiple buckets affected.

2. **User confirmation (widget):**
   - Popover layout (top → bottom):
     1. List of Focuses with their Tasks. Each Task has an inline `✗` to clear it.
     2. Collapsed-by-default tray: `📥 N pending` badge. Tap to expand.
     3. Expanded tray: one inline card per proposal showing summary + suggested target Focus + `✓ / ✗ / edit`. Reasoning hidden behind `?`.
   - Accept → mutation applied to Focus dir; row appended to `decisions.jsonl`.
   - Reject → proposal dropped; row appended to `decisions.jsonl`.
   - Edit → small modal lets user override target Focus or text before accepting.
   - No auto-accept. No per-proposal system notifications. System notifications reserved for overload alerts.
   - All Focuses are equal — no "current" highlight. No primary bucket.
   - Empty/first-run: hero card "+ New Focus" + one-line tip ("create a bucket; run /checkpoint in any session").

The bet: a naive "which of these focus titles+descriptions does this summary belong to?" prompt is enough. If it works, this becomes the foundation for v2 (ticket linking, cross-source bridging).

## LLM

The app does **not** call any LLM. The only model in the loop is whatever Claude Code (or future agent host) is already running in the user's session. The slash command is a prompt; quality of routing = quality of that prompt template + the model the user is already using.

No provider config. No API keys.

## Distribution

Two artifacts, macOS-only for v1, no user PATH management:

1. **`Adhd Ranch.app`** — Tauri v2 macOS bundle. Drag to `/Applications`.
2. **`adhd-ranch` skill** — unpacks to `~/.claude/skills/adhd-ranch/` (global, every project) with `SKILL.md`. The slash command lives at `~/.claude/commands/checkpoint.md`.

App and skill are independent: app works UI-only without the skill; skill is useless without the running app (the slash command will error if `~/.adhd-ranch/run/port` is missing or the health check fails — "adhd-ranch not running, please start the app").

## Application

Single Tauri v2 desktop app written in Rust (core + frontend webview). Menubar/tray app via `TrayIconBuilder`; popover window is borderless, always-on-top, positioned near the tray icon by `tauri-plugin-positioner`.

Responsibilities owned by the app:

- Read/write per-Focus markdown files (frontmatter + body).
- Watch `~/.adhd-ranch/focuses/` via `notify` crate; reflect external edits live.
- Serve the localhost HTTP API.
- Enforce caps + emit overload alerts (`tauri-plugin-notification`).

No separate CLI, no shell scripts. Rust core is the single implementation of read/write/cap logic.

## Writers

1. **User** — widget UI or hand-edit markdown.
2. **In-session agent via `/checkpoint`** — HTTP POST `/proposals`, then user accepts/rejects.

That's it. No hooks, no fallbacks.

## IPC

App exposes a localhost HTTP API on `127.0.0.1:<ephemeral-port>`. Port written to `~/.adhd-ranch/run/port` on bind; clients read it. HTTP only — no file fallback. If the app is down, `/checkpoint` errors clearly. Auth: none in v1 (localhost-only bind).

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
alerts:
  system_notifications: true
```

## Scope

- **v1**: macOS only. User creates Focuses (widget button or hand-edit). User runs `/checkpoint` in Claude Code sessions to enqueue proposals. User accepts or rejects in widget.
- **v2 (deferred)**: external aggregators (Jira, GitHub), additional triggers (slash command for new-focus, hooks, scheduled), cross-platform, `merge_focus` and `complete_task` proposal kinds, optional `/usr/local/bin` symlink, schema migrations.
