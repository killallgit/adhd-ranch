# 035 — Unit tests for MarkdownFocusStore

## What to build

`crates/storage/src/focus_store.rs` has no `#[test]` block. It is the most load-bearing storage module: atomic directory creation, markdown parse, timer sidecar load, atomic write, delete. Coverage today is only implicit through Commands integration tests (TempDir via `crates/commands`).

Add a `#[cfg(test)]` module directly in `focus_store.rs` (or `focus_store/tests.rs`) covering:

| Test | What it proves |
|------|---------------|
| `create_then_list_roundtrip` | `create_focus` → `list()` returns focus with correct fields |
| `list_with_timer_sidecar` | Focus created with timer → `list()` includes populated `FocusTimer` |
| `list_without_timer_sidecar` | Focus created without timer → `list()` has `timer: None` |
| `delete_removes_directory` | `delete_focus` → focus dir gone from fs |
| `delete_nonexistent_returns_err` | Deleting unknown id returns error, does not panic |
| `corrupted_timer_json_degrades_gracefully` | Malformed `timer.json` → focus still returned, timer field `None` or explicit error variant |
| `append_task_persists` | `append_task` → `list()` shows new task in body |
| `delete_task_persists` | `delete_task(index)` → `list()` task removed |

All tests use `tempfile::TempDir`; no real `~/.adhd-ranch` touched.

## Completion promise

`MarkdownFocusStore` has direct unit tests covering the create/list/delete/task mutation cycle and timer sidecar edge cases; the storage seam is independently trusted without Commands.

## Acceptance criteria

- [ ] At least 8 tests in `focus_store.rs` covering the cases above
- [ ] All tests use `TempDir`; no writes to real home dir
- [ ] Corrupted `timer.json` test passes (no panic, graceful fallback)
- [ ] `cargo test -p adhd-ranch-storage` green in isolation
- [ ] `task check` green

## Blocked by

None — can start immediately.
