# Current UI and IPC Contracts

**Status**: Draft baseline evidence, 2026-10-17. This maps the clarified [spec](../spec.md) to existing interfaces, without introducing endpoints or wire formats. Constitution I–IV govern these boundaries even where implementation is incomplete.

## Boundary ownership

- **User → work:** card/tray actions use [Focus writer](../../../src/api/focusWriter.ts) and [Tauri bridge](../../../src-tauri/src/ui_bridge/mod.rs). Hand-edits enter through the watcher. Task operations address a Focus and current index; Timer operations use `TimerOwner`. Complete, Delete, and Clear are distinct.
- **Core → UI:** [Focus reader](../../../src/api/tauriFocusReader.ts), [scene reader](../../../src/api/tauriRenderSceneReader.ts), [Settings API](../../../src/api/settings.ts), and [Agent diagnostics](../../../src/api/agentDebug.ts) expose existing commands/events. Rust owns wire shapes and generated TypeScript; no duplicate schema is introduced.
- **Domains → renderer:** [RenderScene](../../../crates/commands/src/render_scene.rs) is flat. [App selection](../../../src/components/App.tsx) resolves Focus details separately. Agent Animals have no Focus editing, Timer ownership, or Limit contribution.
- **Claude → Ranch:** explicit plugin install uses the local CLI at user scope. A stable client delivers bounded Unix-socket frames; Windows has no delivery. Plugin health is separate from observed delivery. [Research](../research.md) distinguishes current shortcomings from accepted ADR direction.
- **Network:** no Ranch HTTP/network API. Vite development traffic and effects of explicit local Claude CLI actions must not be misreported as Ranch product network features.

## Observable contract and coverage

V1–V7 refer to [quickstart](../quickstart.md). FR-013–FR-015 require accurate observations, not tests requiring bugs to remain.

| Spec entries | Observable contract | Existing proof owner / validation |
|---|---|---|
| FR-001, FR-002 | Canonical local files; explicit edits; Complete retains, Delete removes, Clear affects only Timer; malformed sidecars do not block Focus. | [Store](../../../crates/storage/src/focus_store.rs), [workflows](../../../crates/commands/src/focus.rs); V1/V2. |
| FR-003 | One Timer owner; built-ins 10/30/60 and shared saved custom chips; chip removal leaves running Timer; add Task revives only Expired Focus. | [Timers](../../../crates/commands/src/timers.rs), [Settings workflow](../../../src-tauri/src/app/settings_workflow.rs); V1. |
| FR-004, FR-005, FR-009 | One Pig per Focus; enabled displays/no gaps; drag/toss/Gather; empty-space click-through; Focus-only card closes outside/Escape. | [Scene contract](../../../crates/commands/tests/render_scene_contract.rs), [App tests](../../../src/components/App.test.tsx), [movement tests](../../../src/lib/animalMovement.test.ts); V3. |
| FR-006, FR-007, FR-008 | Tray actions, delete confirmation, red Over Limit; soft defaults 5/7; editable Settings apply immediately; writes allowed; Agents excluded. | [Tray](../../../src-tauri/src/app/tray.rs), [Limits](../../../crates/commands/src/caps.rs), [Settings workflow](../../../src-tauri/src/app/settings_workflow.rs); V4/V5. |
| FR-010, FR-011 | Agents default off; non-editable observed sessions; toggle leaves plugin; explicit install/repair; diagnostics include unchanged Firings. | [Plugin](../../../src-tauri/src/app/claude_plugin.rs), [socket tests](../../../crates/storage/tests/agent_hook_socket.rs), [journal](../../../crates/storage/src/agent_hooks/journal.rs); V5. |
| FR-012, FR-017 | Ambient operation, allowed notifications/user-opened windows only; no outbound product network/model/telemetry; IPC boundary. | Constitution, source review, native/network observation V3/V6. |
| FR-013 | Positional Timer drift is an observation, not a preservation requirement. | [Storage mapping](../../../crates/storage/src/focus_store.rs), draft [004](../../004-task-timer-hand-edits/spec.md); V7. |
| FR-014 | Late-frame eligibility, indistinguishable stop endings, and invalidation/refetch recorded alongside ADR-0002 drift. | [Server](../../../crates/storage/src/agent_hooks/server/unix.rs), [events](../../../src-tauri/src/app/agent_hooks.rs), [reader](../../../src/api/tauriAgentSessionReader.ts); V7. |
| FR-015 | Legacy direct entries remain; new install uses plugin/stable client. No cleanup added. | [Client placement](../../../src-tauri/src/app/claude_hook.rs), draft [007](../../007-one-hook-delivery/spec.md); V7. |
| FR-016 | macOS primary; Linux/Windows packages build; no Windows hook delivery. | [Windows workflow](../../../.github/workflows/windows.yml), [release matrix](../../../.github/workflows/release.yml); platform gates. |
| FR-018, FR-019, FR-020 | No sync/multi-user/import service, external work writer, auto-complete/merge, network API, slash command, or Harness notification forwarding. | Scoped source review V6; fixture file diffs V1/V5 check absence of unsolicited writes in exercised scenarios. |
| FR-021, FR-022 | Primary-only default; Gather/no Wrangle; Pig and Claude Code only. | [Settings](../../../crates/domain/src/settings.rs), [Species](../../../crates/commands/src/render_scene.rs), tray; V3/V4/V6. |

SC-001 → V3; SC-002 → V1; SC-003 → V2; SC-004 → V4/V5; SC-005 → V5; SC-006 → V6. Source inspection and old tests must not substitute for fresh native results.
