# US1 — Ambient Focus Animals

Runtime commit and fresh gate: [pass record](pass-record.md#automated-gate-t004). All automated results below come from the single successful `rtk task check` run (exit 0); no additional suite was run.

| Task | Evidence result | Acceptance limit |
|---|---|---|
| T005 | PASS — source and existing automated contracts reviewed | Does not establish native usability |
| T006 | UNVERIFIED — V3 assessed; native control unavailable | SC-001 and native overlay/display behavior remain open |

[Focus projection](../../../crates/commands/src/focus_animals.rs) creates one Animal from each Focus, uses its title as label, emits Pig, and derives size/Motion from the Focus Timer only. [Scene composition](../../../crates/commands/src/render_scene.rs) combines independent projections into the flat contract. Species is Pig; Motion Walking/Resting; size fixed/linear; `regionId` optional. [AnimalSprite](../../../src/components/AnimalSprite.tsx) and [movement](../../../src/lib/animalMovement.ts) consume renderer vocabulary, not Focus/Task/Timer/Agent types.

Fresh existing proof includes `projects_source_identity_label_species_and_whole_display`, `projects_running_and_expired_timer_curves_and_motion`, `task_timers_do_not_affect_focus_animals`, and [exact wire serialization](../../../crates/commands/tests/render_scene_contract.rs). [App](../../../src/components/App.test.tsx), [movement](../../../src/lib/animalMovement.test.ts), and [movement-hook](../../../src/hooks/useAnimalMovement.test.ts) suites passed with mixed scenes, selection/hit rectangles, drag/toss, region confinement, and Gather mechanics.

V3 native items not performed: naming two actual on-screen Focuses from labels, visible Pig count, drag/toss/Gather to primary, primary-only startup and enabling another display, gap traversal, OS click-through, and attention behavior while working elsewhere. Component/geometry tests are separate evidence, not substitutes. No baseline acceptance claim is made for these items.


## Convergence follow-up (T023)

On 2026-10-17, the macOS arm64 package from the same runtime commit built successfully and launched with fresh synthetic Ranch/Claude roots. The first-launch Welcome Focus was visible as a Pig. A native deletion confirmation appeared; accepting it removed the seeded Focus Markdown while retaining the seed marker. Quit through the app menu ended the process. Cancellation is not verified: earlier coordinate attempts did not establish that Cancel was activated.

This is partial native evidence only. A subsequent macOS Codex capture-permission dialog paused visual validation pending user approval. Seed non-reappearance after restart, two named Focus labels/count, drag/toss/Gather, enabled-display confinement, OS click-through, and sustained ambient attention remain unverified. T023 remains unchecked, and SC-001/FR-004/FR-009/FR-017/FR-021 remain open. Package identity and the permission limitation are recorded in the [follow-up pass record](pass-record.md#macos-package-and-partial-native-evidence-t023-t027).
