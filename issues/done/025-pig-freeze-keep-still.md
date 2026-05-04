# 025 — Pig freeze regression + keep still toggle

## Parent PRD

PRD.md §FR3 (pig UI)

## Problem

### Regression: pig stops wandering after interaction

After clicking (opening PigDetail) or dragging a pig, it does not resume autonomous wandering. Confirmed at runtime.

Likely cause: during drag, `tickPig` is skipped entirely for the dragged pig (so `nextTurnAt` and `lastFrameAt` go stale). On drag end the pig's timers are in the past — `now >= nextTurnAt` fires immediately, picks a new random velocity, but something in the resume path leaves the pig stuck.

Also possible: `selectedIdRef` not clearing cleanly after PigDetail closes.

Fix: during drag, advance `nextTurnAt` and `lastFrameAt` on the dragged pig the same way frozen pigs advance them (add `dt` each frame). This keeps timers current so the pig resumes cleanly.

### Feature: Keep Still toggle

Allow user to pin a pig in place. Pig stops wandering until unpinned.

- Add "Keep Still" / "Resume" toggle in `PigDetail` card
- Pinned pig: `tickPig` returns pig unchanged (position frozen), plus a subtle visual indicator
- Unpinning resumes from current position
- Pinned state is ephemeral — does not persist across app restarts

## Completion promise

After closing PigDetail or releasing a drag, the pig resumes wandering. User can also explicitly pin/unpin a pig via PigDetail.

## Acceptance criteria

- [ ] After closing PigDetail, pig resumes wandering within one rAF cycle
- [ ] After releasing a drag, pig coasts with toss velocity and resumes wandering
- [ ] "Keep Still" toggle in PigDetail pins pig at its current position
- [ ] "Resume" unpins; pig resumes wandering
- [ ] Pinned state not persisted across restarts
- [ ] `task check` green

## Blocked by

None.
