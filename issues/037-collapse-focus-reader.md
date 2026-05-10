# 037 — Collapse `FocusReader` into `PolledReader<Focus[]>`

## What to build

`FocusReader` (`src/api/focuses.ts`) and `PolledReader<T>` (`src/hooks/usePolledReader.ts`) are the same shape: `list/read()` + optional `subscribe()`. `useFocuses` (`src/hooks/useFocuses.ts`) is 22 lines that wrap a `FocusReader` into a `PolledReader<readonly Focus[]>` and rename `value` → `focuses`. Two interfaces, one rename hop, no added value.

Collapse the duplicate seam: `tauriFocusReader` and `fixtureFocusReader` produce a `PolledReader<readonly Focus[]>` directly. `useFocuses` is deleted; callers use `usePolledReader<readonly Focus[]>(reader)` and destructure `value` (rename at the call site).

### Changes

- `src/api/focuses.ts` — delete `FocusReader` and `Unsubscribe` (the latter is already exported from `usePolledReader`). Re-export `PolledReader<readonly Focus[]>` as the file's typedef if a name is helpful (`type FocusReader = PolledReader<readonly Focus[]>`), or remove the file.
- `src/api/tauriFocusReader.ts` — return `PolledReader<readonly Focus[]>`. Rename `list` → `read`. Keep the `subscribe` channel binding.
- `src/api/fixtureFocusReader.ts` — return `PolledReader<readonly Focus[]>`. Rename `list` → `read`.
- `src/hooks/useFocuses.ts` — delete the file.
- `src/hooks/useFocuses.test.tsx` — delete or fold into `usePolledReader.test.tsx` (the polling/subscribe contract is tested there already).
- `src/components/App.tsx` (line 18) — call `usePolledReader(focusReader)`; destructure `value` as `focuses` at the call site.
- `src/hooks/useAppState.ts` — same call-site rename. `FocusesState` becomes `PolledState<readonly Focus[]>`.
- `src/hooks/useAppState.test.tsx` — update fixture imports if signatures changed.
- `src/main.tsx` — no logic change; reader factory call still works.

### Out of scope

- Proposals and caps pipelines (separate slices: 038, 039).
- Generic `tauriReader<T>(invokeKey, eventKey)` factory (slice 040).

## Completion promise

The Focus catalog flows through one reader interface (`PolledReader<T>`) and one polling hook (`usePolledReader`). `FocusReader`, `useFocuses`, and `useFocuses.test.tsx` no longer exist; all Focus catalog reads go directly through `usePolledReader<readonly Focus[]>`.

## Acceptance criteria

- [ ] `src/api/focuses.ts` defines no second-class `FocusReader` interface (either deleted or aliased to `PolledReader<readonly Focus[]>`).
- [ ] `src/hooks/useFocuses.ts` deleted.
- [ ] `src/hooks/useFocuses.test.tsx` deleted or folded into `usePolledReader.test.tsx`.
- [ ] `tauriFocusReader` and `fixtureFocusReader` return `PolledReader<readonly Focus[]>`.
- [ ] `App.tsx` and `useAppState.ts` use `usePolledReader` directly with a call-site rename of `value` → `focuses`.
- [ ] All other tests in `src/` still green; no behavior change visible to the user.
- [ ] `task check` green.

## Blocked by

None — can start immediately.
