# 041 — `FocusWriter` returns `WriteOutcome`

## Parent PRD

PRD.md §FR3 (pig UI) — write feedback path

## What to build

Today every `FocusWriter` method ends in `.catch(logErr("op_name"))`, so the promise resolves to `void` even on failure. `App.tsx` then wraps each call in `try/catch { /* already logged */ }` — both layers swallow rejections. UI cannot tell a rename succeeded vs silently dropped, and there is no feedback path for the user.

Introduce a discriminated `WriteOutcome` returned from every writer method. Callers branch on the result. Console logging stays as the visible failure surface for now; user-visible notification handling is a future slice.

### `WriteOutcome` shape (`src/api/focusWriter.ts`)

```ts
export type WriteOutcome =
  | { ok: true }
  | { ok: false; kind: "ipc" | "domain" | "not_found"; message: string };
```

- `ipc` — Tauri invoke rejected (transport / serde)
- `domain` — Rust returned `CommandError::BadRequest` (e.g. `EmptyTaskText`)
- `not_found` — focus or task index missing
- `message` — raw stringified error for the console; **not** displayed to user

### `FocusWriter` interface

```ts
export interface FocusWriter {
  createFocus(input: CreateFocusInput): Promise<WriteOutcome>;
  deleteFocus(focusId: string): Promise<WriteOutcome>;
  appendTask(focusId: string, text: string): Promise<WriteOutcome>;
  deleteTask(focusId: string, index: number): Promise<WriteOutcome>;
  renameFocus(focusId: string, title: string): Promise<WriteOutcome>;
  updateTask(focusId: string, index: number, text: string): Promise<WriteOutcome>;
  toggleTask(focusId: string, index: number, done: boolean): Promise<WriteOutcome>;
}
```

- No method ever rejects. All failure modes flow through `WriteOutcome`.
- `tauriFocusWriter` maps Tauri rejection → `{ ok: false, kind, message }`.
- Add `fixtureFocusWriter` for tests with a configurable failure mode.

### `App.tsx` callers

- Replace every `try { await writer.X(...) } catch { /* logged */ }` with:
  ```ts
  const outcome = await focusWriter.X(...);
  if (!outcome.ok) console.warn("X failed", outcome.kind, outcome.message);
  ```
- No new UI surface in this slice.

## Completion promise

Every `FocusWriter` method returns a typed `WriteOutcome`; no rejection escapes the writer; no `App.tsx` callsite uses `try/catch` to discard a writer error.

## Acceptance criteria

- [x] `WriteOutcome` defined in `src/api/focusWriter.ts`
- [x] `tauriFocusWriter` returns `WriteOutcome` for all writer methods
- [x] `fixtureFocusWriter` exists with a configurable failure injector
- [x] `App.tsx` removes all `try/catch` around writer calls; branches on `outcome.ok`
- [x] Vitest covers: `ok: true` path, `ipc` failure, `domain` failure
- [x] No `.catch(logErr(...))` left in writer
- [x] `task check` green

## Blocked by

None.

## Hands off to

Future notification UI: surface `outcome.ok === false` through a user-visible notification instead of console-only reporting.

## User stories addressed

- "When I clear a task and the write fails, the system at minimum records the failure in a typed form so the UI can react in a follow-up slice."
