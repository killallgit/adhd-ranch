# Current UI and IPC Contracts

**Status**: Draft baseline evidence, revalidated 2026-10-18. This maps the clarified [spec](../spec.md) to existing interfaces, without introducing endpoints or wire formats. Constitution I–IV govern these boundaries even where implementation is incomplete.

## Boundary ownership

- **User → work:** card/tray actions use [Focus writer](../../../src/api/focusWriter.ts) and [Tauri bridge](../../../src-tauri/src/ui_bridge/mod.rs). Hand-edits enter through the watcher. Startup must load saved work/Settings without automatically creating Focuses or Tasks. The current seed path violates this owner-selected rule; the plan removes it without modifying existing work or marker files. Task operations address a Focus and current index; Timer operations use `TimerOwner`. Complete, Delete, and Clear are distinct.
- **Core → UI:** [Focus reader](../../../src/api/tauriFocusReader.ts), [scene reader](../../../src/api/tauriRenderSceneReader.ts), [Settings API](../../../src/api/settings.ts), and [Agent diagnostics](../../../src/api/agentDebug.ts) expose existing commands/events. Rust owns wire shapes and generated TypeScript; no duplicate schema is introduced.
- **Domains → renderer:** [RenderScene](../../../crates/commands/src/render_scene.rs) is flat. [App selection](../../../src/components/App.tsx) resolves Focus details separately. Agent Animals have no Focus editing, Timer ownership, or Limit contribution.
- **Claude → Ranch:** explicit plugin install uses the local CLI at user scope. A stable client delivers bounded Unix-socket frames; Windows has no delivery. Plugin health is separate from observed delivery. [Research](../research.md) distinguishes current shortcomings from accepted ADR direction.
- **Network:** no Ranch HTTP/network API. Vite development traffic and effects of explicit local Claude CLI actions must not be misreported as Ranch product network features.

## Observable contract and coverage

The table states the clarified acceptance contract, not a claim that every current path matches. **Known mismatch:** while a Focus card is open, [frontend hit regions](../../../src/hooks/useAnimalMovement.ts) cover the whole overlay and [the backdrop](../../../src/components/AnimalDetail.tsx) consumes outside clicks. The [native loop](../../../src-tauri/src/display/overlay.rs) honors those regions. Closing the card is implemented; simultaneous pass-through to the underlying app is not established by that path and needs a separately scoped runtime design/fix plus native proof.

V0–V7 refer to [quickstart](../quickstart.md). FR-013–FR-015 require accurate observations, not tests requiring bugs to remain.

| Spec entries | Observable contract | Existing proof owner / validation |
|---|---|---|
| FR-001, FR-002 | Startup loads existing persisted work/Settings, starts empty without work, never seeds or resets work/Timers, and preserves old examples/marker files; normal timer-only expiry applies (V0, US1/AC4–AC5). Canonical local files; explicit edits; Complete retains, Delete removes, Clear affects only Timer; malformed sidecars do not block Focus. Duplication creates fresh identity/creation time, preserves description and ordered Task text/completion, uses the collision-safe “ copy” title, copies no Focus or Task Timers, and leaves the source unchanged (US2/AC6). | [Store](../../../crates/storage/src/focus_store.rs), [workflows](../../../crates/commands/src/focus.rs), [startup](../../../src-tauri/src/app/mod.rs); V0/V1/V2. |
| FR-003 | One Timer owner; built-ins 10/30/60 and shared saved custom chips; chip removal leaves running Timer; add Task revives only Expired Focus. | [Timers](../../../crates/commands/src/timers.rs), [Settings workflow](../../../src-tauri/src/app/settings_workflow.rs); V1. |
| FR-004, FR-005, FR-009 | One Pig per Focus; absent/Running Focus Timer → Walking, Expired → Resting, independent of Task Timers; enabled displays/no gaps; drag/toss/Gather. Animals and an open Focus card receive interaction; empty space outside both passes clicks through and closes an open card; Escape closes it too. | [Scene contract](../../../crates/commands/tests/render_scene_contract.rs), [App tests](../../../src/components/App.test.tsx), [movement tests](../../../src/lib/animalMovement.test.ts); V3. |
| FR-006, FR-007, FR-008 | Tray actions, delete confirmation, red Over Limit; soft defaults 5/7; editable Settings apply immediately; writes allowed; Agents excluded. | [Tray](../../../src-tauri/src/app/tray.rs), [Limits](../../../crates/commands/src/caps.rs), [Settings workflow](../../../src-tauri/src/app/settings_workflow.rs); V4/V5. |
| FR-010, FR-011 | Agents default off; non-editable observed sessions; toggle leaves plugin; explicit install/repair; diagnostics include unchanged Firings. | [Plugin](../../../src-tauri/src/app/claude_plugin.rs), [socket tests](../../../crates/storage/tests/agent_hook_socket.rs), [journal](../../../crates/storage/src/agent_hooks/journal.rs); V5. |
| FR-012, FR-017 | Ambient operation, allowed notifications/user-opened windows only; no outbound product network/model/telemetry; IPC boundary. | Constitution, source review, native/network observation V3/V6. |
| FR-013 | Positional Timer drift is an observation, not a preservation requirement. | [Storage mapping](../../../crates/storage/src/focus_store.rs), draft [004](../../004-task-timer-hand-edits/spec.md); V7. |
| FR-014 | Late-frame eligibility, indistinguishable stop endings, and invalidation/refetch recorded alongside ADR-0002 drift. | [Server](../../../crates/storage/src/agent_hooks/server/unix.rs), [events](../../../src-tauri/src/app/agent_hooks.rs), [reader](../../../src/api/tauriAgentSessionReader.ts); V7. |
| FR-015 | Legacy direct entries remain; new install uses plugin/stable client. No cleanup added. | [Client placement](../../../src-tauri/src/app/claude_hook.rs), draft [007](../../007-one-hook-delivery/spec.md); V7. |
| FR-016 | macOS primary; v0.1.5 macOS universal, Windows x64/ARM64, and Linux x64 packages built; no Windows hook delivery. Native acceptance remains open. | [Windows workflow](../../../.github/workflows/windows.yml), [release matrix](../../../.github/workflows/release.yml); platform gates. |
| FR-018, FR-019, FR-020 | No sync/multi-user/import service, external work writer, auto-complete/merge, network API, slash command, or Harness notification forwarding. | Scoped source review V6; fixture file diffs V1/V5 check absence of unsolicited writes in exercised scenarios. |
| FR-021, FR-022 | Primary-only default; Gather/no Wrangle; Pig and Claude Code only. | [Settings](../../../crates/domain/src/settings.rs), [Species](../../../crates/commands/src/render_scene.rs), tray; V3/V4/V6. |

SC-003 measures successful save completion of a valid local Focus file to the changed Task row visibly matching it in an already-open, visible Focus card: **at most 1.0 second**. This is an acceptance target, not measured performance. See V2 for timing and evidence conditions.

The owner has resolved the startup policy: no seeding, resume saved state, start empty without work. Implementation and V0 proof are pending; the current seed path still conflicts with Constitution I / FR-019. The historical FR-019 source-scope PASS in the [pass record](../validation/pass-record.md) cannot certify that path as authorized. Existing package/gate evidence does not prove corrected runtime source.

SC-001 → V3; SC-002 → V1; SC-003 → V2; SC-004 → V4/V5; SC-005 → V5; SC-006 → V6. Source inspection and old tests must not substitute for fresh native results.
