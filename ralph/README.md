# Ralph

AFK coding agent. Drains a plan of issues: implements with TDD, opens PRs, waits for CodeRabbit review, resolves comments, merges, repeats.

## Adopt in a new project

**1. Copy or submodule this directory:**
```sh
# copy
cp -r path/to/ralph your-project/ralph

# or submodule
git submodule add <repo-url> ralph
```

**2. Wire into your root `Taskfile.yaml`:**
```yaml
includes:
  ralph:
    taskfile: ralph/Taskfile.yaml
    dir: '{{.ROOT_DIR}}'
```

**3. Bootstrap project state:**
```sh
task ralph:init
```
This creates `ralph/PLAN.md` and `ralph/AGENTS.md`. Fill them in.

**4. Verify setup:**
```sh
task ralph:dry
```

**5. Go AFK:**
```sh
task ralph
```

---

## Directory layout

```
ralph/              ← this package (generic, portable)
  Taskfile.yaml     ← task interface
  ralph.sh          ← outer loop mechanics
  PROMPT.md         ← agent instructions
  init.sh           ← bootstrap helper
  README.md
  PLAN.md           ← issue plan + status tracking
  AGENTS.md         ← build commands, project layout
  reviews/          ← gitignored; CodeRabbit prompts deposited here by shell
issues/             ← one spec file per issue (referenced from ralph/PLAN.md)
  NNN-slug.md
```

## ralph/PLAN.md format

```markdown
## 033
title: Short issue title
status: queued
branch: chore/branch-slug
spec: issues/033-slug.md
pr: ~
```

The `spec:` field must point to a file in `issues/`. Ralph reads this file during §3c (Study the spec).

Status flow: `queued → implementing → pr-open → fixing-review → ready-to-merge → done`

If blocked: `blocked` (agent appends `## Blocked` to the spec file with details).

## Task interface

| Command | What it does |
|---------|-------------|
| `task ralph` | Run until all issues done or blocked |
| `task ralph:dry` | Dry-run: show plan state, print what would run |
| `task ralph:status` | Print current `ralph/PLAN.md` |
| `task ralph:init` | Bootstrap `ralph/` for a new project |
| `task ralph:reset -- 033` | Reset issue 033 back to `queued` |

## Configuration

Override via env vars or Taskfile vars:

```sh
RALPH_MODEL=claude-opus-4-7 task ralph          # change model
task ralph RALPH_MODEL=claude-sonnet-4-6         # Taskfile var form
task ralph RALPH_MAX_ITER=20                     # cap iterations
```

## Requirements

- `claude` CLI (Anthropic Claude Code)
- `gh` CLI (GitHub)
- `python3`
- `task` (Taskfile)
- `task check` defined in the project root — this is the verification gate

## How it works

1. **Agent phase** (each loop): `claude -p < ralph/PROMPT.md`
   - Agent reads `ralph/PLAN.md`
   - Spawns parallel subagents for each `queued` or `fixing-review` issue
   - Each subagent: branches → TDD implement → `task check` → open PR → update plan
   - Signals `RALPH WAITING` or `RALPH DONE` on exit

2. **Shell phase** (between agent calls):
   - Polls open PRs for CodeRabbit reviews via GitHub GraphQL
   - Extracts CodeRabbit's "🤖 Prompt for all review comments" into `ralph/reviews/NNN-cr-prompt.txt`
   - Sets issue to `fixing-review` so agent addresses it next iteration
   - Merges PRs when all review threads resolved
   - Re-runs agent if there's work; sleeps if only waiting on PRs

## CodeRabbit integration

Ralph expects CodeRabbit to review all PRs. No human review required. The shell polls `reviewThreads[].isResolved` via GraphQL and extracts the agent-facing prompt block from CodeRabbit's review body. If your project doesn't use CodeRabbit, PRs with no unresolved threads will be merged automatically after the initial review wait period.
