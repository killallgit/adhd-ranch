# 040 — Generic `tauriReader<T>(invokeKey, eventKey?)` factory

## What to build

After 037–039, three Tauri reader factories (`createTauriFocusReader`, `createTauriProposalReader`, `createTauriCapsReader`) all do the same thing: `invoke<RustT[]>(channel)` on read, optionally `listen(event, onChange)` for subscribe. Consolidate into one helper.

### Changes

- New `src/api/tauriReader.ts`:
  ```ts
  import { invoke } from "@tauri-apps/api/core";
  import { listen } from "@tauri-apps/api/event";
  import type { PolledReader, Unsubscribe } from "../hooks/usePolledReader";

  export interface TauriReaderConfig<Raw, Out> {
    readonly invokeKey: string;
    readonly eventKey?: string;
    readonly map: (raw: Raw) => Out;
  }

  export function createTauriReader<Raw, Out>(
    cfg: TauriReaderConfig<Raw, Out>,
  ): PolledReader<Out> {
    const { invokeKey, eventKey, map } = cfg;
    return {
      read: () => invoke<Raw>(invokeKey).then(map),
      ...(eventKey
        ? {
            subscribe: async (onChange): Promise<Unsubscribe> => {
              const un = await listen(eventKey, () => onChange());
              return () => un();
            },
          }
        : {}),
    };
  }
  ```
- `src/api/tauriFocusReader.ts` — replace with a one-liner:
  ```ts
  export const createTauriFocusReader = () =>
    createTauriReader<RustFocus[], readonly Focus[]>({
      invokeKey: "list_focuses",
      eventKey: "focuses-changed",
      map: (raw) => raw.map(fromRust),
    });
  ```
- Same shape for `tauriProposalReader.ts` and `tauriCapsReader` (caps has no `eventKey`).
- Keep the per-domain `fromRust` mappers in their current files (one mapper per Rust shape).

### Out of scope

- The fixture readers — already trivial after 037–039 (`{ read: () => Promise.resolve(value) }`); no factory needed.

## Completion promise

One `createTauriReader<Raw, Out>` factory backs all Tauri-side `PolledReader<T>` instances; per-domain files contain only the channel name, optional event name, and `Raw → Out` mapping.

## Acceptance criteria

- [x] `src/api/tauriReader.ts` exposes `createTauriReader<Raw, Out>`.
- [x] `tauriFocusReader`, `tauriProposalReader`, and the Tauri caps reader use it.
- [x] No duplicated `invoke + listen + unsubscribe` plumbing remains in `src/api/`.
- [x] All existing reader tests still green.
- [x] `task check` green.

## Blocked by

037, 038, 039 — collapse per-domain readers first; this slice is the leftover polish.
