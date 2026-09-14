# adhd-ranch

Issue tracking for a five-year-old. Menubar/tray desktop app — Tauri v2 + React. macOS is the primary target.

A small number of buckets ("Focuses") with a few bullets each ("Tasks"). Pixel pigs roam the screen as a peripheral reminder — one pig per Focus. Click a pig to inspect or edit its Tasks. Markdown on disk is the source of truth.

See `PRD.md`, `CONTEXT.md`, and `CLAUDE.md` for the full design and the programming rules every slice must follow.

## Quick install (end-user)

1. Download the latest package for your platform from [Releases](../../releases):
   - macOS: `.dmg`
   - Windows: `_x64-setup.exe` or `_arm64-setup.exe`
   - Linux: `.AppImage` or `.deb`
2. The builds are **not codesigned**. macOS Gatekeeper will block the first launch. Either right-click the app → Open (Open button appears), or:
   ```sh
   xattr -dr com.apple.quarantine "/Applications/Adhd Ranch.app"
   ```
3. Launch the app once. The tray icon appears in the menubar, and the first launch creates one example Focus so the ranch has a pig to show.

## Day-to-day usage

1. Open the tray menu. Click **+ New Focus** and give it a title, an optional description, and an optional timer.
2. A pig appears for each Focus and wanders on the overlay.
3. Click a pig to open its detail card. Rename, duplicate, or delete the Focus; add, edit, check off, or clear Tasks.
4. Click the clock/time control on the Focus title or on any Task to start, restart, or clear a timer. Focus timers also drive animal growth and expired-focus alerts.
5. Drag a pig to move it; release to toss it. Tray → **Gather Pigs** pulls every pig back onto the primary display.
6. Hand-edit `~/.adhd-ranch/focuses/<slug>/focus.md` whenever you want — the watcher reflects changes within a second. Adding `- [ ] something` adds a task, `- [x]` marks it done, deleting a line removes it.
7. Tray → **Settings…** opens Preferences: caps, always-on-top, delete confirmation, enabled displays, and notification toggles.

## Limits + alerts

Default caps: **5 Focuses**, **7 Tasks per Focus**. Going over still works (your markdown wins), but the tray icon turns red and the app sends a one-shot system notification per `under → over` transition.

Override defaults in `~/.adhd-ranch/settings.yaml`:

```yaml
caps:
  max_focuses: 5
  max_tasks_per_focus: 7
notifications:
  timer_expired: true
  task_timer_expired: true
  focuses_over_cap: true
  tasks_over_cap: true
widget:
  always_on_top: true
  confirm_delete: true
displays:
  enabled: 0
```

`displays.enabled` is a comma-separated list of monitor indices. Missing keys fall back to defaults. Settings changed through Preferences are persisted and applied immediately; manual edits to `settings.yaml` are picked up on app restart.

## Storage layout (canonical state)

```
~/.adhd-ranch/          (%APPDATA%\adhd-ranch on Windows)
  focuses/
    <slug>/focus.md     YAML frontmatter + - [ ] / - [x] bullets
    <slug>/timer.json   optional Focus countdown timer sidecar
    <slug>/task-timers.json
                        optional Task countdown timer sidecar, indexed to task order
  settings.yaml         optional caps + notification/widget/display config
```

The app makes no network calls and exposes no network API. The UI talks to Rust through Tauri IPC.

## Development

```sh
task install   # install frontend deps
task dev       # launch Tauri dev window
task check     # PR gate: lint + unused-code checks + typecheck + tests + generated-type drift check
task build     # release bundle in src-tauri/target/release/bundle/
```

`task check` needs `cargo-shear` for the unused-dependency check: `cargo install --locked cargo-shear`.

Releases are created manually with `.github/workflows/release.yml`.

- In GitHub Actions, open **release**, choose **Run workflow** from `main`, and enter a SemVer version without `v`, for example `0.1.2`.
- The workflow creates a draft GitHub Release, builds and attests every package, adds `SHA256SUMS.txt`, and publishes only after every release job succeeds.
- Packages built: macOS universal `.dmg`, Windows x64 NSIS installer, Windows ARM64 NSIS installer, Linux x64 `.AppImage`, and Linux x64 `.deb`.
- `SHA256SUMS.txt` is attached to each release for package integrity checks, and GitHub provenance attestations are generated for the release packages.
- Release assets are kept for the newest release and one previous release. Older release pages, tags, and notes remain available without uploaded packages.

## Layout

```
src/                 frontend (React + TS)
  components/        view-only React components
  hooks/             state + effects
  api/               typed IPC clients
  lib/               pure UI helpers
  types/             shared TS types (types/generated/ comes from Rust via ts-rs)
src-tauri/           Tauri v2 host (Rust)
  src/
    app/             composition root, tray, menus, settings + timer workflows
    ui_bridge/       Tauri command handlers
    display/         monitor geometry, overlay window, click-through hit-testing
crates/
  domain/            pure types and logic — no I/O
  storage/           markdown focus store, settings writer, atomic writes, file watcher
  commands/          use cases and workflows called by the Tauri host
docs/                ADRs, research notes
scripts/             unused-code checks (IPC boundary, CSS)
issues/              vertical-slice issue files
.github/workflows/   CI + release
```

Pick the lowest unblocked file in `issues/`, follow `issues/README.md`, ship one slice per PR.
