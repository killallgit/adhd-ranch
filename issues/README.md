# Issues — Ralph workflow

Each issue is a self-contained vertical slice an AFK coding agent ("ralph") can grab, finish, and merge without supervision.

## Priority queue

Complete these before picking up any other open issue. All are unblocked and independent — grab any order.

- [029](029-timer-expiry-and-notification-interface.md) — Timer expiry + NotificationSource interface (GH #29)
- [030](030-pig-growth-expired-visual-tray-list.md) — Pig scale growth + expired visual + tray list (GH #30)
- [031](031-notification-settings-tray.md) — Notification settings in tray (GH #31) — **blocked by 029**

### Architecture queue (deepening, AFK except where noted)

Frontend reader collapse:

- [037](037-collapse-focus-reader.md) — Collapse `FocusReader` into `PolledReader<Focus[]>` — **done**
- [038](038-collapse-proposal-reader.md) — Collapse `ProposalReader` into `PolledReader<Proposal[]>` — **done**
- [039](039-unify-caps-on-polled-reader.md) — Unify `useCaps` on `usePolledReader<Caps>` — **done**
- [040](040-generic-tauri-reader-factory.md) — Generic `tauriReader<T>` factory — **done**

Domain seams:

- [041](041-focus-writer-write-outcome.md) — `FocusWriter` returns `WriteOutcome` (hands off to 029)
- [043](043-proposal-lifecycle-atomicity-design.md) — `ProposalLifecycle::accept` atomicity — **done** by [ADR-0003](../docs/adr/0003-storage-transaction-seam-for-proposal-lifecycle.md)
- [044](044-domain-timer-ticker.md) — Carve domain `TimerTicker` pure module — **done** (merged, consumed by 029)
- [046](046-timer-expiry-service-extraction.md) — Timer expiry workflow module
- [047](047-storage-transaction-seam.md) — Storage transaction seam for Proposal lifecycle
- [048](048-focus-document-mutation-module.md) — FocusDocument mutation module
- [049](049-display-space-pig-movement-seam.md) — Display-space seam for Pig movement
- [050](050-settings-update-workflow.md) — Settings update workflow module

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
