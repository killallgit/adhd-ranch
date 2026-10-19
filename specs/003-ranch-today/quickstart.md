# Validation Guide: Ranch Today

**Status**: Draft guide, updated 2026-10-18, not an execution report. Planning inspected source and test definitions only. See [contract coverage](contracts/current-behavior.md) and [data model](data-model.md).

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

Native/live testing remains deferred by the owner. These are instructions for the later test pass, not actions performed by this planning command. Existing [pass-record](validation/pass-record.md) results and task completion states remain historical; they do not prove the clarified criteria.

For each scenario record pass/fail/unverified, platform, commit, and evidence. Run V0 against the corrected startup source before creating the two clearly named synthetic Focuses used by V1–V7. Automatic creation of an example Focus/Task is now a failed FR-001 / FR-019 / US1/AC5 observation, not an undecided policy or behavior to preserve.

### V0 — Resume saved work without seeding

Use a fresh isolated data/configuration root for each case. All launches use the same environment setup above and Agents as Animals off; stop Ranch completely between launches. Run V0 against the corrected startup source. V0 must pass; automatic creation of an example Focus/Task is a failure. Native execution remains pending until actual V0 evidence is recorded.

1. **Empty root, no marker:** launch and restart. Expect zero Focus Animals and no example Focus/Task files. An empty storage directory and ordinary logs are permitted; startup must not create `.example-focus-seeded`.
2. **Empty work root, legacy marker:** place a synthetic `.example-focus-seeded` under the fixture `focuses/` directory before launch. Repeat. Expect no seeded work and unchanged marker contents; the marker does not control startup behavior.
3. **Saved work and Settings:** explicitly create synthetic Focuses and mixed completed/uncompleted Tasks, set nondefault Settings, and include a normal saved Focus named Welcome to ADHD Ranch. Before restarting, record file inventory/content and stable work identities. Restart with marker absent, then present. Expect the same work/Settings, no extra example, no deletion of the old example, and no new/modified marker. Compare canonical Markdown and Settings bytes before/after. Do not infer restoration of open cards, Animal positions, or old Agent sessions.
4. **Saved Timers:** include a Running Focus/Task Timer with a distant deadline, an Expired Timer, and a Timer that becomes due while Ranch is closed. Record `started_at`, `duration_secs`, status, and work text/completion before restart. After restart, future Timers continue from the saved timestamps; Expired remains Expired; the due Timer expires through the existing worker. Start/duration and Markdown must not reset or change. A normal Running → Expired status write is permitted, not evidence of startup seeding. Repeat the launch to check no restart/reset.

Record source revision, platform, fixture case, launch/restart actions, actual UI/file outcomes, and pass/fail/unverified in `validation/us1.md` and `validation/pass-record.md`, linked to US1/AC4–AC5 and FR-001/FR-019. Link Timer proof to `validation/us2.md` / V1. Missing native execution leaves startup acceptance open. Storage unit tests alone do not establish actual host startup.

The implementation removes seed-preservation tests because their required behavior is superseded; retain the existing storage/Timer tests and this replacement acceptance matrix. Run fresh `rtk task check` and `rtk task check:windows` on synchronized corrected source. Do not use old v0.1.5 results to claim the new runtime passed. No new test harness or source-text assertion is planned.

### V1 — Files, card, and Timers

Rename a Focus. For duplication, give the source a description, ordered Tasks with mixed completion states, and both Focus and Task Timers. Duplicate and inspect both files and Animals: the independent copy gets fresh identity and creation time, retains description and Task text/order/completion, has title `<source title> copy`, and has no Focus or Task Timers; the source remains unchanged. Duplicate the original again while that title is occupied to check the numeric suffix starting at 2 for identity collisions (FR-002 / US2/AC6). Add/edit two Tasks, Complete/undo one, then Delete it. Inspect fixture `focuses/<id>/focus.md`: Complete checks a retained line, Delete removes it. Blank Task input must not create a Task.

Start/restart/Clear Focus and Task Timers and inspect their owning sidecars; Clear retains work. Check built-ins 10/30/60 and shared custom chips across pickers/restart. Remove saved chips during a running Timer: Timer and built-ins remain.

Use short custom Timers to observe Focus growth then expiry (ghostly/resting/away, tray Expired). Adding a Task revives that Focus. Task Timer expiry changes its status/allowed notification without changing Animal size/motion or tray Expired membership; adding a Task never revives another Task's Timer. In disposable fixture copies, missing/corrupt optional sidecars must still permit loading the Focus. Existing tests cover atomic-write failure cases.

### V2 — Hand edits and durability

Keep the corresponding Focus card already open and visible. Successfully save a valid local Focus file after appending a nonempty checkbox; then separately Complete a checkbox and repeat. For each change, measure from successful save completion to the corresponding Task row visibly matching the saved file. Record the timing method, both endpoints on a common time base, precision/uncertainty, and elapsed time. Each measured change must take **at most 1.0 second** (SC-003); a slower result fails that observation. If timing precision cannot establish the bound or the preconditions are unavailable, leave it unverified. Watcher debounce and source inspection are not end-to-end measurements. Restart and verify work, Timers, and saved Settings survive. Hand-edited Settings apply on restart, not by live Settings-file watching.

### V3 — Ambient overlay and displays

Name the open Focuses from visible labels. Confirm one Pig per Focus and check the motion matrix: absent or Running Focus Timer → Walking; Expired Focus Timer → Resting. Task Timer state must not control motion. Check drag/toss and Gather to primary. Click an Animal to open its Focus card; use card controls to verify they receive interaction. Click empty overlay space outside every Animal and the card: the underlying app receives the click and the card closes. Reopen and close with Escape. **Known source mismatch:** the current open-card whole-overlay hit region/backdrop consumes the outside click. Record the actual native result separately; do not mark simultaneous pass-through and closure passed merely because closure works. This guide does not authorize a runtime fix. Work in another app: Ranch must not raise a window or take attention on its own.

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

Record FR-013 positional drift, FR-014 transport/event gaps, and FR-015 retained legacy entries as source/fixture observations. Do not require reproducing them in personal data or fail future work because an approved feature fixes them. Also record whether the no-seeding correction is implemented and passes V0; automatic creation remains a Constitution I / FR-019 failure until corrected, while past seed tests remain historical evidence only. Explicitly report ADR-0002/current-code drift; [research](research.md) links follow-ons. Only the bounded startup correction is planned; FR-013–FR-015 and open-card click-through fixes are excluded.

## Completion evidence

Record commit, platform, commands/exit statuses, native observations, unverified checks, and linked discrepancies in later task/issue evidence. Map all six SCs using the contract. Source inspection and past CI are not fresh success. Retain existing task history, append unchecked startup correction/proof tasks after T029, and map US1/AC4–AC5 plus V0 before execution; do not turn a checked earlier evidence assessment into new native proof. Stop after baseline evidence is reconciled; the startup correction requires its fresh gates, review, and merge, with native acceptance left open until proved. Other runtime changes require separately defined scope.
