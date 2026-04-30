# adhd-ranch

Issue tracking for a five-year-old. macOS menubar app — Tauri v2 + React.

See `PRD.md`, `CONTEXT.md`, and `CLAUDE.md` for the full design and the rules every slice must follow.

## Prereqs

- Rust (stable) + `cargo`
- Node 20+ and `npm`
- [`go-task`](https://taskfile.dev) (`brew install go-task`)
- macOS (v1 is macOS-only)

## Quick start

```sh
task install   # install frontend deps
task dev       # launch Tauri dev window
task check     # PR gate: lint + typecheck + tests
```

## Layout

```
src/                 frontend (React + TS)
  components/        view-only React components
  hooks/             state + effects
  api/               typed HTTP/IPC clients
  types/             shared TS types
src-tauri/           Tauri v2 host (Rust)
  src/
    api/             HTTP API surface
    ui_bridge/       Tauri command handlers
    app/             composition root
crates/
  domain/            pure domain types and logic — no I/O
  storage/           disk + watcher adapters
.github/workflows/   CI
```

## Workflow

Pick the lowest unblocked file in `issues/`, follow `issues/README.md`, ship one slice per PR.
