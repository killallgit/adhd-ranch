# ADR-0003: Use a storage transaction seam for proposal lifecycle writes

**Status:** Accepted
**Date:** 2026-05-12
**Closes:** issue 043 (`proposal-lifecycle-atomicity-design`)

## Context

`ProposalLifecycle::accept` currently performs three sequential writes:

1. Apply the Proposal effect (`append_task`, `create_focus`, or no-op discard)
2. Append a Decision to `decisions.jsonl`
3. Remove the Proposal from `proposals.jsonl`

The current interface returns `Result<DecisionOutcome>`, but the underlying writes are not atomic. If the process dies or a write fails after the Focus/Task mutation, the app can observe a partially accepted Proposal. Retrying can also duplicate the Decision append before the Proposal removal succeeds.

Issue 043 considered two shapes:

- `LifecycleCommit`: one value type for Proposal acceptance only
- `FocusStore::transaction(|tx| ...)`: a general transaction seam for storage workflows

The product is expected to add more multi-write workflows: `merge_focus`, `complete_task`, future Proposal kinds, and possibly settings or migration flows. This makes a one-off Proposal commit too narrow.

## Decision

Introduce a general storage transaction seam.

The exact Rust names may shift during implementation, but the intended interface is:

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

`ProposalLifecycle::accept` will build the edited Proposal and Decision, then run all storage effects inside one transaction. `ProposalLifecycle` owns Proposal rules; storage owns commit ordering, rollback, and recovery.

The storage implementation should start pragmatic: file-based transaction staging under the app data root, using the existing atomic tmpfile + rename primitives where possible. It does not need a database. The transaction must be tested with injected failures between Focus/Task mutation, Decision append, and Proposal removal.

## Why not `LifecycleCommit`

`LifecycleCommit` is attractive for issue 043 alone: it is smaller and maps directly to Proposal acceptance.

We are not choosing it because the seam would be too specific. As soon as `merge_focus` or `complete_task` arrives, either `LifecycleCommit` becomes a generic transaction language under another name, or a second transaction mechanism appears beside it. That would lower locality: future consistency bugs would require checking which workflow uses which commit shape.

## Failure-mode coverage

The transaction implementation must define recovery behavior for:

- Failure before commit starts: no visible mutation.
- Failure after staging but before final commit marker: old state remains authoritative or recovery rolls staged files forward deterministically.
- Failure after the Focus/Task write but before Decision append: no accepted Proposal should be visible without its Decision.
- Failure after Decision append but before Proposal removal: retry must not append a duplicate Decision.

The follow-up implementation issue should choose the concrete marker/staging protocol. The invariant is more important than the exact file layout: after recovery, Proposal acceptance is visible as all three effects or none.

## Test surface

The implementation needs a failure-injection adapter or hook at each write step:

- after Focus/Task mutation staging
- after Decision append staging
- after Proposal removal staging
- before and after final commit marker

Tests should run against temp dirs and assert on actual files, not only in-memory mocks.

## Cost

This is heavier than a Proposal-only commit. It introduces a new storage module, shared error type, and transaction staging tests.

That cost is justified because the app is expected to gain more multi-write workflows. The interface buys leverage by concentrating atomicity, recovery, and write-ordering knowledge in one module instead of every workflow.

## Consequences

- Issue 043 is closed as a design decision.
- A follow-up AFK issue implements the storage transaction seam and migrates `ProposalLifecycle::accept`.
- Future architecture reviews should not re-suggest `LifecycleCommit` unless the planned multi-write workflows are cancelled.
- `FocusStore`, `ProposalQueue`, and `DecisionLog` remain useful read/write adapters, but atomic cross-file workflows should go through the transaction seam.
