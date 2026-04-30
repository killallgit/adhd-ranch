# adhd-ranch

Issue tracking for a five-year-old. macOS menubar app — Tauri v2 + React.

A small number of buckets ("Focuses") with a few bullets each ("Tasks"). Run `/checkpoint` in any Claude Code session and the in-session agent proposes which bucket the moment you just spent belongs in. You accept, reject, or edit the proposal in the menubar widget. Markdown on disk is the source of truth.

See `PRD.md`, `CONTEXT.md`, and `CLAUDE.md` for the full design and the programming rules every slice must follow.

## Quick install (end-user)

1. Download the latest `.dmg` from [Releases](../../releases) and drag `Adhd Ranch.app` into `/Applications`.
2. The v1 build is **not codesigned** — Gatekeeper will block the first launch. Either right-click the app → Open (Open button appears), or:
   ```sh
   xattr -dr com.apple.quarantine "/Applications/Adhd Ranch.app"
   ```
3. Launch the app once. The tray icon appears in the menubar.
4. Install the `/checkpoint` slash command into your global Claude Code:
   ```sh
   git clone https://github.com/<owner>/adhd-ranch.git
   cd adhd-ranch
   task install-skill
   ```
   This copies `skill/checkpoint.md` to `~/.claude/commands/checkpoint.md`.

## Day-to-day usage

1. Open the menubar popover. Click **+ New Focus** and give it a title + short description (the description is what the routing agent reads).
2. In any Claude Code session, run `/checkpoint`. The agent fetches your catalog, picks a bucket (`add_task` / `new_focus` / `discard`), and posts one proposal.
3. The popover shows a `📥 N pending` tray. Tap to expand; tap **✓** to accept, **✗** to reject, **Edit** to change the target Focus or task text first, or **?** to read the agent's reasoning.
4. Hand-edit `~/.adhd-ranch/focuses/<slug>/focus.md` whenever you want — the watcher reflects changes within a second. Adding `- [ ] something` adds a task, deleting a line removes it.

## Limits + alerts

Default caps: **5 Focuses**, **7 Tasks per Focus**. Going over still works (your markdown wins) but the widget shows a red badge and macOS pops a one-shot notification per `under → over` transition.

Override defaults in `~/.adhd-ranch/settings.yaml`:

```yaml
caps:
  max_focuses: 5
  max_tasks_per_focus: 7
alerts:
  system_notifications: true
```

Missing keys fall back to defaults. Restart the app to pick up changes.

## Storage layout (canonical state)

```
~/.adhd-ranch/
  focuses/
    <slug>/focus.md     YAML frontmatter + - [ ] bullets
  proposals.jsonl       pending proposals, one per line
  decisions.jsonl       audit log of accept/reject (with edited flag)
  settings.yaml         optional caps + alert config
  run/port              ephemeral HTTP port
```

`/health` and `/focuses` and friends are exposed at `127.0.0.1:$(cat ~/.adhd-ranch/run/port)` — no auth, localhost-only.

## Development

```sh
task install   # install frontend deps
task dev       # launch Tauri dev window
task check     # PR gate: lint + typecheck + tests
task build     # release .app + .dmg in src-tauri/target/release/bundle/
```

Tagged release: push a `v*` tag (e.g. `v0.1.0`) → `.github/workflows/release.yml` runs on a macOS runner and attaches the `.dmg` to the GitHub release.

## Layout

```
src/                 frontend (React + TS)
  components/        view-only React components
  hooks/             state + effects
  api/               typed HTTP/IPC clients
  lib/               pure UI helpers
  types/             shared TS types
src-tauri/           Tauri v2 host (Rust)
  src/
    api/             HTTP API surface
    ui_bridge/       Tauri command handlers
    app/             composition root
crates/
  domain/            pure types and logic — no I/O
  storage/           disk + watcher adapters
  http-api/          axum router + serve
skill/               /checkpoint slash command
.github/workflows/   CI
```

Pick the lowest unblocked file in `issues/`, follow `issues/README.md`, ship one slice per PR.
