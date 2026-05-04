#!/usr/bin/env bash
# ralph/init.sh — bootstrap .ralph/ for a new project.
# Run from the project root: task ralph:init

set -euo pipefail

RALPH_DIR="ralph"

if [[ -f "$RALPH_DIR/PLAN.md" ]]; then
  echo "ralph: ralph/PLAN.md already exists — nothing to do."
  echo "       Edit it to add your issues, or run 'task ralph:status' to see the current plan."
  exit 0
fi

mkdir -p "$RALPH_DIR"

cat > "$RALPH_DIR/PLAN.md" <<'EOF'
# Ralph Plan

> Single source of truth for Ralph's progress.
> Update after every state change. Never leave stale.
> Status legend: queued → implementing → pr-open → fixing-review → ready-to-merge → done | blocked

## Example issue
title: Replace this with your first issue title
status: queued
branch: chore/your-branch-slug
spec: issues/your-issue-file.md
pr: ~

EOF

cat > "$RALPH_DIR/AGENTS.md" <<'EOF'
# Operational info — <project name>

## Build and verify

```sh
task check          # lint + typecheck + tests — must be green before any PR
```

## Additional targets (fill in for this project)

```sh
# task test         # tests only
# task lint         # lint only
# task typecheck    # typecheck only
```

## Project layout

- Source: (describe your main source dirs)
- Tests: (describe where tests live)
- Issues: issues/NNN-slug.md

## Git and PR

```sh
git fetch origin
git checkout main && git pull --ff-only
git checkout -b <branch>
git rebase origin/main     # always rebase, never merge main into branch

gh pr create --title "[NNN-slug]: Title" --body "..."
gh pr merge <number> --squash --delete-branch
```

EOF

# Add ralph runtime dirs to .gitignore if not already there
if [[ -f ".gitignore" ]]; then
  if ! grep -q "ralph/reviews" .gitignore 2>/dev/null; then
    printf "\n# ralph runtime state\nralph/reviews/\nralph/state/\n" >> .gitignore
    echo "ralph: added ralph/reviews/ and ralph/state/ to .gitignore"
  fi
fi

# Create issues/ dir if missing
if [[ ! -d "issues" ]]; then
  mkdir -p issues
  echo "ralph: created issues/"
fi

echo "ralph: created ralph/PLAN.md and ralph/AGENTS.md"
echo ""
echo "Next steps:"
echo "  1. Edit ralph/PLAN.md — add your issues"
echo "  2. Edit ralph/AGENTS.md — fill in your project's build commands and layout"
echo "  3. Add issue specs to issues/NNN-slug.md (one file per issue)"
echo "     Reference them in ralph/PLAN.md as: spec: issues/NNN-slug.md"
echo "  4. Add to your root Taskfile.yaml:"
echo ""
echo "     includes:"
echo "       ralph:"
echo "         taskfile: ralph/Taskfile.yaml"
echo "         dir: '{{.ROOT_DIR}}'"
echo ""
echo "  5. Run: task ralph:dry   (verify setup)"
echo "  6. Run: task ralph       (go AFK)"
