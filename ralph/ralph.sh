#!/usr/bin/env bash
# ralph/ralph.sh — outer loop. All agent logic in ralph/PROMPT.md.
# Run from project root: task ralph  |  task ralph:dry
# Requires: claude CLI, gh CLI, python3

set -euo pipefail

RALPH_PKG="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(pwd)"
PLAN="${PROJECT_ROOT}/ralph/PLAN.md"
PROMPT="${RALPH_PKG}/PROMPT.md"
REVIEWS_DIR="${PROJECT_ROOT}/ralph/reviews"
LOGS_DIR="${PROJECT_ROOT}/ralph/logs"
MAX_ITERATIONS=20
DRY_RUN=false

[[ "${1:-}" == "--dry-run" ]] && DRY_RUN=true

# ── Logging ───────────────────────────────────────────────────────────────────

mkdir -p "$LOGS_DIR"
LOG_FILE="${LOGS_DIR}/$(date +%Y%m%d-%H%M%S).log"
ln -sf "$LOG_FILE" "${LOGS_DIR}/latest.log"

# All output goes to stdout AND log file
exec > >(tee -a "$LOG_FILE") 2>&1

log() { echo "[ralph] $*"; }

log "Log: $LOG_FILE  (tail -f ralph/logs/latest.log)"

# ── Plan helpers (all via python3 — awk range patterns unreliable on macOS) ──

plan_get() {
  python3 - "$PLAN" "$1" "$2" <<'EOF'
import sys, re
path, num, field = sys.argv[1:]
text = open(path).read()
m = re.search(rf'^## {re.escape(num)}\n(.*?)(?=^## |\Z)', text, re.MULTILINE | re.DOTALL)
if not m:
    sys.exit(0)
for line in m.group(1).splitlines():
    if line.startswith(f'{field}:'):
        print(line.split(':', 1)[1].strip())
        break
EOF
}

plan_set() {
  python3 - "$PLAN" "$1" "$2" "$3" <<'EOF'
import sys, re
path, num, field, value = sys.argv[1:]
text = open(path).read()
new = re.sub(
    rf'(^## {re.escape(num)}\n(?:.*\n)*?){re.escape(field)}: .*',
    rf'\g<1>{field}: {value}',
    text, flags=re.MULTILINE
)
if new == text:
    new = re.sub(rf'(^## {re.escape(num)}\n)', rf'\g<1>{field}: {value}\n', text, flags=re.MULTILINE)
open(path, 'w').write(new)
EOF
}

issues_with_status() {
  python3 - "$PLAN" "$1" <<'EOF'
import sys, re
path, status = sys.argv[1:]
text = open(path).read()
for m in re.finditer(r'^## (\d+)\n(.*?)(?=^## |\Z)', text, re.MULTILINE | re.DOTALL):
    num, body = m.group(1), m.group(2)
    for line in body.splitlines():
        if line.startswith('status:') and line.split(':', 1)[1].strip() == status:
            print(num)
            break
EOF
}

# ── CodeRabbit helpers ────────────────────────────────────────────────────────

coderabbit_reviewed() {
  gh pr view "$1" --json reviews \
    | python3 -c "
import json,sys
rs=json.load(sys.stdin)['reviews']
print(sum(1 for r in rs if r['author']['login']=='coderabbitai'))"
}

unresolved_cr_threads() {
  local owner repo
  owner=$(gh repo view --json owner -q .owner.login)
  repo=$(gh repo view --json name -q .name)
  gh api graphql -f query="{repository(owner:\"$owner\",name:\"$repo\"){pullRequest(number:$1){reviewThreads(first:100){nodes{isResolved isOutdated comments(first:1){nodes{author{login}}}}}}}}" \
    | python3 -c "
import json,sys
ts=json.load(sys.stdin)['data']['repository']['pullRequest']['reviewThreads']['nodes']
print(sum(1 for t in ts if not t['isResolved'] and not t['isOutdated'] and any(c['author']['login']=='coderabbitai' for c in t['comments']['nodes'])))"
}

extract_cr_prompt() {
  local issue_num=$1 pr=$2
  mkdir -p "$REVIEWS_DIR"
  local prompt
  prompt=$(gh pr view "$pr" --json reviews | python3 -c "
import json,sys,re
rs=json.load(sys.stdin)['reviews']
cr=[r for r in rs if r['author']['login']=='coderabbitai']
if not cr: sys.exit(0)
body=cr[-1]['body']
m=re.search(r'<summary>🤖 Prompt for all review comments.*?</summary>\s*\n+(.*?)\n+</details>',body,re.DOTALL)
if not m: sys.exit(0)
b=m.group(1).strip()
b=re.sub(r'^\`\`\`\w*\n','',b); b=re.sub(r'\n\`\`\`$','',b)
print(b.strip())" 2>/dev/null || true)
  [[ -n "$prompt" ]] && echo "$prompt" > "$REVIEWS_DIR/${issue_num}-cr-prompt.txt"
  [[ -n "$prompt" ]]
}

# ── Per-iteration handlers ────────────────────────────────────────────────────

handle_pr_open() {
  local num=$1 pr
  pr=$(plan_get "$num" pr)
  [[ -z "$pr" || "$pr" == "~" ]] && { log "Issue $num: no PR number — skipping"; return; }

  local reviewed
  reviewed=$(coderabbit_reviewed "$pr")
  if [[ "$reviewed" -eq 0 ]]; then
    log "Issue $num: PR #$pr — waiting for CodeRabbit"
    return
  fi

  local unresolved
  unresolved=$(unresolved_cr_threads "$pr")
  if [[ "$unresolved" -eq 0 ]]; then
    log "Issue $num: all CR threads resolved → ready-to-merge"
    plan_set "$num" status ready-to-merge
  elif extract_cr_prompt "$num" "$pr"; then
    log "Issue $num: $unresolved unresolved threads → fixing-review"
    plan_set "$num" status fixing-review
  else
    log "Issue $num: no actionable CR prompt → ready-to-merge"
    plan_set "$num" status ready-to-merge
  fi
}

handle_ready_to_merge() {
  local num=$1 pr
  pr=$(plan_get "$num" pr)
  log "Issue $num: merging PR #$pr"
  if [[ "$DRY_RUN" == true ]]; then
    log "  [dry-run] gh pr merge $pr --squash --delete-branch"
    plan_set "$num" status done
    return
  fi
  gh pr merge "$pr" --squash --delete-branch && plan_set "$num" status done || log "Issue $num: merge failed"
}

# ── Main ──────────────────────────────────────────────────────────────────────

[[ -f "$PLAN" ]]   || { log "ralph/PLAN.md not found"; exit 1; }
[[ -f "$PROMPT" ]] || { log "ralph/PROMPT.md not found"; exit 1; }

log "Starting. Max iterations: $MAX_ITERATIONS."

iter=0
while [[ $iter -lt $MAX_ITERATIONS ]]; do
  iter=$((iter + 1))
  log ""
  log "═══ Iteration $iter / $MAX_ITERATIONS ═══"

  while IFS= read -r num; do [[ -n "$num" ]] && handle_ready_to_merge "$num"; done \
    < <(issues_with_status ready-to-merge)

  while IFS= read -r num; do [[ -n "$num" ]] && handle_pr_open "$num"; done \
    < <(issues_with_status pr-open)

  remaining=$(issues_with_status queued && issues_with_status implementing \
           && issues_with_status pr-open && issues_with_status fixing-review \
           && issues_with_status ready-to-merge || true)

  [[ -z "$remaining" ]] && { log "All done."; exit 0; }

  agent_work=$(issues_with_status queued && issues_with_status fixing-review || true)

  if [[ -z "$agent_work" ]]; then
    log "Waiting on PRs — sleeping 30s"
    sleep 30
    continue
  fi

  log "Running agent for: $(echo "$agent_work" | tr '\n' ' ')"

  if [[ "$DRY_RUN" == true ]]; then
    log "[dry-run] would run: claude -p --dangerously-skip-permissions < $PROMPT"
    continue
  fi

  ITER_LOG="${LOGS_DIR}/iter-${iter}.log"
  log "Agent output → tail -f ralph/logs/iter-${iter}.log"
  claude -p --dangerously-skip-permissions < "$PROMPT" 2>&1 | tee "$ITER_LOG" || true

  grep -qF "<promise>RALPH DONE</promise>" "$ITER_LOG" && { log "Done."; exit 0; }

done

log "Max iterations ($MAX_ITERATIONS) reached."
exit 1
