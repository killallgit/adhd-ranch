# 043 — `ProposalLifecycle::accept` atomicity (HITL design)

> **HITL slice — design-only.** No code change merges from this issue. Output is an ADR + a follow-up AFK implementation issue.

## Parent PRD

PRD.md §FR4 (proposal flow, v1.3 deferred)

## What to build

`ProposalLifecycle::accept` (`crates/commands/src/lifecycle.rs:41-52`, helper `apply` at `:64-90`) loads + validates the proposal, then performs three sequential storage writes:

1. `self.apply(&proposal)` — `store.append_task(...)` or `store.create_focus(...)` (writes to a per-focus dir)
2. `self.record_decision(...)` — append to `decisions.jsonl`
3. `self.queue.remove(...)` — rewrite `proposals.jsonl` without the accepted entry

If write 2 fails after write 1 succeeds, the focus exists but is unrecorded. If write 3 fails after write 2, retry duplicates the decision. The transaction boundary is invisible — caller sees `Result<DecisionOutcome>` and assumes atomicity that storage does not provide.

Decide between two shapes for the seam, write an ADR, then file the AFK implementation issue.

### Option A — `LifecycleCommit` value type

```rust
pub struct LifecycleCommit {
    pub effect: ProposalEffect,           // CreateFocus { ... } | AppendTask { ... } | None
    pub decision: Decision,
    pub remove_proposal_id: String,
}

impl FocusStore {
    fn commit_lifecycle(&self, commit: &LifecycleCommit) -> Result<(), StorageError>;
}
```

- Storage applies all three writes in order with rollback markers.
- Caller hands a single value; storage owns atomicity.
- Adding new proposal kinds (`merge_focus`, `complete_task`, v2) = new `ProposalEffect` variant.

### Option B — `FocusStore::transaction(|tx| ...)` seam

```rust
trait FocusStore {
    fn transaction<F, R>(&self, f: F) -> Result<R, StorageError>
    where F: FnOnce(&dyn FocusStoreTx) -> Result<R, StorageError>;
}
```

- Lifecycle chain runs inside the closure; commit on `Ok`, rollback on `Err`.
- More flexible (any future multi-write op uses it).
- Heavier infrastructure (file-based transactions over `flock` + tmpfile-rename).

### Decision criteria to record in the ADR

- Which proposal kinds (v2 `merge_focus`, `complete_task`) the chosen shape extends to without redesign.
- Failure-mode coverage: what happens if process is killed mid-commit.
- Test surface: how do we inject failure between writes 1↔2 and 2↔3.
- Cost: lines of code + invariant complexity.

## Completion promise

An ADR exists at `docs/adr/NNNN-proposal-lifecycle-atomicity.md` recording the chosen shape with rationale; a follow-up AFK implementation issue is filed and linked.

## Acceptance criteria

- [x] ADR drafted at `docs/adr/NNNN-proposal-lifecycle-atomicity.md`
- [x] ADR names the chosen option and explains rejection of the other
- [x] ADR addresses all four decision criteria above
- [x] Follow-up AFK implementation issue filed (numbered after 045)
- [x] Follow-up issue links back to this ADR

## Blocked by

None.

## Blocks

The follow-up AFK implementation issue.

## User stories addressed

- "When I accept a proposal and the process dies mid-commit, the next launch sees a consistent state — no orphaned focus, no double-recorded decision."
