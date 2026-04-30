# Programming rules — adhd-ranch

These rules govern every later vertical slice. Read them before changing code.

## Functional-first

- Pure functions where possible: same inputs → same output, no hidden state, no side effects.
- Immutable data: frozen structs, `readonly` interfaces, `const`. Don't mutate arguments.
- I/O at the edges. Pure logic in `domain` and `storage` (logic only). Side-effecting code belongs in `api`, `ui_bridge`, `app`, and the React transport layer.
- Composition over inheritance. Compose small functions; don't build subclass trees for behaviour reuse.
- Small focused functions. One job. If the name needs "and", split it.
- Many small files over few large files.

## Dependency injection

- Pass dependencies through constructors / function parameters. No global singletons.
- No module-level mutable state. **No `static mut`. No `lazy_static!`/`OnceCell` for shared mutable state. No module-level `let mut` in TS.** Only allowed module-level values: constants, type aliases, loggers, immutable configuration.
- A `global`-style variable is always a bug here. Flag and fix.

## SOLID + interface boundaries

- Program to interfaces:
  - **Rust**: traits at module boundaries, structs implement traits, callers depend on the trait.
  - **TypeScript**: `interface` / `type` at module boundaries, never depend on concrete classes across modules.
- Single-responsibility per module/file.
- Open/closed: extend behaviour by adding new types, not by editing existing switches.
- Interface segregation: small focused traits/interfaces, not god-traits.

## Data / view separation

- **Rust** — `src-tauri/src/`:
  - `domain/` — pure types + pure logic. No I/O, no Tauri, no async runtime.
  - `storage/` — disk and watcher adapters; depend on `domain` types.
  - `api/` — HTTP API; depends on `storage` + `domain` via traits.
  - `ui_bridge/` — Tauri commands / events; depends on `domain` + `storage` via traits.
  - `app/` — composition root: wires everything in `main.rs`.
- **Frontend** — `src/`:
  - `components/` — React components. View only. **No `fetch`, no direct I/O.**
  - `hooks/` — state + effects; call `api/` clients.
  - `api/` — typed HTTP/IPC clients.
  - `types/` — shared TypeScript types.

## Patterns

Pick the right well-known pattern for the job. Prefer named patterns (Repository, Strategy, Adapter, Observer) over ad-hoc abstractions. Don't invent vocabulary.

## Comments

- Comment **why**, never **what**. Code is self-evident; intent and constraint are not.
- No file-level boilerplate. No docstrings on obvious behaviour.
- Don't add comments to code you didn't write.
- No `TODO` without an issue number or a clear next step. Delete dead code; don't comment it out.

## Tests

- Modern syntax only: `cargo test` with `#[test]`; Vitest `describe` + `it`.
- One concept per test. Descriptive names: `test_expired_token_returns_401`, not `test_auth_error`.
- Don't assert on error message strings. Assert on types, status codes, structured fields.
- Arrange-Act-Assert with clear separation.

## Taskfile

- `task check` is the gate: lint + typecheck + tests. Must be green before opening a PR.
- Always prefer `task <target>` over the underlying tool — owners wire env and ordering into the task.
