# ADR-0001: `cap_state` is already the single projection; no unifying refactor needed

**Status:** Accepted
**Date:** 2026-05-11
**Closes:** issue 042 (`unify-cap-status-projection`)

## Context

During an architecture deepening pass on 2026-05-10, a friction site was claimed: "cap-state has two read paths — `tray.rs` calls `cap_state(...).any_over()` directly for the badge, while `CapEvaluator` wraps `OverCapMonitor` for notifications. Badge and notifier can disagree." Issue 042 proposed introducing a `CapStatus` projection both consumers would read.

## Decision

**Do not implement 042.** The premise is false.

Verified against the code (2026-05-11):

- `crates/domain/src/caps.rs:5-30` already defines `CapState { focuses_over, focus_count, over_task_focus_ids }` and the pure function `cap_state(focuses, caps) -> CapState`.
- `src-tauri/src/app/tray.rs:34,104` reads via `cap_state(&focuses, settings.caps).any_over()`.
- `crates/commands/src/caps.rs:39-40` reads via `let state = cap_state(...); self.monitor.evaluate(&state);`.

Both consumers derive from the **same** `cap_state(...)` function. Given identical `(focuses, caps)` input, both see identical `CapState`. Divergence between badge and notifier is structurally impossible — the only divergence point is the call-site timing (each consumer reads the focus list separately), not the projection.

Renaming `CapState` → `CapStatus` or `cap_state` → `CapStatus::from_focuses` would be pure cosmetic churn against the CLAUDE.md rules ("Surgical Changes — match existing style"; "No abstractions for single-use code").

## Consequences

- Issue 042 file is deleted.
- `issues/README.md` Architecture queue removes the 042 entry.
- Future architecture audits should not re-suggest unifying these consumers behind a new projection. They are already unified.
- The original review note that produced 042 was wrong about the divergence claim. Future audits must verify against the projection function before asserting "two read paths."

## Real (small) friction left, **not** worth a slice

- `OverCapMonitor` holds `Mutex<State>` for in-memory dedup. It is only ever invoked from one tokio task in `src-tauri/src/app/`, single-threaded — concurrent-call risk is zero today. If a second caller ever appears, revisit.
- Naming: `CapState` (state, not status) is fine. No rename.
