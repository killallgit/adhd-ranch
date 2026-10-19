# Research: Ranch Today

**Date**: 2026-10-18 | **Status**: Draft planning evidence

Research target: the clarified [specification](spec.md), revalidated against local `main` at `7900cc445ac224464e254b31d4914b0a52055270`. The five 2026-10-18 clarifications, including the later owner decision to resume saved work without seeding, update the prior 2026-10-17 planning record. Source inspection and read-only research-agent findings inform this record; no tests or native smoke checks were executed during this planning update. Existing [validation and release evidence](validation/pass-record.md) remains historical, with no refined acceptance criterion promoted to passed.

## Source inventory and authority

| Classification | Source | Use in this plan |
|---|---|---|
| Governing rules | [Constitution](../../.specify/memory/constitution.md) | Local ownership, independent domains, typed boundaries, explicit integration lifecycle, and evidence requirements. |
| Accepted decisions | [ADR-0001](../../docs/adr/0001-two-apps-over-one-renderer.md), [ADR-0002](../../docs/adr/0002-agent-hook-transport.md), [ADR-0003](../../docs/adr/0003-rust-owned-render-scene.md) | Separate domains and a Rust-owned flat render scene; accepted hook direction is distinguished from current implementation below. |
| Present user direction | [Clarifications](spec.md#clarifications), 2026-10-17 and 2026-10-18 | FR-013–FR-015 describe observations that follow-on work may fix. The owner has selected removal of automatic startup example creation: load persisted work/Settings, preserve data, and start empty when none exists. The owner accepted precise interaction, motion, latency, and duplication criteria. The planning request authorizes these draft artifacts; it does not silently change the specification's lifecycle status. |
| Verified source observations | [Workspace](../../Cargo.toml), [frontend manifest](../../package.json), [architecture](../../docs/architecture.md), source and test files linked below | Existing stack, storage, boundaries, and available proof. Test definitions establish coverage intent, not fresh passing results. |
| Settled vocabulary | [CONTEXT.md](../../CONTEXT.md) | Focus, Task, Timer, Animal, Agent Session, Limit, Complete, Delete, and Clear. |
| Non-normative context | [002](../002-speckit-docs-pass/spec.md), [004](../004-task-timer-hand-edits/spec.md), [005](../005-late-hook-frame/spec.md), [006](../006-unheard-turn-ends/spec.md), [007](../007-one-hook-delivery/spec.md), [008](../008-claude-session-storage/spec.md) | Documentation history and separate draft features. None supplies additional requirements for 003. Removed PRD content and icebox proposals are not design authority. |

## Decisions

### 1. Validate the existing product rather than rebuild it

- **Decision:** Keep the baseline evidence pass and add only the owner-selected startup correction in decision 8. Reuse existing code for state loading; no new dependencies.
- **Rationale:** The spec describes the baseline plus an explicit correction to startup seeding. [Focus workflows](../../crates/commands/src/focus.rs), [Timers](../../crates/commands/src/timers.rs), [scene projection](../../crates/commands/src/render_scene.rs), and [host composition](../../src-tauri/src/app/mod.rs) already implement the corresponding paths.
- **Alternatives considered:** A second implementation or folding all follow-on specs into this plan would duplicate working behavior and expand scope beyond the owner's clarification.

### 2. Keep the current file model and generated contracts

- **Decision:** Describe the existing Markdown, timer sidecars, Settings, and Rust-to-TypeScript contracts; introduce no schema or migration.
- **Rationale:** [Focus storage](../../crates/storage/src/focus_store.rs) owns `focus.md`, `timer.json`, and `task-timers.json`. [Parsing](../../crates/domain/src/parse.rs) derives Task IDs from Focus ID and checkbox position. [Paths](../../src-tauri/src/app/paths.rs) resolves the Ranch data root. These are current observations, not a reason to preserve positional drift.
- **Alternatives considered:** Stable Task IDs, a database, and a data-root rename belong to separately approved features. Generated TypeScript is not hand-edited documentation.

### 3. Preserve domain ownership at the render boundary

- **Decision:** Validate Focus and Agent Session projections independently, then their composition into `RenderScene`.
- **Rationale:** ADR-0003 refines ADR-0001 by assigning projection and layout to Rust. [Focus projection](../../crates/commands/src/focus_animals.rs), [Agent projection](../../crates/commands/src/agent_animals.rs), and [the scene contract test](../../crates/commands/tests/render_scene_contract.rs) make the seam observable. The movement and sprite layers consume renderer vocabulary.
- **Alternatives considered:** Adding domain flags or Timer/Hook logic to movement would violate the accepted boundary and is unnecessary for this work.

### 4. Treat hook installation and delivery as separate evidence

- **Decision:** Reuse isolated plugin, journal, and socket tests; reserve a real Claude run for a deliberately isolated manual check.
- **Rationale:** [Plugin validation/install](../../src-tauri/src/app/claude_plugin.rs) and [stable client placement](../../src-tauri/src/app/claude_hook.rs) cover configuration. [Socket tests](../../crates/storage/tests/agent_hook_socket.rs) cover delivery, including absent-app success. [Journal tests](../../crates/storage/src/agent_hooks/journal.rs) retain unchanged Firings. Configuration health alone does not prove that a live session delivered anything.
- **Alternatives considered:** Automatically installing into the developer's normal Claude configuration or reading personal transcripts is unnecessary. Cold-start census and transcript ingestion are outside this baseline.

### 5. Record accepted-decision drift without enshrining it

- **Decision:** Explicitly record the ADR-0002 implementation gap; do not implement or assert preservation of FR-013–FR-015 in this baseline.
- **Rationale:** ADR-0002's accepted decision calls for stamped frames, stale-frame rejection, and change-carrying events, and its checklist reports the first slice complete. Current [socket parsing](../../crates/storage/src/agent_hooks/server/unix.rs) still accepts `verb` plus payload, [session storage](../../crates/storage/src/agent_session_store.rs) folds arrival order, and [host events](../../src-tauri/src/app/agent_hooks.rs) emit unit invalidations. [The frontend reader](../../src/api/tauriAgentSessionReader.ts) refetches the list. The accepted direction therefore cannot be reported as fully shipped. [059](../../issues/059-self-describing-hook-frame.md) and [061](../../issues/061-hook-change-events.md) describe the transport/event follow-ons; [060](../../issues/060-session-staleness-sweep.md) describes missed-event recovery. These issue proposals are context, not additional authority or implementation instructions here.
- **Alternatives considered:** Claiming the ADR is fully implemented would be false. Reproducing bugs as permanent acceptance tests would contradict the user's answer. Silently changing the accepted ADR would conceal the discrepancy.

### 6. Use existing gates and distinguish manual gaps

- **Decision:** Use [Taskfile](../../Taskfile.yaml) gates and existing tests, then record manual observations for native behavior.
- **Rationale:** Existing tests cover core rules, persistence, render serialization, settings concurrency, plugin validation, and socket delivery. They do not establish OS click-through, attention behavior, visual usefulness, tray color, display gaps, actual notifications, or full watcher-to-UI latency. The [host watcher](../../src-tauri/src/app/mod.rs) debounces for 200 ms; [the reader](../../src/api/tauriFocusReader.ts) refetches on `focuses-changed`, and [the hook](../../src/hooks/usePolledReader.ts) updates the rendered state. None proves SC-003's save-completion-to-visible-row latency of at most 1.0 second in an already-open, visible card for a valid local Focus file.
- **Alternatives considered:** New tests solely for these planning documents would mirror prose. A dev-server connection is also not evidence of a product network API: Vite is development infrastructure. SC-006 needs a packaged-app observation, separately scoped from an explicit local Claude CLI action.

### 7. Make duplication, motion, and interaction checks explicit

- **Decision:** Reuse the existing implementation and document the clarified behavior in the data model, UI contract, and native guide. A duplicate gets fresh identity/creation time, copied description and ordered Task text/completion, the collision-safe “ copy” title, and no Focus or Task Timers. Focus Timer absence/Running means Walking, Expired means Resting; Task Timers do not control motion. Animals and an open Focus card receive interaction; empty space passes through, closes an open card, and Escape also closes it.
- **Rationale:** [Focus commands](../../crates/commands/src/focus.rs) create the duplicate through the normal creation path and append Tasks without copying Timer state; [storage](../../crates/storage/src/focus_store.rs) persists it. [Focus projection](../../crates/commands/src/focus_animals.rs), [App](../../src/components/App.tsx), and [hit testing](../../src/hooks/useAnimalMovement.ts) own motion projection, card interaction, and OS hit regions. Source inspection supports duplication and motion. It exposes an interaction mismatch: selected Focus state makes `syncRects` submit a whole-overlay hit rectangle, while the card renders an interactive backdrop. The [native hit-test loop](../../src-tauri/src/display/overlay.rs) honors that region, so outside clicks close the card but are consumed. This does not satisfy clarified FR-004/FR-005 / US1/AC2. A later runtime slice needs actual Animal/card bounds plus a way to observe an outside click without intercepting it; the current loop observes cursor position only. No replacement or new OS permission is selected by this documentation plan. Existing tests are not native acceptance evidence.
- **Alternatives considered:** Copying Timers, changing Task identity, routing Task expiry into motion, or making card controls click-through would contradict the clarified spec. Preserving the full-overlay backdrop would retain the discovered mismatch; dropping it without an outside-click observation path would lose required closure. The runtime fix must be separately scoped and designed.

### 8. Resume saved work without seeding

- **Decision:** Implement the owner's refined startup rule by deleting the sole `seed::ensure_example_focus` call and module declaration from [startup composition](../../src-tauri/src/app/mod.rs), then deleting `src-tauri/src/app/seed.rs` and its two obsolete seeding tests. Update [architecture](../../docs/architecture.md). Load existing work/Settings through the current paths; start empty without work. No replacement function, flag, schema migration, or onboarding UI.
- **Rationale:** The only production seed caller is startup. Its adapter creates a Welcome Focus, two Tasks, and `.example-focus-seeded`; even the existing-work branch writes the marker. Removing this path removes the unauthorized work writer. [Focus storage](../../crates/storage/src/focus_store.rs) reads canonical work/sidecars and ignores non-directory entries, so old marker files can remain untouched. [Settings loading](../../src-tauri/src/app/mod.rs) already precedes app setup. [The expiry worker](../../src-tauri/src/app/timer_expiry.rs) checks saved Timers using the current clock; [Timer transitions](../../crates/commands/src/timers.rs) retain their original start/duration on expiry. No new restart semantics are needed. This design conforms to Constitution I; the current runtime is not corrected by writing the plan.
- **Alternatives considered:** Keeping automatic examples would contradict the owner and Constitution I. Making seeding opt-in would add unrequested UI. A no-op seeding function or startup service would retain an unnecessary abstraction. Removing old example Focuses or markers would mutate existing user data without need. Changing Timer deadlines or restoring ephemeral Agent sessions would exceed the persisted-work scope.
- **Proof boundary:** Existing storage tests cover empty/missing roots, canonical files, Timer sidecars, and create/list round trips; command tests cover due/not-due expiry and timer-only changes. They do not execute the native host startup. Do not replace the removed seed tests with source-text assertions or no-op tests. Validate actual launch/restart and file outcomes with V0, then fresh current-source host/Windows gates; keep native proof open while the owner has deferred testing. Existing package results apply to the old runtime only.
- **Compatibility/rollback:** No persisted file format changes. Preserve old Welcome Focuses as normal user-owned work and leave `.example-focus-seeded` untouched. Rolling back the binary is data-compatible, but older versions may seed an empty unmarked root again; no new marker write is added to hide that old behavior.

## Resolution

No unresolved technical choices remain for the bounded startup correction: the owner selected the policy and the existing read/expiry paths support the deletion design. The source correction is implemented; native runtime compliance is not claimed. The outside-click mechanism remains intentionally unselected for a separate runtime slice, and ADR-0002 reconciliation remains outside this correction. Native testing remains deferred, not waived. The current stack is retained at its repository-declared versions; no new library or external API research is needed.
