# Issues — Ralph workflow

Each issue is a self-contained vertical slice an AFK coding agent ("ralph") can grab, finish, and merge without supervision.

## Priority queue

Complete this product slice before picking up architecture-only work.

- [031](031-notification-source-settings.md) — Notification source settings registry (GH #31)

### Architecture queue (deepening, AFK except where noted)

These are not required before 031. Pick one when we choose to spend a slice on internal seams or future-proofing.

- [046](046-timer-expiry-service-extraction.md) — Timer expiry workflow module
- [047](047-storage-transaction-seam.md) — Storage transaction seam for Proposal lifecycle
- [050](050-settings-update-workflow.md) — Settings update workflow module
- [051](051-ranch-animal-vocabulary-seam.md) — RanchAnimal vocabulary seam for future animal types
- [053](053-task-timer-expiry-workflow.md) — Task timer expiry workflow

Completed issue files live in `issues/done/`. Do not pick up files from `issues/done/` or `issues/icebox/`.

Closed without implementation:

- 042 — Unify cap-state behind a single `CapStatus` projection. Premise was wrong; see [ADR-0001](../docs/adr/0001-cap-state-is-already-a-single-projection.md).
- 045 — Symmetric `fixtureReader<T>` + caps fixture parity. Helper too thin to warrant abstraction; test-wrapper deletion folded into 039. See [ADR-0002](../docs/adr/0002-fixture-readers-are-too-thin-for-a-generic-helper.md).

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
