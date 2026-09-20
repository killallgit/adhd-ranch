---
name: ship-pr
description: Take a finished branch all the way to merged. Identifies and links the issue, runs the local gate, rebases onto the branch's real base, opens the PR, obtains a review (CodeRabbit, or a fresh-context local reviewer when CodeRabbit is rate-limited), drives every finding to resolution, squash-merges, and confirms the issue closed. Use when a branch is done and needs to become a merged PR, when asked to "ship it", "open the PR", "get this merged", or when a PR is open but stalled with no review on it.
user-invocable: true
argument-hint: "[branch or PR number] — defaults to the current branch"
---

# Ship a PR

Seven gates, in order. A gate that is not green does not get passed — it gets fixed, or it stops and the human hears why. Never open a PR to "see if CI catches it," and never merge on the assumption a gate would have passed.

`task check` and `task ci` are the same command. A rebased branch that is green locally is green in CI.

## Gate 1 — the issue is identified and its paperwork is in the diff

A PR does not need an issue. But if one exists, it gets linked, and it gets closed by the merge.

This repo tracks work in two places that are easy to confuse:

- **The issue file** — `issues/NNN-slug.md`, the contract, holding the Completion promise.
- **The GitHub issue** — optional, and only some files have one. Titles carry the file number, so the mapping is discoverable:

```bash
gh issue list --state open --search "<NNN>" --json number,title
```

Match on the title's leading number (`031 — Notification source settings registry` is the GitHub issue for `issues/031-notification-source-settings.md`). No match means there is no GitHub issue — that is fine, and not something to invent one for.

When an issue file exists, the implementing PR also retires it, in the same diff:

```bash
git mv issues/NNN-slug.md issues/done/NNN-slug.md
```

and drops its line from the priority queue in `issues/README.md`. This is code work — if you are orchestrating rather than implementing, delegate it; do not edit it yourself.

**For a file-only issue, the `git mv` to `done/` is the close.** Most issues here have no GitHub issue, and that is the intended state, not a gap to fill. Do not open a GitHub issue so that `Closes #N` has something to point at — a stub that restates the file gives you two records that drift, and the file is the one holding the Completion promise. Archiving it in the merge commit closes it as definitively as GitHub would.

If there is no issue at all, say so in the PR body in one line and carry on. An unlinked PR is allowed; an untracked one is not.

## Gate 2 — the local gate is green

```bash
task check
```

Covers lint, typecheck, tests, and the ts-rs drift check. Run it on the complete diff, including the Gate 1 file move.

When the diff touches `src-tauri/**`, `crates/**`, `package.json`, `package-lock.json`, `Cargo.toml`, or `Cargo.lock`, also run:

```bash
task check:windows
```

Those are the paths that trigger `.github/workflows/windows.yml`. It is the only local way to catch a `#[cfg(unix)]` symbol used without a gate, which otherwise surfaces minutes after a push.

Fix failures at the cause. Never weaken a test, silence a lint, or narrow a check to get green.

## Gate 3 — the branch is current with its base

Read the base; never assume `main`. A PR may target a stacked branch.

```bash
git fetch origin
gh pr view --json baseRefName --jq .baseRefName 2>/dev/null \
  || gh repo view --json defaultBranchRef --jq .defaultBranchRef.name
```

Rebase — never merge the base in:

```bash
git rebase origin/<base>
```

**Re-run Gate 2 after the rebase.** A rebase pulls in other people's commits and can break a branch that was green a minute ago. A branch validated before its rebase is not validated.

If a conflict touches a contract an earlier slice guaranteed, stop and flag the human. Do not paper over it.

## Gate 4 — the PR exists and carries its links

```bash
git push --force-with-lease -u origin HEAD
```

Always `--force-with-lease`, never bare `--force`.

```bash
gh pr create --base <base> --title "[<slug>]: <title>" --body "<body>"
```

- **Title:** `[<slug>]: <imperative sentence>`. CodeRabbit warns on titles that miss this shape.
- **Body:** link the issue file (`issues/NNN-slug.md`), quote its **Completion promise** verbatim, and — when a GitHub issue exists — include a closing keyword on its own line:

  ```
  Closes #NNN
  ```

  There is no repository setting that closes issues on merge. GitHub closes an issue only when the PR body carries `Closes`/`Fixes`/`Resolves #N`, or someone links it by hand in the Development sidebar. Omit the keyword and the issue silently stays open. It only works for issues in this repository.
- No co-author or attribution lines.

## Gate 5 — at least one review exists

Every PR gets a review. An agent never decides on its own to skip one. Only an explicit human instruction in this run skips it, and then the reason goes in the PR body so the record shows it was a decision rather than an omission.

CodeRabbit does **not** review this repo automatically — `auto_review.enabled` is off in `.coderabbit.yaml`, deliberately, so the limited OSS quota goes to reviews we actually asked for. Nothing arrives unless you ask. Ask once, after the PR is up:

```bash
gh pr comment <N> --body "@coderabbitai review"
```

Wait about three minutes, then check whether a review actually landed:

```bash
gh pr view <N> --json reviews --jq '[.reviews[] | select(.author.login=="coderabbitai")] | length'
```

`1` or more means a real review. `0` means it did not review — find out why:

```bash
gh pr view <N> --json comments --jq '[.comments[] | select(.body | contains("rate limited by coderabbit.ai"))] | length'
```

- **Non-zero: rate-limited.** CodeRabbit still posts a summary when it is rate-limited, so a summary comment is not evidence of a review. Do not wait for the quota to reset and do not re-request in a loop — that burns the quota without producing a review. Go to the local reviewer.
- **Zero, and still no review:** give it one more wait, then go to the local reviewer.

Incremental review is off too. Pushing fixes does **not** get them re-reviewed — if a change after the first review is worth a second look, request one explicitly.

CodeRabbit reads `.coderabbit.yaml` from the **base** branch, not the PR. A PR that changes `auto_review` is still governed by whatever is on `main`, so it may review itself automatically. Check for an in-progress review before requesting one; asking on top of a running review spends two from the quota.

### The local reviewer

A separate agent, fresh context, **never the agent that wrote the code**. An author reviewing their own diff re-reads their intentions instead of the code, and will not see what they failed to consider.

Brief it with:

- The diff — `git diff origin/<base>...HEAD`.
- `CLAUDE.md`, for the rules the repo actually enforces.
- **`.coderabbit.yaml`.** Its `path_instructions` are this repo's codified review standard, per path — pure `crates/domain`, trait boundaries out of `storage`, injected dependencies in `commands`, view-only React components, no `unwrap`/`expect`/`panic!` outside tests, no `static mut` or `OnceCell` holding shared mutable state, tests asserting on types rather than message strings. Apply them as written, so the fallback review is equivalent to the primary one and not a weaker substitute.
- The issue's Completion promise, as the bar the change must observably meet.

Hold it to the same tone the config sets: name the consequence — a wrong result, a boundary crossed, a `CLAUDE.md` rule broken, a failure mode the tests miss. No praise, no restating the code, nothing about naming taste or missing doc comments.

Post its findings as a PR comment, so the review is on the record rather than only in a transcript.

## Gate 6 — every finding is resolved

Each finding ends as **fixed** or **rebutted**. Nothing is left hanging.

Fixes are code work: hand them to the implementing agent, not to the reviewer and not to the orchestrator. Then re-run Gate 2 and Gate 3, and **push** — Gate 7 reads the *remote* head, so a fix that only exists locally is a fix that does not get merged:

```bash
git push --force-with-lease
git rev-parse HEAD
gh api repos/<owner>/<repo>/pulls/<N> --jq .head.sha
```

Those two SHAs must match before Gate 7. If they differ, the PR still points at the unfixed commit. Read the head with `gh api`, not `gh pr view --json headRefOid` — that one serves a cached value and can report the pre-push SHA for a while after a successful push, which looks exactly like a failed push.

A rebuttal has to say why the finding is *wrong* — the reviewer misread the code, the case it describes cannot occur, the rule it cites does not apply here. "Stylistic", "pre-existing", or "I disagree" is not a rebuttal. "Out of scope" only counts when the concern is real but belongs to another slice, and then it needs an issue file, not a dismissal.

Post every rebuttal as a reply on the PR thread with its reasoning. Since the merge does not wait for a human, the PR thread is the only record of what was argued away — write it so it can be audited later.

## Gate 7 — merge, then confirm the issue closed

Confirm CI is actually green on the pushed head:

```bash
gh pr checks <N>
```

Then, once Gates 1–6 all hold:

```bash
gh pr merge <N> --squash --delete-branch
```

Squash-merge, one issue to one commit on `main`.

Nothing waits for a human here, so the PR thread is the whole record. A reader coming back to this merge should be able to see which review ran, what it found, and what was argued away — Gate 6 is what makes that true.

Then verify the close actually happened rather than assuming it:

```bash
gh issue view <NNN> --json state,closedByPullRequestsReferences
```

`state: CLOSED` with this PR listed means the linkage worked. Still `OPEN` means the keyword was missing or malformed — close it by hand and say so:

```bash
gh issue close <NNN> --comment "Shipped in #<N>."
```

Then move to the next unblocked issue.

**Do not merge** when any of these is true — stop and report instead:

- `gh pr checks` is red or still running.
- No review exists and no human authorized skipping one.
- A finding is open, or was rebutted on grounds you could not state.
- The rebase raised a conflict touching a guaranteed contract.
- The Completion promise is not observably true, even with everything else green.
