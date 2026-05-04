#!/usr/bin/env bash
# ralph/ralph.sh — outer loop for the AFK coding agent.
#
# Mechanics only. All agent logic lives in ralph/PROMPT.md.
# Must be run from the project root (Taskfile handles this).
#
# Usage:
#   task ralph           # normal run
#   task ralph:dry       # dry-run
#   task ralph:run -- --max-iterations N --model <model>
#
# Requires: claude CLI, gh CLI, python3

set -euo pipefail

# ── Config ────────────────────────────────────────────────────────────────────

# RALPH_PKG: path to this package dir (always ralph/ relative to project root)
RALPH_PKG="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(pwd)"

MAX_ITERATIONS=60
MODEL="${RALPH_MODEL:-claude-opus-4-7}"
DRY_RUN=false
PLAN="${PROJECT_ROOT}/ralph/PLAN.md"
PROMPT="${RALPH_PKG}/PROMPT.md"
REVIEWS_DIR="${PROJECT_ROOT}/ralph/reviews"
POLL_INTERVAL=30   # seconds between CodeRabbit polls
CR_TIMEOUT=600     # max seconds to wait for CodeRabbit initial review

# ── Arg parse ─────────────────────────────────────────────────────────────────

while [[ $# -gt 0 ]]; do
  case $1 in
    --max-iterations) MAX_ITERATIONS="$2"; shift 2 ;;
    --model)          MODEL="$2";          shift 2 ;;
    --dry-run)        DRY_RUN=true;        shift  ;;
    *) echo "Unknown flag: $1" >&2; exit 1 ;;
  esac
done

# ── Helpers ───────────────────────────────────────────────────────────────────

log() { echo "[ralph] $*"; }

# Extract field value for a given issue number from PLAN.md
# Usage: plan_get 033 status
plan_get() {
  local num=$1 field=$2
  awk "/^## $num$/,/^## [0-9]|^$/" "$PLAN" | grep "^$field:" | head -1 | sed "s/$field: *//"
}

# Set a field value for a given issue in PLAN.md
# Usage: plan_set 033 status pr-open
plan_set() {
  local num=$1 field=$2 value=$3
  # Use python for reliable multiline sed-equivalent
  python3 - "$PLAN" "$num" "$field" "$value" <<'PYEOF'
import sys, re

path, num, field, value = sys.argv[1:]
text = open(path).read()

# Find the section for this issue and replace the field
pattern = rf'(^## {re.escape(num)}\n(?:.*\n)*?){re.escape(field)}: .*'
replacement = rf'\g<1>{field}: {value}'
new = re.sub(pattern, replacement, text, flags=re.MULTILINE)

if new == text:
    # Field not found — append it to the section
    new = re.sub(
        rf'(^## {re.escape(num)}\n)',
        rf'\g<1>{field}: {value}\n',
        text, flags=re.MULTILINE
    )

open(path, 'w').write(new)
PYEOF
}

# List issue numbers with a given status (returns empty string if none)
issues_with_status() {
  local status=$1
  grep -B20 "^status: $status" "$PLAN" 2>/dev/null | grep "^## [0-9]" | sed 's/^## //' || true
}

# Count unresolved (non-outdated) CodeRabbit threads on a PR
# Returns 0 if all resolved, >0 if some remain
count_unresolved_cr_threads() {
  local pr_number=$1
  local owner repo
  owner=$(gh repo view --json owner -q .owner.login)
  repo=$(gh repo view --json name -q .name)

  gh api graphql -f query="
  {
    repository(owner: \"$owner\", name: \"$repo\") {
      pullRequest(number: $pr_number) {
        reviewThreads(first: 100) {
          nodes {
            isResolved
            isOutdated
            comments(first: 1) {
              nodes { author { login } }
            }
          }
        }
      }
    }
  }" | python3 -c "
import json, sys
data = json.load(sys.stdin)
threads = data['data']['repository']['pullRequest']['reviewThreads']['nodes']
unresolved = sum(
    1 for t in threads
    if not t['isResolved']
    and not t['isOutdated']
    and any(c['author']['login'] == 'coderabbitai' for c in t['comments']['nodes'])
)
print(unresolved)
"
}

# Check if CodeRabbit has posted at least one review on this PR
coderabbit_has_reviewed() {
  local pr_number=$1
  local count
  count=$(gh pr view "$pr_number" --json reviews \
    | python3 -c "
import json, sys
reviews = json.load(sys.stdin)['reviews']
cr = [r for r in reviews if r['author']['login'] == 'coderabbitai']
print(len(cr))
")
  [[ "$count" -gt 0 ]]
}

# Extract CodeRabbit's agent fix prompt from the latest review body
# Writes to .ralph/reviews/NNN-cr-prompt.txt if actionable items found
# Returns 0 if actionable items found, 1 if nothing to fix
extract_cr_prompt() {
  local issue_num=$1 pr_number=$2
  mkdir -p "$REVIEWS_DIR"
  local out="$REVIEWS_DIR/${issue_num}-cr-prompt.txt"

  local prompt
  prompt=$(gh pr view "$pr_number" --json reviews \
    | python3 -c "
import json, sys, re

reviews = json.load(sys.stdin)['reviews']
# Get latest coderabbitai review
cr_reviews = [r for r in reviews if r['author']['login'] == 'coderabbitai']
if not cr_reviews:
    print('')
    sys.exit(0)

latest = cr_reviews[-1]
body = latest['body']

# Extract the agent prompt block between the summary tags
match = re.search(
    r'<summary>🤖 Prompt for all review comments.*?</summary>\s*\n+(.*?)\n+</details>',
    body, re.DOTALL
)
if not match:
    print('')
    sys.exit(0)

block = match.group(1).strip()
# Strip surrounding backtick fences if present
block = re.sub(r'^```\w*\n', '', block)
block = re.sub(r'\n```$', '', block)
print(block.strip())
")

  if [[ -z "$prompt" ]]; then
    return 1
  fi

  echo "$prompt" > "$out"
  return 0
}

# ── PR lifecycle handler ───────────────────────────────────────────────────────
#
# Called for each issue in pr-open state.
# Polls for CodeRabbit review, then transitions state accordingly.

handle_pr_open() {
  local issue_num=$1
  local pr_number
  pr_number=$(plan_get "$issue_num" "pr")

  if [[ -z "$pr_number" || "$pr_number" == "~" ]]; then
    log "Issue $issue_num: status is pr-open but no PR number in PLAN.md — skipping"
    return
  fi

  log "Issue $issue_num: PR #$pr_number — waiting for CodeRabbit review..."

  # Wait up to CR_TIMEOUT seconds for initial CodeRabbit review
  local waited=0
  while ! coderabbit_has_reviewed "$pr_number"; do
    if [[ $waited -ge $CR_TIMEOUT ]]; then
      log "Issue $issue_num: CodeRabbit hasn't reviewed after ${CR_TIMEOUT}s — will check next iteration"
      return
    fi
    sleep "$POLL_INTERVAL"
    waited=$((waited + POLL_INTERVAL))
  done

  log "Issue $issue_num: CodeRabbit has reviewed. Checking threads..."

  local unresolved
  unresolved=$(count_unresolved_cr_threads "$pr_number")

  if [[ "$unresolved" -eq 0 ]]; then
    log "Issue $issue_num: All CR threads resolved — marking ready-to-merge"
    plan_set "$issue_num" "status" "ready-to-merge"
  else
    log "Issue $issue_num: $unresolved unresolved CR threads — extracting fix prompt"
    if extract_cr_prompt "$issue_num" "$pr_number"; then
      plan_set "$issue_num" "status" "fixing-review"
      log "Issue $issue_num: CR prompt written to $REVIEWS_DIR/${issue_num}-cr-prompt.txt"
    else
      log "Issue $issue_num: No actionable prompt found despite unresolved threads — marking ready-to-merge"
      plan_set "$issue_num" "status" "ready-to-merge"
    fi
  fi
}

# ── Merge handler ─────────────────────────────────────────────────────────────

handle_ready_to_merge() {
  local issue_num=$1
  local pr_number
  pr_number=$(plan_get "$issue_num" "pr")

  log "Issue $issue_num: Merging PR #$pr_number..."

  if [[ "$DRY_RUN" == "true" ]]; then
    log "  [dry-run] would: gh pr merge $pr_number --squash --delete-branch"
    plan_set "$issue_num" "status" "done"
    return
  fi

  if gh pr merge "$pr_number" --squash --delete-branch; then
    plan_set "$issue_num" "status" "done"
    log "Issue $issue_num: Merged and done ✓"
  else
    log "Issue $issue_num: Merge failed — leaving as ready-to-merge for next iteration"
  fi
}

# ── Main loop ─────────────────────────────────────────────────────────────────

main() {
  if [[ ! -f "$PLAN" ]]; then
    log "Error: $PLAN not found. Run from project root." >&2
    exit 1
  fi
  if [[ ! -f "$PROMPT" ]]; then
    log "Error: $PROMPT not found." >&2
    exit 1
  fi

  log "Starting. Model: $MODEL. Max iterations: $MAX_ITERATIONS."

  local iter=0

  while [[ $iter -lt $MAX_ITERATIONS ]]; do
    iter=$((iter + 1))
    log ""
    log "═══ Iteration $iter / $MAX_ITERATIONS ═══"

    # ── Step A: Handle shell-side state transitions ──

    # Merge any ready-to-merge PRs
    while IFS= read -r num; do
      [[ -z "$num" ]] && continue
      handle_ready_to_merge "$num"
    done < <(issues_with_status "ready-to-merge")

    # Poll CodeRabbit for any open PRs
    while IFS= read -r num; do
      [[ -z "$num" ]] && continue
      handle_pr_open "$num"
    done < <(issues_with_status "pr-open")

    # ── Step B: Check if all done ──

    local remaining
    remaining=$(issues_with_status "queued"; issues_with_status "implementing"; \
                issues_with_status "pr-open"; issues_with_status "fixing-review"; \
                issues_with_status "ready-to-merge")

    if [[ -z "$remaining" ]]; then
      log "All issues done. RALPH DONE."
      exit 0
    fi

    # ── Step C: Skip agent call if nothing for it to act on ──

    local agent_work
    agent_work=$(issues_with_status "queued"; issues_with_status "fixing-review")

    if [[ -z "$agent_work" ]]; then
      log "No agent work this iteration (waiting on PRs or merges). Sleeping ${POLL_INTERVAL}s..."
      sleep "$POLL_INTERVAL"
      continue
    fi

    # ── Step D: Run agent ──

    log "Agent work available: $agent_work"

    if [[ "$DRY_RUN" == "true" ]]; then
      log "[dry-run] would run: claude -p --model $MODEL < $PROMPT"
    else
      OUTPUT=$(claude -p \
        --dangerously-skip-permissions \
        --model "$MODEL" \
        < "$PROMPT" 2>&1) || true
      echo "$OUTPUT"

      if echo "$OUTPUT" | grep -qF "<promise>RALPH DONE</promise>"; then
        log "Agent signalled RALPH DONE."
        exit 0
      fi
    fi

  done

  log "Max iterations ($MAX_ITERATIONS) reached. Check ralph/PLAN.md for remaining work."
  exit 1
}

main "$@"
