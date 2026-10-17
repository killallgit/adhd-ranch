# US5 — Agent Observation and Diagnostics

Fresh gate and runtime commit: [pass record](pass-record.md#automated-gate-t004), `rtk task check`, exit 0. Controlled integration fixtures are distinct from real Claude delivery.

| Task | Result | Limit |
|---|---|---|
| T016 | PASS — source and existing controlled tests reviewed | No real-session census or plugin installation |
| T017 | Automated install-dialog/default-off behavior PASS; native UI UNVERIFIED | No actual toggle/dialog action operated |
| T018 | UNVERIFIED — optional live/native check unavailable | Agent Hooks window, live delivery, Limit/file independence and toggle journey remain open |

[Agent Session](../../../crates/domain/src/session/agent_session.rs) has identity/name/Pen/activity, no Focus/Task/Timer fields; activity is Working/Idle. [Hook types](../../../crates/domain/src/agents/hooks.rs) define Start/Working/Idle/End and optional session/pen diagnostic fields. [LiveSessions](../../../crates/storage/src/agent_session_store.rs) is an in-memory arrival-order fold; unknown sessions can become visible on a later establishing/updating Firing, with no startup census. [Journal](../../../crates/storage/src/agent_hooks/journal.rs) retains unchanged Firings without retaining raw prompt payloads.

[Plugin owner](../../../src-tauri/src/app/claude_plugin.rs) validates configuration and gates installation to the Install Hooks window. [Stable client placement](../../../src-tauri/src/app/claude_hook.rs) atomically replaces a local client and preserves the prior binary on failure. Disabling via [Settings effects](../../../src-tauri/src/app/settings_workflow.rs) clears local sessions, not the installed plugin. [Host hook adapter](../../../src-tauri/src/app/agent_hooks.rs) ignores Firings while disabled. No personal configuration or live CLI installation was touched.

Fresh plugin tests verify complete matching user-scope identity, idempotent installation, update/repair/reinstall paths, and preservation on invalid marketplace sources. Stable-client tests cover replacement/failure isolation. [Socket integration](../../../crates/storage/tests/agent_hook_socket.rs) passed real temporary-socket/client/wrapper delivery and silent success with Ranch absent. The client/server use bounded payload/read/write policies. [App tests](../../../src/components/App.test.tsx) passed non-editable Agent selection and independent Focus/Agent changes; [Limit source](../../../crates/domain/src/caps.rs) cannot count Agent Sessions.

[Agent Hooks view source](../../../src/components/AgentDebugWindow.tsx) renders wiring/configuration health, live sessions, recent Firings, and “no change” rows; loading/error states are distinct from empty lists. This source review does not prove that an actual native window displayed those results.

T018's live check was not attempted: native control is disabled, and no independently usable disposable authenticated Claude environment was provisioned. No credentials were borrowed to make it run. Native checks explicitly UNVERIFIED: opening Agent Hooks and inspecting all three sections/unchanged repeat, actual Install Hooks/repair, an observed post-listener session, no Focus card on click, before/after work-file/Limit comparison, and toggle-off retaining Focus Animals/plugin. SC-004's live Agent portion and SC-005 remain open. Windows hook delivery is unsupported, not a failing Unix check.
