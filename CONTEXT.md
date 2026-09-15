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
The creature on screen that stands for one thing the user keeps an eye on: either a Focus or an Agent Session. The same word is used in the UI and in code.
_Avoid_: RanchAnimal, Pig (for the general concept)

**Species**:
The kind of an Animal, which decides how it looks and moves. Pig is the only Species today.

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

**Expired**:
The state of a Timer whose countdown has run out. Only Timers expire.

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
