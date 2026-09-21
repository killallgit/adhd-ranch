# 062 — One hook client path, and reclaim the orphans

## Parent PRD

PRD.md §FR6 (Configuration) and §Phase 7, and ADR-0002's fourth action item — the only one with a
live bug attached rather than a design change.

## What to build

`paths::hook_client_bin()` derives the client from `std::env::current_exe()`. The resulting path
is written verbatim into `~/.claude/settings.json`, and `ClaudeCodeHooks::uninstall` matches that
string character for character. So every checkout the app has ever run from leaves a permanent
entry set that the app can no longer remove.

This is not hypothetical. On the developer machine right now, all five events carry two entries
pointed at the same socket:

```
SessionStart  '…/adhd-ranch/.claude/worktrees/animals-by-sessions-7d5c3c/target/debug/adhd-ranch-hook' … start
SessionStart  '…/adhd-ranch/target/debug/adhd-ranch-hook'                                              … start
```

The app log shows what that costs: **26 firings, 13 of them `changed=false`** — exactly half.
Every hook is delivered twice and absorbed only because `record` happens to be idempotent. That
holds for levels and will not hold for 061's change events, where a doubled `Activity` is a
doubled animation.

### Target shape

**Forward fix — a stable path.** `hook_client_bin()` returns a fixed location under the data root
(`~/.adhd-ranch/bin/adhd-ranch-hook`), so every build of the app writes the same string and
uninstall matches it.

Putting the binary there is the running app's job at startup, not the installer's. That keeps
`install_writes_nothing_but_the_settings_file` and its reasoning intact — "an installer that made
them would be making promises about a ranch that may not even be running" — while the app makes
the same promise about the client that it already makes about the socket. Copy from
`current_exe().parent()/adhd-ranch-hook` when the destination is missing or older than the
source, so a dev rebuild is picked up. If the source is absent, `claude_hook::reconcile` fails
loudly; it already returns `Result<(), String>` precisely so switching agents on can fail.

**Reclaim — the orphans already out there.** At install, also remove any entry whose command is
`'<any path>/adhd-ranch-hook' '<our socket>' <one of our verbs>`.

This sits against the project's own rule that we uninstall exactly what we install, with no
heuristics — so be explicit about why it holds: the socket path, the verb set and the basename
are all strings the ranch wrote, in the shape the ranch writes them. Nothing else on the machine
produces that. It is structural recognition of our own output, not a guess about somebody else's
entry. A command differing anywhere else is left alone.

### Out of scope

- Any change to the frame, the event table, or what the ranch does with a firing.
- Uninstalling entries pointed at a *different* socket. Two ranches with two data roots is not a
  configuration we support, and quietly deleting the other one's hooks would be the bug this
  issue is about, pointed the other way.
- Windows. There is no socket to install against and no hook is registered there.

## Completion promise

An entry the ranch wrote can always be removed by the ranch, and one firing produces one delivery.

## Acceptance criteria

- [ ] `hook_client_bin()` names a path that does not vary with where the app runs from
- [ ] The app puts the client at that path at startup, copying when missing or stale
- [ ] `install` still writes nothing but the settings file — the existing test passes unchanged,
      or is replaced with its reasoning restated
- [ ] `reconcile` returns an error, not a log line, when the client cannot be placed
- [ ] Install removes entries matching our command shape at any client path on our socket
- [ ] Test: an entry naming a different socket survives install untouched
- [ ] Test: an entry naming a different binary survives install untouched
- [ ] Test: install over the two-entry-set state leaves exactly one set per event
- [ ] Verified on the developer machine: `~/.claude/settings.json` holds one adhd-ranch entry per
      event, and the log stops showing a `changed=false` twin for every firing
- [ ] `task check` and `task check:windows` green

## Blocked by

None. Independent of 059–061, and worth taking first: it halves the hook traffic those slices are
reasoning about.

## User stories addressed

- "When I turn agents off, they are off — not still firing at a socket from a checkout I deleted."
- "When I move the app, my settings file does not accumulate another copy of it."
