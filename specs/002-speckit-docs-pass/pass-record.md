# Pass record

## How to read this

- `PRD.md` was removed after `specs/003-ranch-today/spec.md` existed. Git history still has it. It is not a current requirement.
- Finished issue write-ups and research notes that remain are not requirements.
- The current description is `specs/003-ranch-today/spec.md` and its status is Draft until the owner approves it.
- Accepted decisions remain in `docs/adr/`. Names remain in `CONTEXT.md`.

## Inventory

| Source | Class | Checked | Disposition |
|---|---|---|---|
| `PRD.md` | Historical context, plus verified observations that were revalidated | Compared with `docs/architecture.md`, accepted ADRs, and `main` on 2026-10-15. Finished phases, the localhost HTTP API, and the proposal queue are not current requirements | Deleted after carry-forward |
| `CONTEXT.md` | Accepted decision for names. The review banner was present direction that had gone stale | Each entry has a definition. Animal cites ADR-0001. No entry is marked unsettled | Kept as authority. Banner corrected |
| `docs/architecture.md` | Verified observation of the implementation. A few sentences used retired words or called every Animal a pig | Compared with the constitution, ADR-0001, ADR-0003, and `specs/003-ranch-today/spec.md` on 2026-10-15 | Kept as the implementation description. Contradicting sentences corrected |
| `docs/adr/0001-two-apps-over-one-renderer.md` | Accepted decision | Status line is Accepted | Kept as authority |
| `docs/adr/0002-agent-hook-transport.md` | Accepted decision | Status line is Accepted. Its unfinished user-visible consequences are issues 059, 060, and 062 | Kept as authority |
| `docs/adr/0003-rust-owned-render-scene.md` | Accepted decision | Status line is Accepted. `specs/001-render-domain-seam/` is the record of the shipped work | Kept as authority |
| `docs/research/serena-multi-agent.md` | Historical context | Not a behavior claim about the ranch product | Marked historical |
| `docs/research/manual-github-release-retention.md` | Historical context | Not a behavior claim about the ranch product | Marked historical |
| `docs/research/claude-codex-session-terminology.md` | Historical context | Glossary of vendor terms. `CONTEXT.md` remains the ranch's names | Marked historical |
| `specs/001-render-domain-seam/spec.md` | Accepted decision for that shipped feature | Implemented in PR #88. Consolidation updates only its lifecycle status, not requirements | Kept as authority |
| `issues/README.md` | Present direction for the queue | Compared with the issue files on 2026-10-15 | Updated so the queues match the classification |
| `issues/031-notification-source-settings.md` | Present direction | `src/components/SettingsWindow.tsx` still contains `NOTIFICATION_SOURCES` | Still to do. See classification |
| `issues/051-ranch-animal-vocabulary-seam.md` | Present direction | The scene and movement cutover shipped; `src/hooks/useAnimalMovement.ts` still imports the generic `PigHitRect` and Pig-named TypeScript wrappers | Still to do. See classification |
| `issues/056-task-timer-binding.md` | Present direction | `crates/storage/src/focus_store.rs` still stores Task Timers by index. No reorder repair | Still to do. See classification |
| `issues/059-self-describing-hook-frame.md` | Present direction | No late-frame stamp and no distinct failed-end Firing in the hook path | Still to do. See classification |
| `issues/060-session-staleness-sweep.md` | Present direction | `docs/architecture.md` still calls stale-session recovery future work. No such sweep is in the code | Still to do. See classification |
| `issues/061-hook-change-events.md` | Present direction | No `SessionChange` type. Delivery still emits a session-list refresh | Still to do. See classification |
| `issues/062-stable-hook-client-path.md` | Present direction | `src-tauri/src/app/claude_hook.rs` copies a stable client and does not write Claude settings. It also does not remove older direct entries | Still to do. See classification |
| `issues/icebox/021-all-monitors-default.md` | Present direction, deferred | Primary display remains the default | Left deferred |
| `issues/icebox/022-wrangle.md` | Present direction, deferred | Gather exists. Wrangle does not | Left deferred |
| `issues/done/` (49 files) | Historical context | The archive is the finished write-ups. This pass added a directory mark and did not rewrite the old bodies, except the shipped note on 046 | Marked historical |
| Closed-without-building notes 042, 045, 047 | Historical context | No files exist for these ids. The index still states why each was closed | Left closed |

## Issue classification

| Issue | Class | Evidence | Owning specification |
|---|---|---|---|
| 031 `issues/031-notification-source-settings.md` | still to do | The promise is that Settings toggles come from the Rust source registry. `SettingsWindow.tsx` still has a hardcoded `NOTIFICATION_SOURCES` list. The visible toggles already exist, so this is an internal seam, not a follow-on spec | `specs/003-ranch-today/spec.md` — FR-008 |
| 051 `issues/051-ranch-animal-vocabulary-seam.md` | still to do | The broad movement/scene cutover shipped in PR #88. `PigHitRect` and Pig-named TypeScript IPC wrappers remain generic. Keep only that adapter rename open; retain wire names as compatibility contracts | `specs/003-ranch-today/spec.md` — FR-004 |
| 056 `issues/056-task-timer-binding.md` | still to do | The promise is that a Task Timer stays on its Task across a hand-edit, or that the drift is a decided and tested behavior. Storage is still by position, and no reorder test decides it. The person can see the Timer move | `specs/004-task-timer-hand-edits/spec.md` |
| 059 `issues/059-self-describing-hook-frame.md` | still to do | The promise is that a late frame is ignored and Agent Hooks can tell an ended turn from a died turn. Neither is true on `main`. Both are visible in the session Animal or the Agent Hooks window | `specs/005-late-hook-frame/spec.md` |
| 060 `issues/060-session-staleness-sweep.md` | still to do | The promise is that a session whose turn ended unheard stops being drawn as working, without a restart. Architecture still lists that recovery as future work | `specs/006-unheard-turn-ends/spec.md` |
| 061 `issues/061-hook-change-events.md` | still to do | The promise is that a consumer can see a session change without comparing two lists. No `SessionChange` is exported. The person does not gain a new visible behavior. The overlay already refetches the list | `specs/003-ranch-today/spec.md` — FR-014, under ADR-0002 |
| 062 `issues/062-stable-hook-client-path.md` | still to do | The promise is that an entry the ranch wrote can be removed by the ranch, and one firing arrives once. The stable client exists. Older direct settings entries are not removed | `specs/007-one-hook-delivery/spec.md` |
| 021 `issues/icebox/021-all-monitors-default.md` | deferred | The owner deferred multi-monitor default-on. The file stayed in `issues/icebox/` | None. Deferred |
| 022 `issues/icebox/022-wrangle.md` | deferred | The owner deferred Wrangle. Gather remains the shipped action | None. Deferred |
| 042 | superseded | The index says the premise was wrong: both consumers already read the same cap-state projection. No file. Not copied into the current spec | None. Not active |
| 045 | superseded | The index says the helper was too thin and the work folded into 039. No file | None. Not active |
| 046 `issues/done/046-timer-expiry-service-extraction.md` | already satisfied | The promise is one tested workflow module, with `app/timer_expiry.rs` only wiring the loop. `crates/commands/src/timers.rs` owns `expire_due` and its tests. `src-tauri/src/app/timer_expiry.rs` spawns the one-second loop and adapts notifications. The type names differ from the write-up. The index had listed 046 under closed-without-building; that line was the lower-authority statement and was removed | None. Not active |
| 047 | superseded | The index says the proposal queue and HTTP API were removed, so the storage transaction for that queue was not built. No file | None. Not active |

## Corrected

- `issues/README.md`: removed 046 from the closed-without-building list. The outcome shipped. Higher authority: the promise checked against `crates/commands/src/timers.rs` and `src-tauri/src/app/timer_expiry.rs`.
- `docs/architecture.md`: the scope line and the interaction loop called every sprite a pig and called Limits caps. Higher authority: `CONTEXT.md` and `specs/003-ranch-today/spec.md`.
- `README.md`: the design pointer named `PRD.md`, and the opening said one pig per Focus. Higher authority: `specs/003-ranch-today/spec.md`.
- `CONTEXT.md`: the banner said the language was under review. The entries are settled. Higher authority: the glossary entries and ADR-0001.
- `CHANGELOG.md` `[0.1.4]`: the entry says hook install writes Claude's `settings.json` and that a dead session has nothing to go stale. Left in place. It is the note for that release, not a current requirement. Current behavior is FR-014 and FR-015 in `specs/003-ranch-today/spec.md`.
- `CLAUDE.md`: no sentence contradicted the current specification. No edit.

## Unresolved

None.

## Consolidation with the saved foundation

The owner requested consolidation on 2026-10-15 after PR #89 merged. The original documentation
pass remains above; the combined cleanup also preserves the earlier setup and the mechanical
HTML entrypoint move as separate changes.

- `main` at `8fa695d` includes PR #89. The product draft and architecture now describe built-in
  10/30/60 minute chips and persisted custom chips, replacing the stale 2/4/8/16/32 description.
- Spec 001's status and ADR-0003's completed action items now reflect PR #88. Its requirements and
  recorded validation are unchanged. The separate backend-movement decision remains open.
- Issue 051 stays active only for the remaining generic TypeScript hit-test adapter names. The
  earlier foundation branch closed it too broadly; compatibility wire names remain exempt.
- Issues 057 and 058 remain in `issues/done/`; their original proposals are historical.
- The obsolete `issues/done/024-display-subsystem-agent-prompt.md` is removed; the actual shipped
  issue remains at `issues/done/024-display-subsystem.md` and the prompt remains in Git history.
- The orchestrator and lane agents from `chore/project-foundation` are retained for Claude,
  Codex, and Cursor. Both installed Spec Kit integrations are retained, with Codex as default.
- The archive design and accepted ADR-0004 are preserved as `specs/008-claude-session-storage/`.
  Its former 002 prefix collided with the documentation pass. Renumbering does not approve new
  implementation or change the archive decisions.
- HTML sources move under `src/`, while Vite still serves the same five page URLs and builds them
  into root `dist/` for Tauri. This is checked separately from the document reader checks.
- Ranch Today and specs 004–007 remain Draft. Review with the owner is still required before
  treating their proposed requirements as authority.

### Consolidation validation

- `task check` passed: Rust/frontend lint, unused-code checks, TypeScript, all unit tests, and
  generated-type drift. Application TypeScript, Rust, generated types, and dependencies are unchanged.
- All five relocated HTML documents match their originals except equivalent module paths. Each
  existing page URL and module entry returned HTTP 200 from Vite; production HTML and assets remain
  under root `dist/`, matching Tauri's existing URLs.
- JSON/TOML parsing, Spec Kit manifest hashes, shell syntax and command-format checks, local
  Markdown links, and `git diff --check` passed.
- The spec 008 files match the saved archive planning apart from the 002-to-008 path renumbering.
  Product and follow-on spec lifecycle states are unchanged.
- Native desktop interaction was not rerun; the entrypoint change is covered by content, HTTP,
  production-output, and regression-gate checks.
