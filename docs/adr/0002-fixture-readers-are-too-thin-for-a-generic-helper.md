# ADR-0002: Fixture readers are too thin to warrant a generic helper

**Status:** Accepted
**Date:** 2026-05-11
**Closes:** issue 045 (`symmetric-fixture-reader`)
**Related:** 037, 038, 039, 040

## Context

Issue 045 proposed a generic `createFixtureReader<T>(value)` helper plus delegation from the three concrete fixture factories. Filed alongside 037–040 as the "fixture side" of the reader collapse.

After 037, 038, 039, and 040 landed, each fixture factory is three lines:

```ts
export function createFixtureFocusReader(
  focuses: readonly Focus[],
): PolledReader<readonly Focus[]> {
  return { read: () => Promise.resolve(focuses) };
}
```

`createFixtureProposalReader` and `createFixtureCapsReader` are identical in shape (just `T` differs). None set `subscribe`. None do `Raw → Out` mapping.

## Decision

**Do not implement 045.** The generic helper would deduplicate `{ read: () => Promise.resolve(value) }` — one line — across three call sites. Net leverage: zero. Per CLAUDE.md ("No abstractions for single-use code", "Surgical Changes"). Issue 040's own "Out of scope" note already documented this: _"The fixture readers — already trivial after 037–039 (`{ read: () => Promise.resolve(value) }`); no factory needed."_

The original 045 was also drafted from a partially-wrong premise (see the session that filed it — the caps fixture factory already existed at `src/api/caps.ts:23`, not "missing"). The second concrete piece of 045 — deleting the local `fixtureCapsReader` wrapper in `useAppState.test.tsx:14` and importing `createFixtureCapsReader` directly — was folded into the 039 commit and is already done.

## Consequences

- Issue 045 file deleted.
- `issues/README.md` Architecture queue removes the 045 entry.
- Future audits should not re-suggest a generic fixture helper unless the fixture shape grows non-trivial logic (e.g. configurable failure injection, delayed resolution, multi-step state). The current `Promise.resolve(value)` shape does not justify abstraction.
- `createFixtureReader<T>` is not introduced.

## When this should be reopened

If any of the following happen, reconsider:

- Two or more fixture factories grow logic beyond `Promise.resolve(value)` (failure injection, delay, mutable state).
- A test or storybook needs a configurable fixture and starts hand-rolling that logic across files.
