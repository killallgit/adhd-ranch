# 002 — Build pipeline: signed `.app` + dmg distributable

## Parent PRD

`PRD.md`

## What to build

Finish the build/release pipeline early so every later slice ships through a real artifact. Produces a distributable `.app` bundle and `.dmg` from the slice-001 skeleton — even though the app barely does anything yet — to de-risk packaging for the rest of the project.

- `task build` produces a release `.app` under `target/`.
- DMG packaging via `tauri build` (or `create-dmg`) producing `Adhd Ranch.dmg`.
- App icon set (`.icns`) wired into `tauri.conf.json`.
- GitHub Actions release workflow on tag push (`v*`) builds + uploads the dmg as a release asset. macOS runner.
- Codesigning: unsigned for v1 with documented `Gatekeeper` bypass instructions in README, OR ad-hoc signed; choose the simplest viable path and document it.
- Footprint sanity check: release `.app` < 20 MB on disk (PRD non-functional req).

## Acceptance criteria

- [ ] `task build` produces a working `.app` that launches and shows the slice-001 popover.
- [ ] `Adhd Ranch.dmg` opens, drag-to-Applications works, app launches from `/Applications`.
- [ ] CI workflow on tag `v*` produces the dmg as a downloadable release asset.
- [ ] Release `.app` size < 20 MB.
- [ ] README documents install + Gatekeeper bypass (if unsigned).

## Blocked by

- Blocked by `issues/001-tray-popover-skeleton.md`

## User stories addressed

Foundation for shipping. No user story directly delivered.
