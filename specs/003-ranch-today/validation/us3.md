# US3 — Tray, Settings, and Limits

Fresh automated gate and runtime commit: [pass record](pass-record.md#automated-gate-t004), `rtk task check`, exit 0.

| Task | Result | Limit |
|---|---|---|
| T011 | PASS — source/defaults/coverage reviewed | Source facts separated from native observations |
| T012 | UNVERIFIED — native V4 tray/Settings journey | No tray action or OS dialog was operated |
| T013 | Automated transition/gating rules PASS; native crossings UNVERIFIED | Real red icon, notifications, and UI-to-file writes remain open |

[Defaults](../../../crates/domain/src/settings.rs) are 5 Focuses / 7 Tasks, primary display only, confirmation on, Agents off. [Limit rules](../../../crates/domain/src/caps.rs) use Focus count and all stored Tasks, including completed Tasks; Agent Sessions are absent from the evaluator input. [Command evaluator](../../../crates/commands/src/caps.rs) emits only enabled transition notifications. Existing create/append workflows save independently of Limit state.

[Tray source](../../../src-tauri/src/app/tray.rs) routes New Focus, Gather to overlay-0, Agents, Settings, delete, and quit; deletion returns without mutation on cancellation, and over-Limit state selects a red icon. [Settings workflow](../../../src-tauri/src/app/settings_workflow.rs) persists before updating runtime consumers and rebuilding the tray. This is current source evidence, not a native interaction result.

Fresh tests include `focuses_to_over_calls_notifier_once`, `focuses_source_disabled_suppresses_focus_calls_but_not_task_calls`, `tasks_source_disabled_suppresses_task_calls_but_not_focus_calls`, `notification_source_toggle_is_read_from_latest_settings`, `displays_default_is_primary_only`, and Settings persistence/concurrency tests. These support thresholds, transition deduplication, notification source settings, and runtime update paths.

V4 remains UNVERIFIED for actual tray actions, confirmation/cancellation, Preferences menu naming, Settings applying in live windows, restart persistence, saved over-threshold UI writes, visible red tray state, and real notification delivery/suppression. Completed-Task counting is a verified source observation. [US5](us5.md) separately records Agent separation and its unavailable live proof; SC-004 is not fully verified.
