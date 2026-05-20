# adhd-ranch

Issue tracking for a five-year-old. macOS menubar app — Tauri v2 + React.

A small number of buckets ("Focuses") with a few bullets each ("Tasks"). Pixel pigs roam the screen as a peripheral reminder — one pig per Focus. Click a pig to inspect or edit its Tasks. Markdown on disk is the source of truth.

See `PRD.md`, `CONTEXT.md`, and `CLAUDE.md` for the full design and the programming rules every slice must follow.

## Quick install (end-user)

1. Download the latest package for your platform from [Releases](../../releases):
   - macOS: `.dmg`
   - Windows: `_x64-setup.exe` or `_arm64-setup.exe`
   - Linux: `.AppImage` or `.deb`
2. The v1 builds are **not codesigned**. macOS Gatekeeper will block the first launch. Either right-click the app → Open (Open button appears), or:
   ```sh
   xattr -dr com.apple.quarantine "/Applications/Adhd Ranch.app"
   ```
3. Launch the app once. The tray icon appears in the menubar.
4. The `/checkpoint` slash command exists in the repo for the deferred v1.3 agent proposal flow, but v1.2 does not require installing it.

## Day-to-day usage

1. Open the tray menu. Click **+ New Focus** and give it a title + short description. The description is retained for the deferred v1.3 routing agent.
2. A pig appears for each Focus and wanders on the overlay.
3. Click a pig to open its detail card. Add, edit, complete, or clear Tasks from there.
4. Click the clock/time control on the Focus title or on any Task to start, restart, or clear a timer. Focus timers also drive animal growth and expired-focus alerts.
5. Hand-edit `~/.adhd-ranch/focuses/<slug>/focus.md` whenever you want — the watcher reflects changes within a second. Adding `- [ ] something` adds a task, deleting a line removes it.

## Limits + alerts

Default caps: **5 Focuses**, **7 Tasks per Focus**. Going over still works (your markdown wins) but the widget shows a red badge and macOS pops a one-shot notification per `under → over` transition.

Override defaults in `~/.adhd-ranch/settings.yaml`:

```yaml
caps:
  max_focuses: 5
  max_tasks_per_focus: 7
notifications:
  timer_expired: true
  focuses_over_cap: true
  tasks_over_cap: true
widget:
  always_on_top: true
  confirm_delete: true
displays:
  enabled: 0
```

Missing keys fall back to defaults. Settings changed through the app are persisted immediately; manual file edits are picked up on app restart.

## Storage layout (canonical state)

```
~/.adhd-ranch/
  focuses/
    <slug>/focus.md     YAML frontmatter + - [ ] bullets
    <slug>/timer.json    optional focus countdown timer sidecar
    <slug>/task-timers.json
                         optional task countdown timer sidecar, indexed to task order
  proposals.jsonl       pending proposals, one per line
  decisions.jsonl       audit log of accept/reject (with edited flag)
  settings.yaml         optional caps + notification/widget/display config
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

Releases are proposed by `.github/workflows/release-please.yml` and packaged by `.github/workflows/release.yml`.

- Normal release: merge the Release Please PR. It bumps versions, updates `CHANGELOG.md`, creates a `v*` GitHub Release, and publishing that release triggers the cross-platform build.
- Release Please needs a `RELEASE_PLEASE_TOKEN` repository secret backed by a PAT or GitHub App token with contents and pull-request write access. The default `GITHUB_TOKEN` is intentionally not used because releases it creates do not trigger the packaging workflow.
- Manual repair path: GitHub Actions → **release** → Run workflow → enter a SemVer version without `v`, for example `0.1.0`. Leave `draft` unchecked to publish it immediately, or check it for a private review pass.
- Packages built: macOS universal `.dmg`, Windows x64 NSIS installer, Windows ARM64 NSIS installer, Linux x64 `.AppImage`, and Linux x64 `.deb`.
- `SHA256SUMS.txt` is attached to each release for package integrity checks, and GitHub provenance attestations are generated for the release packages.

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
skill/               /checkpoint slash command for deferred v1.3 proposal flow
.github/workflows/   CI
```

Pick the lowest unblocked file in `issues/`, follow `issues/README.md`, ship one slice per PR.
