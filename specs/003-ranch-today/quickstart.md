# Validation Guide: Ranch Today

**Status**: Draft guide, not an execution report. Planning inspected source and test definitions only. See [contract coverage](contracts/current-behavior.md) and [data model](data-model.md).

## Prerequisites and isolation

Use the [development setup](../../README.md), repository toolchains, Node, Task, and RTK. macOS is the native design target. Run from the repository root. Before final implementation validation, synchronize the validation branch with current main and record its commit; preserve unrelated changes.

Use synthetic data in a fresh short temporary path to fit Unix socket path limits. Quit other Ranch instances first. Do not copy personal Focuses, Claude settings, credentials, or transcripts.

```sh
rtk mktemp -d /tmp/ranch003.XXXXXX
```

Replace `/tmp/ranch003.ABC123` below with the directory printed above. Give Ranch a separate data root and empty Claude configuration:

```sh
rtk mkdir -p /tmp/ranch003.ABC123/ranch /tmp/ranch003.ABC123/claude
rtk env ADHD_RANCH_HOME=/tmp/ranch003.ABC123/ranch CLAUDE_CONFIG_DIR=/tmp/ranch003.ABC123/claude task dev
```

Do not choose Install Hooks during basic smoke checks. A separate Claude process needs the same environment values for the optional live check.

## Existing automated gates

```sh
rtk task test:rust
rtk task test:web
```

Existing proof targets:

| Area | Tests already in the repository |
|---|---|
| Markdown edits, sidecars, positional mapping | [Focus store](../../crates/storage/src/focus_store.rs) |
| Timer start/Clear/expiry/revival and projection | [Timers](../../crates/commands/src/timers.rs), [Focus Animals](../../crates/commands/src/focus_animals.rs) |
| Scene shape, mixed selection, movement, Escape | [Render contract](../../crates/commands/tests/render_scene_contract.rs), [App](../../src/components/App.test.tsx), [movement](../../src/lib/animalMovement.test.ts), [detail](../../src/components/AnimalDetail.test.tsx) |
| Chips, concurrent saves, Limits crossings | [Settings workflow](../../src-tauri/src/app/settings_workflow.rs), [Limits](../../crates/commands/src/caps.rs) |
| Plugin health/repair, client, unchanged Firings, delivery/absent-app success | [Plugin](../../src-tauri/src/app/claude_plugin.rs), [client](../../src-tauri/src/app/claude_hook.rs), [journal](../../crates/storage/src/agent_hooks/journal.rs), [socket integration](../../crates/storage/tests/agent_hook_socket.rs) |

A later shipping slice must pass `rtk task check`: lint, typecheck, tests, generated-type drift. Platform changes also need relevant gates: `rtk task check:windows` checks cross-target compilation; real Windows bundling uses [Windows CI](../../.github/workflows/windows.yml), Linux packaging the [release matrix](../../.github/workflows/release.yml). Compilation is not native behavioral proof. Planning does not run release workflows or add tests for prose.

## Native scenarios

For each scenario record pass/fail/unverified, platform, commit, and evidence. On first fixture launch expect one example Focus. Delete it and restart: its marker prevents reappearance. Create two clearly named Focuses for further checks.

### V1 — Files, card, and Timers

Rename and duplicate a Focus; compare files and Animals. Add/edit two Tasks, Complete/undo one, then Delete it. Inspect fixture `focuses/<id>/focus.md`: Complete checks a retained line, Delete removes it. Blank Task input must not create a Task.

Start/restart/Clear Focus and Task Timers and inspect their owning sidecars; Clear retains work. Check built-ins 10/30/60 and shared custom chips across pickers/restart. Remove saved chips during a running Timer: Timer and built-ins remain.

Use short custom Timers to observe Focus growth then expiry (ghostly/resting/away, tray Expired). Adding a Task revives that Focus. Task Timer expiry changes its status/allowed notification without changing Animal size/motion or tray Expired membership; adding a Task never revives another Task's Timer. In disposable fixture copies, missing/corrupt optional sidecars must still permit loading the Focus. Existing tests cover atomic-write failure cases.

### V2 — Hand edits and durability

Append a nonempty checkbox in fixture Markdown; time save-to-card visibility. Complete a checkbox externally and repeat. Record actual elapsed time against SC-003's approximately one-second outcome; debounce is not a measurement. Restart and verify work, Timers, and saved Settings survive. Hand-edited Settings apply on restart, not by live Settings-file watching.

### V3 — Ambient overlay and displays

Name the open Focuses from visible labels. Confirm one Pig per Focus, drag/toss, and Gather to primary. Click empty overlay space: the underlying app receives it. Close a Focus card by outside click/Escape. Work in another app: Ranch must not raise a window or take attention on its own.

Verify primary-only defaults; with multiple displays, enable another and check movement stays in visible regions rather than gaps. Mark that portion unverified without appropriate hardware. Native click-through, attention, visual usefulness, and ghosting need observations beyond component tests.

### V4 — Tray, Settings, and soft Limits

Check New Focus, cancel/confirm delete, Gather, Agents toggle, Settings, and quit. Cancellation preserves the Focus. Verify default Limits 5/7, then lower them for a short crossing test. Writes above either threshold still save, tray turns red, and an enabled notification fires once per crossing rather than repeatedly. Completed Tasks still count. Repeat with notifications disabled: no notification fires. App Settings apply immediately and persist after restart.

### V5 — Agent independence and delivery

Run isolated plugin/socket/journal tests first; these are not a census of live sessions. Fresh UI defaults to Agents off. Enabling alone must not install a plugin; opening an Install Hooks dialog is distinct from executing installation.

An optional live check requires a disposable Claude configuration that can run independently. Do not borrow credentials from the normal environment. Launch Claude with the same `ADHD_RANCH_HOME` and `CLAUDE_CONFIG_DIR` as Ranch, then explicitly choose Install Hooks. Record configuration health separately from an observed Firing, including an unchanged repeat. The session must emit a hook after Ranch starts listening; there is no cold-start census. Mark live delivery unverified if the isolated environment cannot run.

With an observed session, show its Animal, click it (no Focus card), and confirm activity neither changes work files nor contributes to Limits. Toggle visibility off: Focus Animals and installed plugin remain. Compare fixture files before/after activity. Do not read personal transcripts. Windows hook delivery is unsupported.

### V6 — Runtime boundaries and non-goals

Review writers, exposed commands, and dependencies for the exclusions in the contract map: no sync/import service, external work writer, auto-completion/merging, Harness notification forwarding, Wrangle, second Species/Harness, outbound client, telemetry, model call, or stored provider credential. This is scoped source evidence, not universal runtime proof.

Observe a packaged Ranch process using platform process-scoped network inspection during core scenarios. Record tool, process, time window, and exercised actions: expect no outbound product connection or network API. Vite development traffic does not establish SC-006. Report explicit local Claude CLI actions separately. Mark runtime verification unverified without a packaged build or suitable observation tool.

### V7 — Known limitations

Record FR-013 positional drift, FR-014 transport/event gaps, and FR-015 retained legacy entries as source/fixture observations. Do not require reproducing them in personal data or fail future work because an approved feature fixes them. Explicitly report ADR-0002/current-code drift; [research](research.md) links follow-ons. No fixes are included in this baseline pass.

## Completion evidence

Record commit, platform, commands/exit statuses, native observations, unverified checks, and linked discrepancies in later task/issue evidence. Map all six SCs using the contract. Source inspection and past CI are not fresh success. Stop after baseline evidence is reconciled; runtime changes require separately approved scope, gates, review, and merge.
