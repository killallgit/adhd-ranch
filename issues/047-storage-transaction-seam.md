# 047 — Storage transaction seam for Proposal lifecycle

## Parent PRD

PRD.md §FR5 (HTTP API), §FR8 (Audit log), and §Out of scope / v1.3+ (`/checkpoint` proposal flow) + [ADR-0003](../docs/adr/0003-storage-transaction-seam-for-proposal-lifecycle.md)

## What to build

Implement the general storage transaction seam chosen in ADR-0003 and move `ProposalLifecycle::accept` onto it.

Today `ProposalLifecycle::accept` performs separate writes: apply the Proposal effect, append a Decision, then remove the Proposal. This must become one transaction so a crash or injected failure cannot leave a partially accepted Proposal.

### Target shape

Create a storage transaction module in `crates/storage` with a small interface for the write effects Proposal acceptance needs now:

```rust
trait StorageTransaction {
    fn append_task(&mut self, focus_id: &str, text: &str) -> Result<(), StorageError>;
    fn create_focus(
        &mut self,
        new_focus: &NewFocus,
        id: &str,
        created_at: &str,
        timer: Option<FocusTimer>,
    ) -> Result<String, StorageError>;
    fn append_decision(&mut self, decision: &Decision) -> Result<(), StorageError>;
    fn remove_proposal(&mut self, id: &ProposalId) -> Result<bool, StorageError>;
}

trait TransactionalStore {
    fn transaction<R>(
        &self,
        f: impl FnOnce(&mut dyn StorageTransaction) -> Result<R, StorageError>,
    ) -> Result<R, StorageError>;
}
```

Concrete names may change, but the seam must be general enough for future multi-write workflows such as `merge_focus` and `complete_task`.

### Implementation notes

- Keep the concrete implementation file-based; do not introduce a database.
- Use temp dirs/files and atomic rename primitives.
- Define recovery behavior for an interrupted transaction.
- Add failure injection at each meaningful write step so tests can prove all-or-none behavior.
- `ProposalLifecycle::accept` should construct the edited Proposal and Decision, then call the transaction seam. It should not manually coordinate cross-file ordering anymore.
- `reject` may remain simple unless sharing the transaction makes the implementation clearer; either way, explain the choice in code or tests.

## Completion promise

Accepting a Proposal is atomic across Focus/Task mutation, Decision append, and Proposal removal; after any injected failure or crash-recovery simulation, storage observes all effects or none.

## Acceptance criteria

- [ ] A general transaction seam exists in `crates/storage`
- [ ] The concrete transaction implementation supports Proposal acceptance effects
- [ ] `ProposalLifecycle::accept` uses the transaction seam instead of three independent writes
- [ ] Failure-injection tests cover failure between each Proposal acceptance write
- [ ] Tests assert on real temp-dir files after failure and after success
- [ ] Retrying after an interrupted accept cannot duplicate a Decision
- [ ] ADR-0003 remains linked from this issue
- [ ] `task check` green

## Blocked by

043 — done by ADR-0003

## User stories addressed

- "When I accept a proposal and the process dies mid-commit, the next launch sees a consistent state — no orphaned focus, no double-recorded decision."
