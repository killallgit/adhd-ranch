# 039 — Unify `useCaps` on `usePolledReader<Caps>`

## What to build

`useCaps` (`src/hooks/useCaps.ts`) is a one-shot reader pattern: returns `Caps` directly, silently falls back to `DEFAULT_CAPS` on error, no loading state. `useFocuses` and `useProposals` (now collapsed in 037/038) use the polled pattern: `loading | ready | error`. Two patterns for the same job.

Unify on `usePolledReader<Caps>`. Preserve current UX (silent default fallback, no loading flash) by mapping the polled state at the call site.

### Changes

- `src/api/caps.ts` — `CapsReader` becomes `PolledReader<Caps>`. `get()` renames to `read()`. Drop the `subscribe?` field if caps are not pushed by Tauri (currently they are not — caps are read once at startup).
- `src/api/caps.ts` — `createTauriCapsReader` and `createFixtureCapsReader` return `PolledReader<Caps>`.
- `src/hooks/useCaps.ts` — delete.
- `src/hooks/useAppState.ts` (line 30) — replace `useCaps(capsReader)` with:
  ```ts
  const capsState = usePolledReader<Caps>(capsReader);
  const caps = capsState.status === "ready" ? capsState.value : DEFAULT_CAPS;
  ```
- `useAppState` re-exports `caps: Caps` to match the current consumer contract — no downstream component needs to handle a loading/error variant.

### Behavior contract

- Silent fallback to `DEFAULT_CAPS` on error preserved.
- Caps surface to consumers as `Caps`, not as a polled state. (Other catalogs surface the full polled state because they need to render loading/empty UI; caps drive only badge thresholds where defaults are correct.)

### Out of scope

- Pushing live cap-config updates from Tauri (would need `subscribe`); caps still load once at startup.
- Generic factory (040).

## Completion promise

`useCaps` no longer exists. Caps load through `usePolledReader<Caps>`; the silent-default UX is preserved by collapsing the polled state to `Caps` inside `useAppState`.

## Acceptance criteria

- [ ] `CapsReader` is `PolledReader<Caps>` (or aliased to it).
- [ ] `src/hooks/useCaps.ts` deleted.
- [ ] `useAppState` uses `usePolledReader<Caps>` and falls back to `DEFAULT_CAPS` on non-ready states.
- [ ] No component receives a loading or error state for caps; consumer contract unchanged.
- [ ] Existing cap-badge tests still green.
- [ ] `task check` green.

## Blocked by

037 — establish the rename and call-site pattern in the simpler Focus pipeline first.
