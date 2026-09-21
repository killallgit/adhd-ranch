# adhd-ranch

A desktop ranch you can see from the corner of your eye: each Focus the user holds, and optionally each coding agent at work, lives on screen as a slow, wandering Animal. This file is the domain glossary only. Implementation notes live in `docs/architecture.md`; Claude Code and Codex vocabulary is sourced in `docs/research/claude-codex-session-terminology.md`.

## Guiding metaphor

**A ranch you can see from the corner of your eye.** Pixel animal sprites roam the screen, one per Focus and one per agent at work. They walk slowly in the background. No interruption; no modal. You glance, you know what's on your plate. The Animals don't demand attention; they just exist.

## Language

> Terms below are under review in an active domain-modeling session. Definitions will be rewritten as each term is settled.

### Grouping

**Ranch**:
An arbitrary, user-named grouping of everything the user tracks in one environment, such as "work" or "personal". Every Animal belongs to a Ranch.
_Avoid_: account, workspace, profile

**Animal**:
The sprite the ranch draws and moves. An Animal is a rendering, not a thing the user tracks: Focus and Agent Session each project their own Animals, and the movement layer knows about neither. The same word is used in the UI and in code. Settled by ADR-0001.
_Avoid_: RanchAnimal, Sprite, Pig — all for the general concept

**Species**:
The kind of an Animal, which decides how it looks and moves. Pig is the only Species today.

**Motion**:
How an Animal is moving right now: walking or resting. Motion is the renderer's vocabulary, so each half maps its own state into it — a Focus rests because its Timer expired, an Agent Session rests because it is between turns, and neither knows why the other one does. A new Motion can be added for one half without touching the other.
_Avoid_: activity (that belongs to Agent Session), expired (that belongs to Timer)

### Work

**Focus**:
A goal the user creates and is paying attention to, such as "Customer X bug". Shown as an Animal.
_Avoid_: project, goal (as the term)

**Task**:
One unit of work inside a Focus.

**Complete** (a Task):
Mark a Task done. The Task stays in its Focus, shown as done.
_Avoid_: check off, done (as a verb), finish

**Delete** (a Task):
Remove a Task from its Focus entirely.
_Avoid_: clear, remove

**Timer**:
An optional countdown attached to exactly one Focus or one Task. Say "Focus Timer" or "Task Timer" only when the owner matters.
_Avoid_: FocusTimer (for the general concept), countdown

**Timer Owner**:
The Focus or the Task a Timer is attached to. Every Timer has exactly one Owner, and every Owner has at most one Timer.

**Expired**:
The state of a Timer whose countdown has run out. Only Timers expire.

**Revive** (a Focus):
Return a Focus whose Timer expired to normal by clearing that Timer. Adding a Task revives its Focus. Tasks are never revived: an expired Task Timer stays until the user clears or restarts it.
_Avoid_: reset, un-expire, restart

**Clear** (a Timer):
Remove a Timer from its owner. Clearing an expired Timer returns its owner to normal.
_Avoid_: using "clear" for Tasks or Focuses

**TimerPreset**:
A named duration choice for a Timer: 2m, 4m, 8m, 16m, 32m, or Custom minutes.

**Limit**:
A soft threshold on how much work is open at once, such as how many Focuses exist or how many Tasks one Focus holds. Going past a Limit blocks nothing. Agent Sessions don't count toward Limits.
_Avoid_: cap, max

**Over Limit**:
The state of having gone past any Limit. The app warns the user once when it happens.
_Avoid_: over-cap, overload

### Agents

**Harness**:
A coding-agent tool that runs Agent Sessions and reports on them through hooks, such as Claude Code or Codex. Claude Code is the first Harness.
_Avoid_: provider, vendor, client

**Agent Session**:
One run of a coding agent in a Harness, together with the worktree and repo it runs in, identified by that Harness's session id. Shown as an Animal when Agents as Animals is on. The user never creates one; the app sees it through the Harness. Codex also calls this a thread.
_Avoid_: Session (alone), agent (for the run)

**Session Activity**:
Whether an Agent Session is mid-turn: Working from the moment a prompt is submitted until the turn ends, Idle otherwise. There is no clock in this and no expiry — unlike a Timer, an Activity never runs down, and a Session can be Working for a second or an hour. It is what an Agent Session maps into a Motion.
_Avoid_: status, state, busy, running

**Hook Firing**:
One report from a Harness that something happened in an Agent Session: which event, which Session, and what the ranch made of it. Firings arrive unasked — the ranch never polls a Harness, and never reads a Session's transcript to find out what it said. A Firing the ranch could not read, or that changed nothing, is still a Firing, and the Agent Hooks window shows it either way.
_Avoid_: hook event, hook call, message, notification

**Pen**:
The area of the ranch holding every Animal from one repository checkout. One Pen per checkout; worktrees share the Pen of the checkout they branched from. Pens exist only for Agent Sessions — a Focus Animal has the run of the DisplaySpace — and the movement layer is handed regions, never Pens.
_Avoid_: pasture, paddock, group

**Agents as Animals**:
The Setting that shows each Agent Session as an Animal. Off by default.

### Screen

**DisplaySpace**:
The enabled displays, treated as the visible regions where Animals may move. Gaps between displays are not part of it.

**Gather**:
Bring Animals back onto the primary display, either one Animal or all of them.
_Avoid_: wrangle, round up, summon

### App

**Settings**:
The user's stored configuration for the app.
_Avoid_: Preferences, config (Preferences is only the macOS menu label for the window that edits Settings)
