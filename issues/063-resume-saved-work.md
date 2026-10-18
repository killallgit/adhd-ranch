# 063 — Resume saved work without startup examples

## Owning specification

`specs/003-ranch-today/spec.md`: FR-001, FR-019, US1/AC4–AC5; `plan.md#startup-correction-slice`; tasks T030–T035. These feature artifacts remain Draft. The owner explicitly selected no seeding and resumption from saved work on 2026-10-18, then requested implementation. This narrow correction follows that direction without promoting other Draft features.

## What to build

Remove the automatic example writer: delete `src-tauri/src/app/seed.rs` and its module declaration/call in `src-tauri/src/app/mod.rs`. Update `docs/architecture.md`. Reuse existing work/Settings loading and Timer expiry; preserve old examples and marker files. No replacement service, flag, onboarding, schema/data migration, click-through fix, or Agent change.

## Completion promise

Ranch starts from the person's saved work and Settings, creates no example Focus or Task, and starts empty when no saved work exists, without resetting or deleting existing work or restarting Timers.

## Acceptance criteria

- [x] Automatic example creation and its marker writer are removed from startup; no replacement work writer is introduced.
- [ ] Saved Focuses/Tasks/Settings and old examples remain intact; legacy markers stay untouched and do not affect startup behavior.
- [ ] Empty roots with/without a legacy marker remain empty across launch/restart (V0).
- [ ] Future/Expired/due-while-closed Focus/Task Timers retain start/duration and follow normal expiry; canonical work is unchanged (V0).
- [x] Architecture documentation matches the correction.
- [x] Fresh `rtk task check` and `rtk task check:windows` pass on synchronized corrected source.
- [ ] Independent review, current-head green CI, and merge are recorded; native acceptance remains open until actual V0 proof exists.

## Constitution compliance

I: remove an unauthorized writer; preserve user-owned data and timer-only expiry. II–IV: retain independent domains, existing typed boundaries, and opt-in integration behavior. V: one narrow correction, current-source gates and actual native evidence before acceptance claims. VI: distinguish owner direction, implementation, historical evidence, and Draft artifacts. No exception or new architecture decision is proposed.

## Blocked by

No code prerequisite. Native startup proof is explicitly deferred until the owner resumes testing. This issue must not be moved to `issues/done/` or called fully accepted without that proof. The separate open-card click-through gap does not block this startup-only correction.

## User stories addressed

US1/AC4–AC5. Full US1 and all six baseline success criteria retain their actual separate status.
