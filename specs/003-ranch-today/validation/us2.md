# US2 — Focus Card, Tasks, and Timers

Runtime commit and fresh gate: [pass record](pass-record.md#automated-gate-t004). Automated proof is the existing successful `rtk task check` run, exit 0.

| Task | Evidence result | Native acceptance |
|---|---|---|
| T007 | PASS — source/model/coverage review | No native action claimed |
| T008 | UNVERIFIED native V1 card/CRUD; automated layers PASS | Actual card-to-file journey and closure remain open |
| T009 | UNVERIFIED native picker/restart journey; automated layers PASS | Shared-window chips and running Timer preservation need native observation |
| T010 | UNVERIFIED native expiry/revival journey; automated rules PASS | Visible ghosting/away/resting, real notifications, and tray appearance remain open |

[Focus model](../../../crates/domain/src/focus.rs) rejects blank Task text and permits optional Timers. [Parser](../../../crates/domain/src/parse.rs) requires frontmatter `id`/`title`, reads nonempty checkboxes, and derives `focus_id:index`; that identity remains an observation, not a preservation rule. [Focus workflows](../../../crates/commands/src/focus.rs) route explicit mutations; append invokes Focus revival. [Timer workflows](../../../crates/commands/src/timers.rs) address exactly one `TimerOwner`, start/restart Running, Clear the Timer only, and expire with timer-only persistence plus settings-gated notifications.

Fresh [store tests](../../../crates/storage/src/focus_store.rs) cover append, indexed Delete, Complete/undo, text edits preserving completion, rename, missing/corrupt sidecars, and atomic persistence paths. Command tests cover duplicate and invalid input. Existing Timer tests `start_writes_a_running_timer_for_the_preset_duration`, `start_takes_the_same_path_for_a_task`, `clear_removes_the_timer`, `revive_clears_an_expired_focus_timer`, `expire_due_persists_expired_status_for_a_task`, and notification dedup/gating passed. Task Timers do not drive Focus projection.

[Settings normalization](../../../crates/domain/src/settings.rs) preserves positive unique custom minutes excluding built-ins 10/30/60. [Picker helper](../../../src/lib/timerPreset.ts) defines the current menu. [Settings workflow](../../../src-tauri/src/app/settings_workflow.rs) edits saved chips separately, preserves them against stale Preferences writes, persists before applying, and never clears work Timers merely because a chip was removed. Existing stale-save/normalization tests and [picker](../../../src/components/TimerPresetPicker.test.tsx)/[detail](../../../src/components/AnimalDetail.test.tsx) suites passed.

SC-002 remains UNVERIFIED end-to-end: no native rename/duplicate/delete, Task editing, sidecar comparison after UI actions, outside-click/Escape, chip sharing across actual windows/restart, or timed native expiry was performed. Automated callback/storage/rule proof must not be reported as that journey.
