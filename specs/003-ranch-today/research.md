# Research: Ranch Today

**Date**: 2026-10-17 | **Status**: Draft planning evidence

Research target: the clarified [specification](spec.md), checked against local `main` at `a48525d9a89555bf9bc925a20d626c0dc634b4ce`. Source inspection and a read-only research agent informed this record; tests and native smoke checks were not executed during planning.

## Source inventory and authority

| Classification | Source | Use in this plan |
|---|---|---|
| Governing rules | [Constitution](../../.specify/memory/constitution.md) | Local ownership, independent domains, typed boundaries, explicit integration lifecycle, and evidence requirements. |
| Accepted decisions | [ADR-0001](../../docs/adr/0001-two-apps-over-one-renderer.md), [ADR-0002](../../docs/adr/0002-agent-hook-transport.md), [ADR-0003](../../docs/adr/0003-rust-owned-render-scene.md) | Separate domains and a Rust-owned flat render scene; accepted hook direction is distinguished from current implementation below. |
| Present user direction | [Clarifications](spec.md#clarifications), 2026-10-17 | FR-013–FR-015 describe observations that follow-on work may fix. The planning request authorizes these draft artifacts; it does not silently change the specification's lifecycle status. |
| Verified source observations | [Workspace](../../Cargo.toml), [frontend manifest](../../package.json), [architecture](../../docs/architecture.md), source and test files linked below | Existing stack, storage, boundaries, and available proof. Test definitions establish coverage intent, not fresh passing results. |
| Settled vocabulary | [CONTEXT.md](../../CONTEXT.md) | Focus, Task, Timer, Animal, Agent Session, Limit, Complete, Delete, and Clear. |
| Non-normative context | [002](../002-speckit-docs-pass/spec.md), [004](../004-task-timer-hand-edits/spec.md), [005](../005-late-hook-frame/spec.md), [006](../006-unheard-turn-ends/spec.md), [007](../007-one-hook-delivery/spec.md), [008](../008-claude-session-storage/spec.md) | Documentation history and separate draft features. None supplies additional requirements for 003. Removed PRD content and icebox proposals are not design authority. |

## Decisions

### 1. Validate the existing product rather than rebuild it

- **Decision:** Plan a baseline evidence pass for the five existing user stories, with no runtime changes or new dependencies.
- **Rationale:** The spec describes shipped behavior. [Focus workflows](../../crates/commands/src/focus.rs), [Timers](../../crates/commands/src/timers.rs), [scene projection](../../crates/commands/src/render_scene.rs), and [host composition](../../src-tauri/src/app/mod.rs) already implement the corresponding paths.
- **Alternatives considered:** A second implementation or folding all follow-on specs into this plan would duplicate working behavior and expand scope beyond the owner's clarification.

### 2. Keep the current file model and generated contracts

- **Decision:** Describe the existing Markdown, timer sidecars, Settings, and Rust-to-TypeScript contracts; introduce no schema or migration.
- **Rationale:** [Focus storage](../../crates/storage/src/focus_store.rs) owns `focus.md`, `timer.json`, and `task-timers.json`. [Parsing](../../crates/domain/src/parse.rs) derives Task IDs from Focus ID and checkbox position. [Paths](../../src-tauri/src/app/paths.rs) resolves the Ranch data root. These are current observations, not a reason to preserve positional drift.
- **Alternatives considered:** Stable Task IDs, a database, and a data-root rename belong to separately approved features. Generated TypeScript is not hand-edited documentation.

### 3. Preserve domain ownership at the render boundary

- **Decision:** Validate Focus and Agent Session projections independently, then their composition into `RenderScene`.
- **Rationale:** ADR-0003 refines ADR-0001 by assigning projection and layout to Rust. [Focus projection](../../crates/commands/src/focus_animals.rs), [Agent projection](../../crates/commands/src/agent_animals.rs), and [the scene contract test](../../crates/commands/tests/render_scene_contract.rs) make the seam observable. The movement and sprite layers consume renderer vocabulary.
- **Alternatives considered:** Adding domain flags or Timer/Hook logic to movement would violate the accepted boundary and is unnecessary for validation.

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
- **Rationale:** Existing tests cover core rules, persistence, render serialization, settings concurrency, plugin validation, and socket delivery. They do not establish OS click-through, attention behavior, visual usefulness, tray color, display gaps, actual notifications, or full watcher-to-UI latency. The watcher's debounce is not proof of SC-003's approximately one-second outcome.
- **Alternatives considered:** New tests solely for these planning documents would mirror prose. A dev-server connection is also not evidence of a product network API: Vite is development infrastructure. SC-006 needs a packaged-app observation, separately scoped from an explicit local Claude CLI action.

## Resolution

No unresolved technical choices remain for this documentation and validation scope. Runtime verification remains work to execute, not evidence already obtained. The current stack is retained at its repository-declared versions; no new library or external API research is needed.
