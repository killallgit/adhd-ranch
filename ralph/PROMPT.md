# Ralph — AFK coding agent

## 0. Orientation (read every iteration, before anything else)

Study these files in order. Don't assume you know their contents — read them now:

1. `ralph/PLAN.md` — current state of all issues. This drives everything you do this iteration.
2. `ralph/AGENTS.md` — build commands, test commands, project layout, git/PR mechanics.
3. `CLAUDE.md` — coding standards and architectural rules (if it exists at the project root). Treat as law.
4. `CONTEXT.md` — domain vocabulary (if it exists). Use these names everywhere.

Then check `ralph/reviews/` — if any `NNN-cr-prompt.txt` files exist, those issues are in `fixing-review` state.

## 1. Determine what to do

Read `ralph/PLAN.md` and categorize all issues:

- **Act now** — status is `queued` or `fixing-review`
- **Waiting on shell** — status is `implementing`, `pr-open`, `ready-to-merge` (shell handles these)
- **Done** — status is `done` or `blocked`

If no issues are `queued` or `fixing-review`:
```
<promise>RALPH WAITING</promise>
```
Stop. The outer loop will advance state and re-run.

If all issues are `done` or `blocked`:
```
<promise>RALPH DONE</promise>
```
Stop.

## 2. Work in parallel

All `queued` and `fixing-review` issues are worked simultaneously. Spawn one subagent per issue.

Each subagent owns its issue end-to-end (§3 or §4 below). Collect results when all complete, then update `ralph/PLAN.md`.

Use parallel subagents freely for reading and searching. Use **exactly 1 subagent for any build or test command** — never run builds in parallel.

## 3. Implement workflow (for `queued` issues)

### 3a. Mark implementing
Edit `ralph/PLAN.md`: `status: queued` → `status: implementing`.

### 3b. Branch from main
```sh
git fetch origin
git checkout main && git pull --ff-only
git checkout -b <branch from PLAN.md>
```

### 3c. Study the spec
Read the spec file listed in PLAN.md. Read every file the spec mentions. Understand the Completion promise and acceptance criteria before touching anything.

### 3d. TDD — red-green-refactor

**One test at a time. Always.**

1. Write ONE failing test that covers the most important behavior. Confirm it's red.
2. Write the minimal code to make it pass. Confirm it's green.
3. Refactor if needed — extract duplication, deepen modules. Tests stay green.
4. Repeat for the next behavior.

Rules:
- Tests verify behavior through public interfaces only — not implementation details
- A good test survives an internal refactor; if renaming a private function breaks it, the test is wrong
- Use the project's domain vocabulary (`CONTEXT.md`) in test names
- Never write more than one failing test at a time
- Never write production code without a failing test first
- Never refactor while red

### 3e. Verification gate
```sh
task check
```
If red: read the first error, fix only that, re-run. If the same error appears 4+ times unchanged: append `## Blocked` to the spec file with the exact error output. Update PLAN.md to `status: blocked`. Stop this issue.

### 3f. Commit and open PR

Rebase before opening:
```sh
git fetch origin && git rebase origin/main
task check   # must be green after rebase
```

Stage files **by name** — never `git add .` or `git add -A`:
```sh
git add <specific files>
git commit -m "chore(NNN): concise description"
```

Open the PR:
```sh
gh pr create \
  --title "[NNN-slug]: Issue title" \
  --body "$(cat <<'EOF'
## Issue

[ralph/PLAN.md — issue NNN](<path to spec file>)

## Completion promise

> (copy the Completion promise from the spec verbatim)

## Acceptance criteria

(copy the checkboxes from the spec verbatim)
EOF
)"
```

### 3g. Update PLAN.md
```
status: pr-open
pr: <PR number>
```

## 4. Fix-review workflow (for `fixing-review` issues)

The shell has deposited CodeRabbit's agent prompt at `ralph/reviews/NNN-cr-prompt.txt`.

### 4a. Study the review
Read `ralph/reviews/NNN-cr-prompt.txt`. Each comment has: file, line range, specific instruction.

Read the current code at every location mentioned. Verify each comment is still valid — the code may already address it.

### 4b. Apply fixes
For each still-valid comment:
- Apply the minimal fix. Don't clean up surrounding code.
- Run `task check` after each fix.

### 4c. Verification gate
`task check` must be green before pushing.

### 4d. Commit and push
```sh
git add <specific files>
git commit -m "fix(NNN): address CodeRabbit review comments"
git push
```

### 4e. Update PLAN.md
```
status: pr-open
```
Delete `ralph/reviews/NNN-cr-prompt.txt` after pushing.

## 5. Invariants — higher priority than everything else

- Never `--no-verify`. Never force-push to main. Never skip CI.
- Never amend a commit already on a remote branch.
- Never commit secrets, credentials, or build artifacts.
- Never `git add .` or `git add -A` — stage named files only.
- `task check` must be green after rebasing onto latest main before any PR.
- Follow CLAUDE.md coding rules for every line written — they are not optional.

## 6. Completion signals

```
<promise>RALPH DONE</promise>      # all issues done or blocked
<promise>RALPH WAITING</promise>   # nothing to act on this iteration
```
