# 038 — Collapse `ProposalReader` into `PolledReader<Proposal[]>`

## What to build

Mirror of issue 037 for the Proposal catalog. `ProposalReader` (`src/api/proposals.ts`) is the same shape as `PolledReader<T>`. `useProposals` (`src/hooks/useProposals.ts`) is a 22-line rename hop.

Collapse: `tauriProposalReader` and `fixtureProposalReader` return `PolledReader<readonly Proposal[]>`; `useProposals` is deleted; callers use `usePolledReader<readonly Proposal[]>`.

### Changes

- `src/api/proposals.ts` — delete `ProposalReader` (or alias to `PolledReader<readonly Proposal[]>`). Keep `ProposalDecisionResult`, `ProposalEdit`, `ProposalWriter` — those are write-side, unrelated.
- `src/api/tauriProposalReader.ts` — return `PolledReader<readonly Proposal[]>`. Rename `list` → `read`.
- `src/api/fixtureProposalReader.ts` — same rename.
- `src/hooks/useProposals.ts` — delete.
- `src/hooks/useAppState.ts` (line 29) — call `usePolledReader(proposalReader)`; destructure `value` as `proposals`. `ProposalsState` becomes `PolledState<readonly Proposal[]>`.
- `src/hooks/useAppState.test.tsx` — update if reader signatures changed.

### Out of scope

- Focus and caps pipelines (037, 039).
- Generic factory (040).
- `ProposalWriter` — unchanged.

## Completion promise

The Proposal catalog flows through one reader interface and one polling hook. `ProposalReader` and `useProposals` no longer exist; reads go directly through `usePolledReader<readonly Proposal[]>`.

## Acceptance criteria

- [ ] `ProposalReader` interface deleted (or aliased only).
- [ ] `src/hooks/useProposals.ts` deleted.
- [ ] `tauriProposalReader` and `fixtureProposalReader` return `PolledReader<readonly Proposal[]>`.
- [ ] `useAppState.ts` uses `usePolledReader` directly for proposals.
- [ ] All proposal-related tests still green; no behavior change.
- [ ] `task check` green.

## Blocked by

037 — same pattern, do Focus first to validate the call-site shape.
