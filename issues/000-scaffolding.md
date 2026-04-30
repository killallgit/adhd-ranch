# 000 — Scaffolding: project structure, Taskfile, tests, CI, programming rules

## Parent PRD

`PRD.md`

## What to build

Foundational scaffolding for the Tauri v2 + React app. No product behavior. Establishes the substrate every later slice builds on.

- Initialize Tauri v2 project, single Rust crate (`src-tauri/`), React + TypeScript frontend (Vite).
- Internal Rust module layout enforces **data/view separation** and many-small-files: e.g. `src-tauri/src/{domain,storage,api,ui_bridge,app}` with each module focused on one job. Pure logic in `domain` and `storage`; I/O at edges.
- Frontend layout mirrors separation: `src/{components,hooks,api,types}` — no fetch calls inside view components.
- `Taskfile.yaml` at repo root with targets: `run`, `dev`, `check`, `test`, `lint`, `fmt`, `build`, `ci`.
  - `check` = lint + typecheck + tests (gate for PRs).
- Test frameworks wired with one trivial passing test each:
  - Rust: `cargo test` (unit tests inline + `#[cfg(test)]` modules).
  - Frontend: Vitest + React Testing Library.
- Lint stack: `clippy` + `rustfmt` for Rust; Biome (or ESLint) + Prettier for TS/React. All hooked into `task lint` and `task fmt`.
- GitHub Actions CI workflow (`.github/workflows/ci.yml`) runs `task ci` on push + PR. macOS runner.
- `CLAUDE.md` at repo root documenting programming rules:
  - Functional-first; pure core, I/O at edges.
  - Constructor DI; no module-level mutable state, no global singletons.
  - SOLID; program to interfaces (Rust traits, TS interfaces).
  - Many small files, small focused modules.
  - Strict data/view separation in both Rust and React.
  - Pick the right pattern for the job; prefer well-known patterns (Repository, Strategy, Adapter, etc.) over ad-hoc abstractions.
  - Comment the WHY only.
- README stub describing how to run dev + check.

## Acceptance criteria

- [ ] `task check` passes locally and in CI on a fresh clone.
- [ ] `task dev` launches the Tauri app with an empty window (no product UI yet).
- [ ] `task test` runs both Rust and frontend tests; both green.
- [ ] `task lint` and `task fmt` cover Rust and TS/React; CI fails on lint errors.
- [ ] Repo contains `CLAUDE.md` with the programming rules above.
- [ ] Module layout demonstrably separates data from view in both Rust and React (folders + at least one stub file per layer).
- [ ] `.github/workflows/ci.yml` runs on macOS and executes `task ci`.

## Blocked by

None - can start immediately.

## User stories addressed

Foundation only. No user story directly delivered.
