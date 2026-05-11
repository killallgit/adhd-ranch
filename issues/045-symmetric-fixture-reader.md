# 045 — Symmetric `fixtureReader<T>` + caps fixture parity

## Parent PRD

PRD.md §FR3 (pig UI) — test infrastructure

## What to build

After 040 lands, the Tauri side has `createTauriReader<T>(invokeKey, eventKey?)` consolidating focus / proposal / caps Tauri readers. The fixture side is **inconsistent**:

- `src/api/fixtureFocusReader.ts` — dedicated file, exports factory
- `src/api/fixtureProposalReader.ts` — dedicated file, exports factory
- `src/api/caps.ts:23` — exports `createFixtureCapsReader(caps)` co-located with the production reader (no dedicated file)
- `src/hooks/useAppState.test.tsx:14` — **re-implements** the caps fixture body locally (`{ get: () => Promise.resolve(caps) }`) instead of importing `createFixtureCapsReader`

No generic helper exists; each fixture is bespoke. Close the gap with `createFixtureReader<T>` that all three concrete fixtures delegate to, and migrate the test off its local reimplementation.

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
- `createFixtureCapsReader` (already exists in `src/api/caps.ts:23`) → reimplement to wrap `createFixtureReader<Caps>({ initial: seed })`
- `src/hooks/useAppState.test.tsx:14`: delete local `fixtureCapsReader` wrapper; import `createFixtureCapsReader` directly

### Optional: configurable failure injection

If a test wants to assert on read failure, allow `{ initial: T } | { error: Error }`. Defer if not needed by current tests.

## Completion promise

Every `PolledReader<T>` consumer has a fixture factory built on one shared `createFixtureReader<T>` helper; no test re-implements a fixture body locally.

## Acceptance criteria

- [ ] `src/api/fixtureReader.ts` exports `createFixtureReader<T>`
- [ ] `createFixtureFocusReader`, `createFixtureProposalReader`, `createFixtureCapsReader` all delegate to `createFixtureReader<T>`
- [ ] `useAppState.test.tsx` imports `createFixtureCapsReader` from `src/api/caps.ts`; the local `fixtureCapsReader` wrapper at line 14 is deleted
- [ ] All existing Vitest specs pass without behaviour change
- [ ] `task check` green

## Blocked by

040 (and transitively 037, 038, 039 — generic Tauri reader factory).

## User stories addressed

- "Adding a new polled reader requires zero test plumbing — the fixture factory drops in."
