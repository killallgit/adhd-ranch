# Ranch Today Validation Pass

**Date**: 2026-10-17 | **State**: Platform package proof complete; v0.1.5 published; six native validation tasks remain open; no full native acceptance claimed.

Runtime source commit: `a48525d9a89555bf9bc925a20d626c0dc634b4ce` (local main equals origin/main after a successful fetch). Tests run in a temporary detached worktree at that commit. The working checkout contains only unrelated/documentation edits, preserved in place. The clarified 003 documents guide this pass but remain Draft.

## Initial scope and setup (T001–T002)

Reuse existing code and tests. No implementation, new tests, dependency upgrades, migration, generated-type edits, hook fixes, PR, or release. Fresh short temporary Ranch data and empty Claude configuration directories were created outside the repository. Temporary paths and raw logs are not committed. Native computer control is disabled in this session; browser control cannot operate the native Ranch windows, tray, OS click-through, or notifications. No Ranch instance was running at process inventory. No app was launched, plugin installed, personal configuration copied, credential borrowed, or transcript read.

T002 fixture setup: **PASS**. Native first-launch example/delete/restart observation: **UNVERIFIED** because native control is unavailable. Existing seed tests will provide separate automated evidence if the gate passes.

## Input revisions at start

| Artifact | SHA-256 |
|---|---|
| `specs/003-ranch-today/spec.md` | `bd59188787842ae9b95fd0e8b4a02acd128cbe2ca2c969ad60508ac6ad08b5cb` |
| `specs/003-ranch-today/plan.md` | `46df8f3cd6ec862e71897c4d46c005f836ffc90ea36c7ae04b90d584d1b92954` |
| `specs/003-ranch-today/tasks.md` | `a4b963f32e6de17cb2ad6e543ad5bb949df62cea105fce488a1a58e00a7a15fe` |
| `specs/003-ranch-today/research.md` | `4e508efbade9bb0e2e1da7262a9fd07adeabb7ad5f69037dd93b4a5c86e9fa57` |
| `specs/003-ranch-today/data-model.md` | `30c6242a0aed24df0d054f518baa025f22a54c2b33ab520c73039d9ccc201a46` |
| `specs/003-ranch-today/contracts/current-behavior.md` | `f72ca3581d92cb35d4d973c8dc0ccf178314a957a58313af9c6a704fe07b59bb` |
| `specs/003-ranch-today/quickstart.md` | `7b5efe4b9f55d6a415c053130f9273384241e89e34d3bd2ab72f521235cc756f` |
| `.specify/memory/constitution.md` | `c40cb396de6834eff9e65f8d94caf69e7644ae783841eeedb395760ae48cba69` |

The tasks hash identifies the pre-execution checklist; task markers change as evidence is recorded.

## Constitution check

| Principle | Result for this validation pass |
|---|---|
| I | Isolated synthetic data; no personal-data writes or new runtime behavior. |
| II | Focus and Agent evidence stay separate; their accepted renderer seam is retained. |
| III | No source/generated contract changes; existing pure-rule and boundary tests only. |
| IV | No implicit installation; configuration and delivery proof remain distinct. |
| V | Current-main gate passed; native gaps and failures remain visible; no shipping claim. |
| VI | Draft status retained; accepted ADR direction distinguished from current code and follow-on proposals. |

## Automated gate (T004)

**PASS** — `rtk task check`, exit 0, 133.93 seconds, on synchronized runtime commit `a48525d9a89555bf9bc925a20d626c0dc634b4ce` in the detached validation checkout. Normal Rust suites passed 335 tests; Vitest passed 170 tests across 26 files. Generated-export checks also passed. This ran Rustfmt, debug/release Clippy, Biome, unused dependency/code checks, IPC/CSS checks, TypeScript checking, Rust unit/integration tests, Vitest, and generated-type drift. The checkout remained clean afterward, including manifests, lockfiles, and generated types. No gates were suppressed. Raw logs were inspected outside the repository; summaries below retain fresh results without temporary paths.

- running 59 tests
- test result: ok. 59 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.20s
- running 2 tests
- test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
- running 112 tests
- test result: ok. 112 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
- running 96 tests
- test result: ok. 96 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.27s
- running 9 tests
- test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 5.03s
- running 4 tests
- test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
- running 53 tests
- test result: ok. 53 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.07s
- Test Files  26 passed (26)
- Tests  170 passed (170)
- Duration  2.40s (transform 1.06s, setup 1.37s, collect 2.76s, tests 4.84s, environment 8.54s, prepare 1.55s)
- running 136 tests
- test result: ok. 136 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.14s
- running 0 tests
- test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
- running 68 tests
- test result: ok. 68 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.24s
- running 2 tests
- test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
- running 0 tests
- test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

## Source reconciliation and setup verification (T003)

**PASS** for source freshness: the fetched main commit matches the planning runtime target. ADR-0001, ADR-0002, and ADR-0003 are Accepted. ADR-0003 refines the renderer placement from ADR-0001. ADR-0002's stamped frames/change events remain accepted direction, not completed implementation: see the known-observations section to be consolidated under T019. Draft specs 004–008 and local issue proposals do not add requirements to 003. No artifact or code change is required by this reconciliation.

Setup verification: Git repository confirmed; `.gitignore` covers 19 checked generated, dependency, environment, backup, editor, and temporary-path examples. No Docker, ESLint, Prettier, Terraform, or Helm configuration was detected at the root. The npm package is private, so no publish ignore is required. No ignore/configuration files changed.

## Toolchain

macOS 26.6.2 arm64; rustc 1.99.0, cargo 1.99.0, Node v24.21.0, npm 12.0.2, Task 3.53.1. These are the execution environment, not proposed project minimums. Runtime source/manifests/lockfiles in the working checkout have no edits; the isolated checkout tests their current-main versions. Build output, dependency installation, and generated-type checks stay in the temporary checkout/target directory.

## Known observations and follow-on routing (T019)

These are source observations, not failed preservation tests or approved alternatives to Accepted ADRs.

| Entry | Current evidence | Follow-on context |
|---|---|---|
| FR-013 | Task Timers are positional arrays; parser identity is `focus_id:index`; arbitrary external insertion/reordering has no stable binding repair. | [004](../../004-task-timer-hand-edits/spec.md), [056](../../../issues/056-task-timer-binding.md) remain draft/proposal context. |
| FR-014 | Client/server frame is verb plus payload; LiveSessions applies arrival order; Stop/StopFailure both map to Idle; host emits unit invalidations and the frontend refetches. ADR-0002 accepts stamped frames and change-carrying events, so its target/checklist cannot certify current implementation. | [005](../../005-late-hook-frame/spec.md), [059](../../../issues/059-self-describing-hook-frame.md); [006](../../006-unheard-turn-ends/spec.md), [060](../../../issues/060-session-staleness-sweep.md) for missed-event recovery; [061](../../../issues/061-hook-change-events.md) for change events. |
| FR-015 | Current plugin/client owner does not edit older direct entries; stable client sits directly under the Ranch root. No personal configuration was inspected to establish whether such entries exist on this machine. | [007](../../007-one-hook-delivery/spec.md), [062](../../../issues/062-stable-hook-client-path.md). |

Before follow-on implementation, refresh two stale issue contracts: 061 still names this observational baseline's FR-014 as its owning build specification; it needs an approved build contract aligned with ADR-0002. 062 describes the old direct-settings installer and a proposed `bin/` layout, whereas current code uses a Claude-managed plugin and a client directly under the Ranch data root. Its historical machine/log examples are not fresh evidence. These corrections are routed to the existing issues; neither their implementation nor personal hook cleanup is authorized by this pass.

## Boundaries, platform scope, and runtime gaps (T020)

**PASS, scoped source review:** inspected `src-tauri/src/ui_bridge/mod.rs`, host composition, Focus/Timer workflows, Agent adapters, manifests, and Tauri configuration. The exposed application boundary is typed Tauri IPC; user work writes route through explicit Focus operations, seed setup, filesystem hand-edits, and timer-state expiry. Agents do not own Focus writes, editing, or Limits. Species is Pig and the only Harness module is Claude Code. No sync/import/multi-user service, Wrangle action, automatic Task completion/Focus merging, or Harness notification forwarding was found in this scope.

A source search across Rust/TypeScript runtime files under `crates/`, `src/`, and `src-tauri/src/` for `fetch(`, XMLHttpRequest, WebSocket, reqwest, hyper, TCP listeners/streams, ureq, telemetry, Sentry, and provider API-key symbols returned no matches (rg exit 1, expected for no matches). This is bounded source evidence, not proof of every dependency's runtime behavior. Tauri's localhost Vite dev URL is development infrastructure; packaged assets use `frontendDist`. Explicit plugin install delegates marketplace operations to the local Claude CLI, whose external effects are separate from Ranch product runtime traffic.

**UNVERIFIED:** SC-006 packaged-process network observation was not run. No native app was launched or core scenarios exercised in a packaged process; availability of a network inspection utility alone is not runtime evidence. Native ambient/attention/notification checks are also unverified.

**Platform evidence:** current macOS host passed debug/release Clippy and host tests in T004; frontend production build passed. Actual macOS bundling/native behavior was not tested. Windows cross-check, Windows installers, and Linux packages were not run; this documentation-only pass changes no platform source and does not trigger release workflows. [Windows workflow](../../../.github/workflows/windows.yml) defines manual x64/ARM64 NSIS builds; [release matrix](../../../.github/workflows/release.yml) defines Linux AppImage/deb and other packages. Configuration is not a fresh package-build result. Windows hook delivery is unsupported by the Unix-gated implementation.

## Evidence index (T021)

Every tracked entry is assessed. UNVERIFIED means some required native/platform/runtime proof is missing even when its automated layers passed. PASS for negative capabilities below is explicitly scoped to the current source/architecture review, not a universal runtime experiment. No test in the initial automated gate failed. The later Linux package-build failure is recorded in the convergence execution below.

| Entry | Result | Supporting task/result |
|---|---|---|
| FR-001 | UNVERIFIED | T007–T008/T014–T015: [store/parser proof](us2.md), [watcher/sidecar proof](us4.md); native file/UI journey absent. |
| FR-002 | UNVERIFIED | T008: [Complete/Delete/Clear layers](us2.md) passed; native card-to-file flow absent. |
| FR-003 | UNVERIFIED | T009–T010: [Timer/chip/rule proof](us2.md) passed; native sharing/restart/expiry absent. |
| FR-004 | UNVERIFIED | T005–T006: [render/movement proof](us1.md) passed; native count, drag/toss/Gather/click-through absent. |
| FR-005 | UNVERIFIED | T008/T016/T018: [card](us2.md), [Agent selection](us5.md) tests passed; native selection/closure absent. |
| FR-006 | UNVERIFIED | T011–T013: [tray routes and Limits](us3.md); no actual tray/dialog/color check. |
| FR-007 | UNVERIFIED | T011/T013/T016/T018: [Limits](us3.md) and [Agent boundary](us5.md); live crossing/Agent comparison absent. |
| FR-008 | UNVERIFIED | T012/T015: [Settings workflow](us3.md), [restart distinction](us4.md); native windows/restart absent. |
| FR-009 | UNVERIFIED | T005–T006/T011: [geometry/defaults](us1.md); actual multi-display behavior absent. |
| FR-010 | UNVERIFIED | T016–T018: [Agent source/fixtures](us5.md) passed; native visibility/plugin-retention journey absent. |
| FR-011 | UNVERIFIED | T016–T018: [plugin/socket/journal proof](us5.md) passed; actual Agent Hooks window/live installation/delivery absent. |
| FR-012 | UNVERIFIED | T020: bounded source/IPC review passed; packaged network observation absent. |
| FR-013 | OBSERVATION | T014/T019: positional drift, [US4](us4.md), follow-on 004/056. |
| FR-014 | OBSERVATION | T016/T019: arrival-order/refetch drift, [US5](us5.md), follow-ons 005/059 and 061. |
| FR-015 | OBSERVATION | T016/T019: retained legacy entries/current stable plugin client, [US5](us5.md), follow-on 007/062. |
| FR-016 | PASS (package/source scope) | T029: v0.1.5 macOS universal, Windows x64/ARM64, and Linux x64 packages passed; Windows hook delivery remains unsupported by the reviewed source. See [release package proof](#release-package-proof-t029). Native behavior remains outside this package claim. |
| FR-017 | UNVERIFIED | T006/T020: [native attention scenario](us1.md) not performed. |
| FR-018 | PASS (source scope) | T020: no sync, multi-user, or external work-import capability found. |
| FR-019 | PASS (source scope) | T010/T018/T020: IPC/writer ownership reviewed; no network API, slash command, or integration work writer found. |
| FR-020 | PASS (source scope) | T010/T018/T020: expiry changes timer state; no automatic work completion/merge or Harness notification writer found. |
| FR-021 | UNVERIFIED | T005–T006/T011–T012: primary-only/no-Wrangle source confirmed; native Gather absent. |
| FR-022 | PASS (source scope) | T005/T016/T020: Pig-only Species and Claude-only Harness confirmed. |
| SC-001 | UNVERIFIED | T006: [native glance usefulness](us1.md) not observed. |
| SC-002 | UNVERIFIED | T008–T010: [automated layers](us2.md) passed; actual card-to-file journey absent. |
| SC-003 | UNVERIFIED | T015: [watcher test](us4.md) passed; save-to-visible-card timing not measured. |
| SC-004 | UNVERIFIED | T013/T018: [Limit rules](us3.md) passed; native notification/tray/Agent comparison absent. |
| SC-005 | UNVERIFIED | T017–T018: [controlled Agent proof](us5.md) passed; actual native visibility/non-editability journey absent. |
| SC-006 | UNVERIFIED | T020: no packaged-process network observation. |

No unmapped entries. Five scoped PASS entries (four source-only, plus FR-016 package/source proof), three observations, and 20 entries with remaining proof gaps; all six success criteria remain UNVERIFIED end-to-end. Source/runtime discrepancies are linked to existing follow-on issues above. Unavailable observations are a manual validation backlog in [quickstart](../quickstart.md), not newly demonstrated product defects.

## Completion limits

All 22 execution steps can be recorded as assessed under this task list's evidence semantics; checked boxes do not mean every product criterion passed. This pass changes only evidence documents and task progress. The specification remains Draft; Accepted ADRs remain authoritative. Product acceptance, independent review, CI on a documentation PR, and merge are not claimed.

## Final verification (T022)

**PASS** — evidence links/anchors, all 28 exact requirement-index rows, task IDs/labels, task/result consistency, whitespace, and preservation checks completed. `rtk git diff --check` exited 0. Only the existing tasks file changed during execution; all other 22 pre-existing edited/untracked files, including the clarified specification and plan artifacts, retained their starting hashes. Six evidence notes were added. Runtime source, lockfiles, configuration, and generated contracts remained unchanged.

All 22 task evidence steps are recorded. The final Constitution I–VI check retains the controls in the table above: local synthetic data only, independent domains, unchanged typed boundaries, no integration install, current-main automated proof, explicit native gaps, and unchanged Draft/Accepted authority. No exception or product conformance waiver is granted. Four negative-capability entries passed within source-review scope; three limitations remain observations; 21 entries, including all six SCs, retain required proof gaps.

The clean temporary worktree, build output, and synthetic data/configuration directories were removed after recording results; no branch was created. No review, PR, CI run, merge, or release was triggered. The complete runtime gate was not repeated after evidence-only edits. Next validation is the native/platform/process work explicitly marked UNVERIFIED in this record and the story notes.


## Convergence execution — 2026-10-17 (T023–T029)

This follow-up supersedes the initial capability assessment where explicitly stated below. Runtime source is still `a48525d9a89555bf9bc925a20d626c0dc634b4ce`, verified against remote main before these builds. A fresh detached checkout, synthetic Ranch data, and empty Claude configuration were used. No runtime source, dependency, lockfile, generated contract, or acceptance requirement was changed. The still-current T004 gate was reused.

### macOS package and partial native evidence (T023, T027)

**PASS — package build:** `rtk task build`, exit 0, 124.86 seconds, macOS 26.6.2 arm64. Produced `Adhd Ranch.app` and `Adhd Ranch_0.0.0_aarch64.dmg` from the assessed commit. DMG SHA-256: `1407f73ecad6c64c447a35c2bb6c76f32b07427779076a94cf0791f553894800` (4,186,303 bytes). This establishes bundling, not installer execution or full native acceptance.

**Observed native subset:** the packaged app launched with the isolated environment. Its first-launch Welcome Focus appeared as a Pig over a controlled synthetic TextEdit document. The tray exposed Focus deletion, and the native confirmation dialog appeared. Confirming deletion removed that Focus's canonical Markdown; the seed marker remained. Quit through the app's menu ended the packaged process. Restart/non-reappearance and the remaining V3 actions have not been verified. Coordinate-based cancellation attempts did not establish successful cancellation and are excluded from passing evidence.

CLI Accessibility automation was usable even though the browser tool exposes no native control. During capture, macOS displayed an additional Codex screen/audio capture permission dialog. Validation paused without approving that permission automatically; user approval was requested. Existing captures/raw logs remain outside the repository and contain only controlled validation content intended for this pass. No further capture or dependent native scenario was attempted while approval was pending.

**T023 remains UNVERIFIED/open.** See [US1 follow-up](us1.md#convergence-follow-up-t023). **T027 remains UNVERIFIED/open:** a matching package exists, but process-scoped network observation while exercising core scenarios has not been performed. No native FR or SC is promoted by this partial result. T024–T026 and T028 also remain open; a native launch alone does not prove their journeys. No Claude plugin was installed, credential borrowed, or personal transcript read.

### Windows packages (T029, Windows portion)

**PASS:** manually dispatched the existing non-publishing workflow with `rtk gh workflow run windows.yml --repo killallgit/adhd-ranch --ref main`. [Run 39070782884](https://github.com/killallgit/adhd-ranch/actions/runs/39070782884) completed successfully with head SHA matching the assessed runtime commit. Both Windows x64 NSIS and Windows ARM64 NSIS jobs passed. The workflow's unchanged build commands produced these downloaded installers:

| Target | Installer | Bytes | SHA-256 |
|---|---|---|---|
| x86_64-pc-windows-msvc | `Adhd Ranch_0.0.0_x64-setup.exe` | 2,844,689 | `f33663980596696fee11d79a26c3593bdee95c99dc14ae94f69958b2404a6070` |
| aarch64-pc-windows-msvc | `Adhd Ranch_0.0.0_arm64-setup.exe` | 2,577,237 | `505b2f69e8f50b5f2a333a03f2a15ac75177fe16bdbff86b43a4a2003ddb9461` |

The run's artifact retention is one day; binaries are not committed. These are fresh package-build results, not native Windows behavior or supported Windows hook delivery. No release workflow or publishing action was triggered.

### Linux packages (T029, Linux portion)

A disposable Ubuntu 22.04 amd64 container is running on the available arm64 Docker host with emulation. It uses the existing release matrix's system prerequisites and `npm run tauri -- build --bundles appimage,deb`, with `APPIMAGE_EXTRACT_AND_RUN=1`. Rust 1.99.0 and Node 24.21.0 match the macOS validation toolchain. Source was transferred as a Git archive of the exact assessed commit.

The first attempt stopped before compilation: the Docker VM could not see the host bind-mounted source directory, leaving the container workspace empty; `npm ci` exited 1 because no lockfile was present. This was a validation setup failure, not a demonstrated product/lockfile defect. Transferring the archive with `docker cp` corrected the setup; locked dependency installation then succeeded. **FAILED — package attempt:** the corrected run exited 1 after 893.72 seconds. Frontend production bundling succeeded. Rust compilation stopped while linking the `tauri-plugin-fs v2.5.1` dependency build script:

```text
error: linking with `cc` failed: signal: 11 (SIGSEGV) (core dumped)
error: could not compile `tauri-plugin-fs` (build script) due to 1 previous error
```

The failed invocation used Rust's bundled LLD linker (`-fuse-ld=lld`). The failure is localized to a toolchain process crash in the emulated environment; emulation/toolchain causation is suspected but not proven. It is not a demonstrated application source defect. No workaround, linker override, dependency edit, or relaxed gate was introduced. There is no successful `.deb` or AppImage artifact from this attempt. Execution stopped at this failure, as required by the implementation workflow.

**T029 remains unchecked.** Its Windows portion passed, but Linux package compatibility remains unverified. The next proof needs a successful matching package build in a suitable Linux x64 environment, or a separately diagnosed correction to this disposable environment. Windows native behavior and Windows hook delivery are not established by the passing installers.


### Convergence checkpoint

T023–T029 remain unchecked: no task's complete acceptance proof was obtained. All six SCs remain unverified end to end. Fresh macOS and Windows package results and the partial seeded-Focus native observation are retained without promoting the remaining FR/SC claims. The failed Linux attempt remains visible. Further native visual work requires the pending macOS capture-permission decision; live Agent proof also requires an independently usable disposable Claude configuration.

Evidence-only verification: relative links and task markers checked; 22 earlier assessment tasks checked and all seven convergence tasks open. All 27 other files snapshotted at this attempt's start retained their hashes. Runtime source, manifests, lockfiles, and generated contracts remain unchanged; Linux manifests/lockfiles also matched the assessed source after dependency installation. The existing T004 gate was not repeated. No `.specify/extensions.yml` exists, so no post-implementation extension hooks apply. No new test, product fix, pipeline, PR, merge, or release was created.

Cleanup completed: removed the owned Linux container, clean detached validation worktree, and macOS compile cache. Retained the macOS app/DMG, Windows installers, synthetic fixture, exact source archive, and raw build logs outside the repository. The Ranch process and synthetic underlay window are closed. `rtk git diff --check` exited 0; evidence links and anchors passed validation.


## Owner direction — ship before later native testing (2026-10-17)

The owner explicitly authorized continuing through PR, independent review, merge, and release, with manual testing deferred until afterward. This changes execution order, not the acceptance results above. T023–T028 remain open for native/live verification, and T029 remains open until matching Windows and Linux packages succeed. The failed emulated Linux attempt remains historical evidence; the existing release workflow will supply a fresh native Linux x64 build without introducing a new pipeline.

003 describes the existing product and adds no runtime implementation. Its known limitations remain observations with separately scoped follow-ons; this release does not implement the Draft 004–008 proposals or claim full 003 acceptance. The specification remains Draft. The shipping change contains only 003 planning/evidence documents and the release changelog; all unrelated dirty files are preserved outside its isolated branch.

Constitution I–IV and VI retain the prior controls: unchanged local-data and integration contracts, independent domains, generated boundaries, explicit installation, and unchanged authority. For V, documentation delivery will require independent review, a green automated gate/PR CI, and merge. The owner has deferred product-level native acceptance; those criteria remain unverified rather than waived or marked complete. Release packaging must still succeed on every configured target before the existing workflow publishes.


## Release package proof (T029)

**PASS — T029 complete.** [v0.1.5](https://github.com/killallgit/adhd-ranch/releases/tag/v0.1.5) was published at `2026-10-17T22:20:57Z` from `a6029d6ce2ac9c0e635975f252b0b8a0233e074b`. [Release run 39074102546](https://github.com/killallgit/adhd-ranch/actions/runs/39074102546) completed successfully: macOS universal DMG, Windows x64 NSIS, Windows ARM64 NSIS, and native Ubuntu 22.04 x64 AppImage/deb jobs all passed, including provenance attestations. Checksum generation, publication, and the existing release-asset retention job also passed.

The unchanged release workflow runs `npm ci --no-audit --no-fund` and the existing Tauri package commands with a generated `release-version.json` containing `0.1.5`: macOS uses `--target universal-apple-darwin --bundles dmg`; Windows uses the x64/ARM64 MSVC targets with `--bundles nsis`; Linux uses `--bundles appimage,deb`. These are package builds, not compile-only evidence. The earlier matching non-publishing Windows smoke builds are retained above. No new pipeline, linker override, dependency change, or relaxed gate was introduced.

The release source differs from the original assessed runtime `a48525d9a89555bf9bc925a20d626c0dc634b4ce` only in baseline documentation and changelog. `git diff --exit-code` across `crates/`, `src/`, `src-tauri/`, manifests/lockfiles, `Taskfile.yaml`, and `.github/workflows/` passed. The version override changes package metadata; assessed runtime/build source is unchanged. The native Linux success supersedes the missing package proof, without erasing the failed emulated attempt or claiming its crash cause is proven.

Downloaded all five packages and `SHA256SUMS.txt` with `gh release download v0.1.5`. Python SHA-256 verification matched every package, and the manifest covered exactly the five downloaded packages:

| Package | Bytes | SHA-256 |
|---|---|---|
| `Adhd.Ranch_0.1.5_darwin_universal.dmg` | 8,359,578 | `96c6733e92333b8da1f72a84bb10fddf195d01317ceaea524b8720894c8ae413` |
| `Adhd.Ranch_0.1.5_linux_amd64.AppImage` | 84,457,976 | `b85edd2f85f73354f0f55d7bf288e803c413cf993291540b49b34a3228c576ad` |
| `Adhd.Ranch_0.1.5_linux_amd64.deb` | 6,447,450 | `4591f12d2189a3a97fd751bd865ad41bdadef7d73472fb822f7131229d3a0110` |
| `Adhd.Ranch_0.1.5_windows_arm64-setup.exe` | 2,574,296 | `8cb05e9024764b916aba6a4f3d589eb0e51c7743cfd37cd74e82d57c87dfd9e7` |
| `Adhd.Ranch_0.1.5_windows_x64-setup.exe` | 2,847,131 | `49e9f50f196a8e085934e767ac22af5fd2077e7efef5d97b84ea76ecac006b4b` |

Delivery: [PR #92](https://github.com/killallgit/adhd-ranch/pull/92) merged after independent review, correction of CodeRabbit's Linux-proof finding, and [current-head CI](https://github.com/killallgit/adhd-ranch/actions/runs/39073736207). Fresh local `rtk task check` passed (exit 0, 136.05 seconds), and [merged-main CI](https://github.com/killallgit/adhd-ranch/actions/runs/39073944179) passed before release dispatch. This proof update changes documentation only and reuses those unchanged-runtime results.

T023–T028 remain open for the owner's deferred native/live testing. All six SCs remain unverified end to end; package builds do not establish UI behavior, notifications, live Claude delivery, or runtime network behavior. The specification remains Draft and FR-013–FR-015 remain known observations. No product acceptance or ADR-conformance waiver is claimed.


## Startup correction execution — 2026-10-18 (T030–T035)

**Scope:** owner-selected no startup seeding, resumption from saved work/Settings, and empty start without work. Native testing remains explicitly deferred. The owner approved proceeding past the reviewer-owned `checklists/baseline.md` gate (0/30 checked); its markers remain unchanged. This does not waive native acceptance or mark requirements review complete.

**T030 — PASS, preparation:** fetched origin successfully; local main and origin/main both identified `7900cc445ac224464e254b31d4914b0a52055270`. Created isolated branch `fix/resume-saved-work` from origin/main and carried only current 003 artifacts; unrelated main-checkout edits remain in place. Created `issues/063-resume-saved-work.md` with the narrow completion promise, scope and acceptance criteria. No app launch, personal fixture/configuration copy, plugin installation, migration, or lifecycle-status promotion occurred.

Input SHA-256 values below identify the clarified pre-implementation artifact versions; later task markers/results are execution updates.

| Artifact | SHA-256 |
|---|---|
| `specs/003-ranch-today/spec.md` | `3394925e405a83374a6938256349b4de1f8b39a31204f80a9c1c600d5a527fd8` |
| `specs/003-ranch-today/plan.md` | `d29f546e9af6f39fa286c815e29b061a70ba86a8ca08eb92c73dcd542b1a6960` |
| `specs/003-ranch-today/tasks.md` | `b409aec098728e93abe7782a1f1d7913e51d012f7614d8529a1471c392f64723` |
| `specs/003-ranch-today/research.md` | `ee2fa20e592ca3b893dad8080f3dc3bf30cbe8b22b89ee5e0f1bd648e2290939` |
| `specs/003-ranch-today/data-model.md` | `e87e80bd60e1ab63b7efcb7574fa9ac25ae7f583436099e2feadd019bfc0114e` |
| `specs/003-ranch-today/contracts/current-behavior.md` | `95ebf6cd09494ef11fde3fc7d4a2fc908f61896fa0700d73d0928e39d8aebfef` |
| `specs/003-ranch-today/quickstart.md` | `c22ce702fad5bfd71ca7c70e0e6d35a46a0f1a4cd5d947302e49fffcde625a3d` |

**Setup verification:** existing `.gitignore` covers Rust/Node output, generated sidecars, logs, environment files, editor/OS files and temporary files. No Docker, ESLint, Prettier, Terraform or Helm setup applies. The npm package is private, so no publishing ignore is needed. No ignore/configuration changes are required; worktrees use the local Git exclude. Existing files, schemas, manifests and generated contracts are unchanged by preparation.

**Constitution:** the startup deletion is the selected remedy for I, not an exception or a claim of current runtime compliance. II–IV retain existing boundaries. V requires fresh corrected-source gates and actual V0 proof; old v0.1.5 results do not establish the new runtime. VI retains Draft status and historical evidence. T034/T035 remain open while required native/delivery evidence is absent.

**T031 — PASS, implementation:** removed the sole startup seed call and module declaration, then deleted `src-tauri/src/app/seed.rs`. Its two tests asserted automatic creation and its marker-based persistence, behavior now explicitly superseded by the owner. No replacement function, writer, flag, migration, cleanup, or test suppression was added. Existing storage, Settings, Timer start/expiry, IPC, renderer and Agent code are unchanged. No saved work or marker file was inspected or modified.

**T032 — PASS, architecture:** replaced the obsolete first-launch seed statement in `docs/architecture.md` with saved-state loading, empty startup, preservation of existing examples/markers, and Timer continuity with normal expiry. This documents the implemented deletion, not native acceptance.

**T033 — PASS, fresh corrected-source gates:** `rtk task check`, exit 0, 114.68 seconds; `rtk task check:windows`, exit 0, 38.03 seconds, on macOS arm64 and the synchronized startup branch based on `7900cc445ac224464e254b31d4914b0a52055270`. A second fetch confirmed origin/main unchanged after the gates, so no rebase or redundant rerun was needed. Normal Rust suites passed 333 tests (the prior 335 minus the two deliberately retired seed-behavior tests); Vitest passed 170 tests across 26 files. Generated-export/drift checks passed separately. Host debug/release linting, frontend build/lint/typecheck, unused-code checks, IPC/CSS checks, and Windows x64 cross-target compilation passed. No gate was weakened, suppressed, or narrowed.

Windows emitted the existing unused `script` warning at `src-tauri/src/app/claude_plugin/contract.rs:84` and the cross-check's intentional placeholder-sidecar notice. `git diff --exit-code origin/main -- src-tauri/src/app/claude_plugin/contract.rs` confirmed the warning's owner is unchanged by this correction. Windows cross-compilation is not installer/package or native Windows proof.

Runtime source identity for these results: base commit above plus deletion of `src-tauri/src/app/seed.rs` and exactly two removed lines in `src-tauri/src/app/mod.rs`. No other runtime source changed. The resulting `mod.rs` SHA-256 is `534b045b7b8641750170d3e0db820b8f610a617702b1f62d9c26e6c31e82618a`. Manifests, lockfiles, and generated contracts matched origin/main after both gates. The existing storage/Timer tests retain their assertions; their passing results are component evidence, not execution of actual native startup.

**Independent review — PASS:** a fresh reviewer inspected the corrected source, caller search, preserved readers/expiry, issue contract, and evidence scope. No actionable correctness/security/architecture findings. Review did not run builds or claim native proof. Subsequent edits update evidence, status wording and the historical link for the deleted seed file only; runtime code is unchanged.

**Completion boundary:** T030–T033 are complete. T023–T028, T034 V0 native launch/restart, and T035 final native/delivery reconciliation remain unchecked. No app launch, screen capture, user-data/legacy-marker access, plugin installation, PR, merge, new package build or release occurred in this implementation pass. No native FR/SC or full issue completion is promoted; issue 063 stays open. The reviewer-owned custom checklist remains 0/30 checked under the owner's explicit proceed override.
