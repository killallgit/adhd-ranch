# PRD — adhd-ranch

**Status:** Draft v1
**Owner:** ryan
**Last updated:** 2026-04-30

---

## Problem

A developer working across many microservices, agents, repos, and tickets loses the thread. Tools that try to help (Jira, GitHub, Linear) demand structure and time the user can't spend mid-flight. Notes, todos, and ticket trees become more cognitive load, not less. Agentic tools accelerate this — Claude Code spawns a tree of in-flight work with no central place to track "what am I actually focused on right now?"

The problem isn't capacity. It's *signal compression*. The user needs a few buckets they can glance at and a way for those buckets to stay current without manual upkeep.

## Why now

Agentic coding is the multiplier. A user with 3+ Claude Code sessions running at once has exactly the context-fragmentation this addresses. The skill/slash-command/hook surfaces of Claude Code make it feasible to get free, in-context summaries without separate LLM cost.

## Target user

Solo developer (initially: the author) who:
- Uses Claude Code daily across multiple repos.
- Has ADHD-flavored attention, or just runs more parallel work than working memory accommodates.
- Owns a Mac.
- Will hand-edit markdown if it's faster than clicking.

## Guiding metaphor

**Issue tracking for a 5 year old.** A small number of buckets. An adult sorts new things into them. The 5 year old clears bullets when they're done.

## Goals (v1)

1. Always-visible widget showing a short list of current Focuses, each with up to a few Tasks.
2. Zero-friction capture: user runs `/checkpoint` in any Claude Code session; the in-session agent decides what bucket the moment belongs to and proposes an update.
3. Human in the loop: every agent-suggested change is a *proposal* the user must accept.
4. Hard caps (5 Focuses, 7 Tasks per Focus) with overload alerts when exceeded.
5. Markdown is the source of truth — user can hand-edit any Focus file in any editor.

## Non-goals (v1)

- Mirroring Jira / GitHub. No external aggregators.
- Auto-completion of Tasks (`complete_task` deferred).
- Auto-merge of Focuses (`merge_focus` deferred).
- Cross-platform (Linux/Windows). macOS only.
- Multi-user / sync / cloud.
- Token-economical automatic capture (no Stop hook capture; user-triggered only).
- Notification hook forwarding ("Claude needs you" relay) — deferred to v1.x.

## User stories

- **US1.** As a developer with three Claude sessions running, I glance at my menubar and see three buckets: "Customer X bug", "API refactor", "Doc cleanup". Each bucket lists 2–4 plain-English bullets. I never need to wonder "wait, what was I doing in that other terminal?"
- **US2.** Mid-session, I run `/checkpoint`. The agent reads my buckets, summarizes what we just did in one sentence, picks the right bucket, and submits a proposal. I tap accept in the widget. Done.
- **US3.** I run `/checkpoint` on something that doesn't fit any bucket. The agent proposes a new Focus with a suggested title + description. I accept or edit the title.
- **US4.** I finish a Task. I tap `✗` next to its bullet. Bullet disappears. No "are you sure".
- **US5.** I add a Task by hand: open `~/.adhd-ranch/focuses/customer-x-bug/focus.md` in vim, append `- [ ] release staging`. Save. Widget reflects it within seconds.
- **US6.** I have 6 Focuses open. The widget shows a red badge. macOS notification: "too much going on — trim 1". I delete a stale Focus.

## Functional requirements

### FR1 — Focus storage
Each Focus is a directory under `~/.adhd-ranch/focuses/<slug>/` containing `focus.md` with YAML frontmatter (`id`, `title`, `description`, `created_at`) and a body of `- [ ]` bullets. Plain text only.

### FR2 — Tauri menubar app
- macOS tray icon via `TrayIconBuilder`.
- Borderless, always-on-top popover positioned near the tray icon.
- File watcher (`notify`) on `~/.adhd-ranch/focuses/`; UI re-renders on disk changes.
- macOS system notifications via `tauri-plugin-notification`.

### FR3 — Widget UI
Popover layout (top → bottom):
1. List of Focuses, each with its Tasks. Each Task has inline `✗`. Each Focus has `…` → `Delete` (with confirm).
2. "+ New Focus" button (always visible — also empty-state hero when no Focuses exist).
3. Collapsed `📥 N pending` tray (badge with proposal count). Tap to expand.
4. Expanded tray: per proposal, an inline card with summary + suggested target Focus + `✓ / ✗ / edit`. Reasoning hidden behind `?`.

### FR4 — HTTP API
Localhost-only, ephemeral port written to `~/.adhd-ranch/run/port`.

| Method | Path                       | Purpose                                              |
|--------|----------------------------|------------------------------------------------------|
| GET    | `/health`                  | liveness for clients                                 |
| GET    | `/focuses`                 | catalog `[{id, title, description}]` for the agent   |
| POST   | `/focuses`                 | create Focus `{title, description}`                  |
| DELETE | `/focuses/{id}`            | remove Focus dir                                     |
| POST   | `/focuses/{id}/tasks`      | append Task `{text}` (used by widget UI on accept)   |
| DELETE | `/focuses/{id}/tasks/{idx}`| remove Task by 0-based bullet index                  |
| POST   | `/proposals`               | enqueue proposal from `/checkpoint` agent            |
| GET    | `/proposals`               | list pending proposals                               |
| POST   | `/proposals/{id}/accept`   | accept; mutation applied; row appended to decisions  |
| POST   | `/proposals/{id}/reject`   | reject; row appended to decisions                    |

No auth.

### FR5 — `/checkpoint` slash command
- Installed at `~/.claude/commands/checkpoint.md` (global).
- Reads port from `~/.adhd-ranch/run/port`. If missing or `/health` fails, errors clearly: "adhd-ranch not running, please start the app".
- Instructs the in-session agent to:
  1. `GET /focuses`.
  2. Compose ONE sentence (≤ 12 words) summarizing the current work.
  3. Decide kind: `add_task` | `new_focus` | `discard`.
  4. `POST /proposals` with `{ kind, target_focus_id?, task_text?, new_focus?, summary, reasoning }`.
- Single proposal per `/checkpoint`. Agent runs it again if multiple buckets affected.

### FR6 — Caps
- `MAX_FOCUSES = 5`, `MAX_TASKS_PER_FOCUS = 7` (configurable in `settings.yaml`).
- Writes that exceed caps still succeed (no hard rejection — markdown is canonical and user might edit anyway), but flip an over-cap flag.
- Widget shows red badge while over.
- macOS notification fires once per `under → over` transition.

### FR7 — Configuration
`~/.adhd-ranch/settings.yaml`:
```yaml
caps:
  max_focuses: 5
  max_tasks_per_focus: 7
alerts:
  system_notifications: true
```
Defaults applied for any missing keys.

### FR8 — Audit log
Every accepted/rejected proposal appended to `~/.adhd-ranch/decisions.jsonl` with `{ts, proposal_id, decision, reasoning, target}`. Useful for tuning the routing prompt later.

## Non-functional requirements

- **Latency:** widget reflects file changes within 1s of save. HTTP `POST /proposals` returns within 50ms.
- **Reliability:** atomic writes (tmpfile + rename), per-file `flock`. No partial-write corruption visible to readers.
- **Footprint:** Tauri release build < 20 MB on disk; idle RAM < 50 MB.
- **Resilience:** if the app crashes, no markdown corruption (atomic writes); on relaunch the app reads from disk and resumes.
- **Privacy:** zero outbound network. No telemetry, no LLM calls, no tickets posted anywhere.

## Success metrics (v1)

This is a personal tool first. Metrics are subjective:

- Author uses `/checkpoint` ≥ 5 times per work day for two weeks without abandoning.
- Author can name their currently active Focuses without opening any other tool.
- Routing accuracy ≥ 70% (proposals accepted as-is, no edit) — measured from `decisions.jsonl`.
- Proposals are useful enough that the user prefers them to manually editing markdown for at least half the writes.

If those hold, the bet pays. If not, the audit log gives labelled data to improve the prompt or fold in a Pass-2 routing call.

## Out of scope / v2 ideas

- `merge_focus` and `complete_task` proposal kinds.
- Notification-hook forwarding ("Claude needs you" badge).
- Slash command for `/new-focus`.
- `/usr/local/bin/adhd-ranch` symlink for shell shortcuts.
- Linux + Windows ports.
- External aggregators: Jira, GitHub, Linear.
- Schema migrations (when frontmatter changes).
- Rollup pass with separate LLM call (only if naive in-session routing proves insufficient).
- Multi-machine sync (probably never — markdown in a synced dir like iCloud or git-managed dotfiles handles this for free).

## Open questions / risks

- **R1.** Routing accuracy of a one-shot prompt may be poor when Focuses overlap conceptually. Mitigation: user can edit proposal target; audit log captures every miss for prompt tuning.
- **R2.** The user might not run `/checkpoint` often enough. Mitigation: a future automatic/idle trigger can plug in via the same HTTP entrypoint without architectural change. Not v1.
- **R3.** The 5/7 caps might feel too tight. Mitigation: configurable; trivial to bump.
- **R4.** No real auth on the HTTP API. Any local process can write. Acceptable for a personal Mac; revisit if shared.
- **R5.** `flock` semantics on macOS over network filesystems (iCloud Drive, NFS) are not perfect. Mitigation: keep `~/.adhd-ranch/` on local disk in v1; document.

## Implementation phases

1. **Phase 0 — Skeleton.** Tauri project, tray icon, popover window, empty `~/.adhd-ranch/` structure, hard-coded Focus list.
2. **Phase 1 — Storage.** Markdown read/write, file watcher, schema parser, atomic writes, settings load.
3. **Phase 2 — HTTP API.** All routes from FR4. Proposals queue. Accept/reject paths.
4. **Phase 3 — Slash command.** `~/.claude/commands/checkpoint.md` with prompt template; manual end-to-end test.
5. **Phase 4 — Caps + alerts.** Over-cap detection, system notifications, widget badge.
6. **Phase 5 — Polish.** Edit-proposal modal, delete-Focus confirm, empty state, decisions audit log, packaging (`.app`, dmg).

Each phase is independently testable. v1 ships at end of Phase 5.
