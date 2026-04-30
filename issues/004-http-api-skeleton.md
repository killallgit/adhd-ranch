# 004 — HTTP API skeleton: `/health`, `GET /focuses`, port file

## Parent PRD

`PRD.md`

## What to build

Stand up the localhost HTTP server inside the Tauri app. Bind to `127.0.0.1` on an ephemeral port, write that port to `~/.adhd-ranch/run/port` on bind. Implement two read endpoints from FR4: `GET /health` (liveness) and `GET /focuses` (catalog `[{id, title, description}]`).

HTTP transport layer is a thin adapter over the storage trait from slice 003 — no business logic in handlers. Use `axum` or `actix-web` (pick one, justify in CLAUDE.md if non-obvious).

See `PRD.md` FR4.

## Acceptance criteria

- [ ] App launch binds an ephemeral port; `~/.adhd-ranch/run/port` exists with the port number.
- [ ] On clean shutdown, port file is removed (best effort).
- [ ] `curl http://127.0.0.1:$(cat ~/.adhd-ranch/run/port)/health` returns 200.
- [ ] `curl .../focuses` returns the catalog as JSON, matching the on-disk Focuses.
- [ ] Server bound to `127.0.0.1` only — not reachable from `0.0.0.0`.
- [ ] Handler functions take the storage trait as a dependency (constructor DI), not a global.
- [ ] Integration test boots the server with a temp dir and asserts both endpoints.
- [ ] `task check` green.

## Blocked by

- Blocked by `issues/003-markdown-storage-and-watcher.md`

## User stories addressed

Enables US2/US3 by exposing the catalog the in-session agent needs.
