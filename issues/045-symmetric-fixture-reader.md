# 045 — Symmetric `fixtureReader<T>` + caps fixture parity

## Parent PRD

PRD.md §FR3 (pig UI) — test infrastructure

## What to build

After 040 lands, the Tauri side has `createTauriReader<T>(invokeKey, eventKey?)` consolidating focus / proposal / caps Tauri readers. The fixture side stays asymmetric:

- `src/api/fixtureFocusReader.ts` — factory exists
- `src/api/fixtureProposalReader.ts` — factory exists
- caps fixture — **hand-rolled inline** in every test (`src/hooks/useAppState.test.tsx`)

Two adapters per concept (Tauri + fixture) is the **real seam** test. Without a fixture caps reader, that test fails for caps. Close the gap with a generic `fixtureReader<T>` mirroring 040's `tauriReader<T>`.

### `fixtureReader<T>` (`src/api/fixtureReader.ts`)

```ts
import type { PolledReader, Unsubscribe } from "../hooks/usePolledReader";

export interface FixtureReaderConfig<T> {
  initial: T;
}

export function createFixtureReader<T>(config: FixtureReaderConfig<T>): PolledReader<T> {
  // read() returns a fresh promise resolving to config.initial
  // subscribe(): no-op returning a noop Unsubscribe
}
```

### Migrations

- `createFixtureFocusReader` → wraps `createFixtureReader<readonly Focus[]>({ initial: seed })`
- `createFixtureProposalReader` → wraps `createFixtureReader<readonly Proposal[]>({ initial: seed })`
- `createFixtureCapsReader` (new) → wraps `createFixtureReader<Caps>({ initial: seed })`
- `src/hooks/useAppState.test.tsx`: replace inline caps fixture with `createFixtureCapsReader(...)`

### Optional: configurable failure injection

If a test wants to assert on read failure, allow `{ initial: T } | { error: Error }`. Defer if not needed by current tests.

## Completion promise

Every `PolledReader<T>` consumer has a fixture factory built on one shared `createFixtureReader<T>` helper; no test hand-rolls a caps fixture.

## Acceptance criteria

- [ ] `src/api/fixtureReader.ts` exports `createFixtureReader<T>`
- [ ] `createFixtureFocusReader`, `createFixtureProposalReader`, `createFixtureCapsReader` all built on it
- [ ] `useAppState.test.tsx` uses `createFixtureCapsReader` (no inline caps fixture object)
- [ ] All existing Vitest specs pass without behaviour change
- [ ] `task check` green

## Blocked by

040 (and transitively 037, 038, 039 — generic Tauri reader factory).

## User stories addressed

- "Adding a new polled reader requires zero test plumbing — the fixture factory drops in."
