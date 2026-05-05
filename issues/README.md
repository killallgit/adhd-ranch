# Issues — Ralph workflow

Each issue is a self-contained vertical slice an AFK coding agent ("ralph") can grab, finish, and merge without supervision.

## Priority queue

Complete these before picking up any other open issue. All are unblocked and independent — grab any order.

- [029](029-timer-expiry-and-notification-interface.md) — Timer expiry + NotificationSource interface (GH #29)
- [030](030-pig-growth-expired-visual-tray-list.md) — Pig scale growth + expired visual + tray list (GH #30)
- [031](031-notification-settings-tray.md) — Notification settings in tray (GH #31) — **blocked by 029**

### Icebox (deferred)

Multi-monitor work paused. Files moved to `issues/icebox/`; GitHub issues carry the `icebox` label.

- [icebox/021](icebox/021-all-monitors-default.md) — All monitors enabled by default (GH #19)
- [icebox/022](icebox/022-wrangle.md) — Wrangle pig + wrangle all (GH #23)

## How to pick up an issue

1. Pick the lowest-numbered open issue whose **Blocked by** entries are all merged to `main`.
2. Sync with `main`:
   ```sh
   git fetch origin
   git checkout main && git pull --ff-only
   git checkout -b <type>/<slug>
   ```
   Branch types: `feat`, `chore`, `fix`, `spike`, `hotfix`. Slug = short kebab-case from the issue title.
3. Treat **Completion promise** as the contract. **Acceptance criteria** is the checklist that proves the contract.
4. Implement. Keep diffs small enough a human can review in ≤15 minutes.

## Definition of done

An issue is done when ALL of:

- Every box in **Acceptance criteria** is ticked.
- **Completion promise** is observably true on the merged commit.
- `task check` green on the branch after a final rebase onto latest `main`.
- PR opened, CI green, merged to `main`.

If any box can't be ticked, the issue is **not** done. Flag the human; don't silently skip.

## Staying current with `main`

- Always **rebase** onto `main`. Never merge `main` into the feature branch.
- Before opening the PR: `git fetch origin && git rebase origin/main && task check`.
- If a prior slice lands on `main` mid-flight, rebase immediately and re-run `task check`.
- If a conflict touches a contract guaranteed by an earlier slice, stop and flag the human — don't paper over it.

## PR conventions

- Title format: `[<slug>]: <issue title>` (matches EyePop convention; not Conventional Commits).
- Body: link the issue file (`issues/NNN-...md`) and quote the **Completion promise** verbatim.
- No co-author attribution.
- CI must be green before merge.
- Squash-merge by default; one issue = one commit on `main`.

## After merge

- Delete the feature branch.
- Move to the next unblocked issue.

## Issue file shape

```
## Parent PRD
## What to build
## Completion promise   ← single sentence, observable, non-negotiable
## Acceptance criteria  ← checkboxes that prove the promise
## Blocked by
## User stories addressed
```

emit: "RALPH DONE" when all tasks completed
