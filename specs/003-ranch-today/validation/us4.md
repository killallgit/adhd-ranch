# US4 — Hand-Edited Markdown

Fresh automated gate and runtime commit: [pass record](pass-record.md#automated-gate-t004), `rtk task check`, exit 0.

| Task | Result | Limit |
|---|---|---|
| T014 | PASS — parser, storage, watcher, paths and tests reviewed | Positional identity recorded as observation |
| T015 | Automated adapter/storage checks PASS; native V1/V2 UNVERIFIED | No save-to-visible-card timing or restart journey measured |

[Parser](../../../crates/domain/src/parse.rs) requires frontmatter id/title, ignores empty checkbox text, maps checked state, and derives `focus_id:index`. [Store](../../../crates/storage/src/focus_store.rs) handles optional sidecars: corrupt/missing Focus Timer loads as absent; missing/malformed Task Timer JSON yields no timers. Task Timers remain indexed by checkbox position. App-side indexed deletion shifts that slot; arbitrary external reorder/insert has no stable identity repair (FR-013).

[Watcher](../../../crates/storage/src/watcher.rs) recursively observes Focus files; [host composition](../../../src-tauri/src/app/mod.rs) uses a 200 ms debounce and emits invalidations. That value is not SC-003's full editor-save-to-UI latency. Settings are loaded through the startup/settings paths, separate from the Focus-directory watcher. [Paths](../../../src-tauri/src/app/paths.rs) uses the explicit Ranch override, otherwise Unix home or Windows data-directory fallback.

Fresh existing proof includes `fires_callback_when_a_file_changes_under_root`, `reading_a_timer_without_a_sidecar_is_none`, `reading_a_corrupt_timer_sidecar_is_none`, positional/deletion Task Timer tests, and Markdown parsing/editing/storage tests. [Historical seed tests](https://github.com/killallgit/adhd-ranch/blob/a48525d9a89555bf9bc925a20d626c0dc634b4ce/src-tauri/src/app/seed.rs) `creates_example_focus_once_when_store_is_empty` and `does_not_recreate_example_after_user_clears_all_focuses` passed under T004; they are automated proof, not a first-launch native observation.

V2 not performed: external checkbox add/Complete reflected in an actual card, approximately one-second measured visibility, work/Timer/Settings durability across native restart, Settings hand-edit applying on next launch. V1 optional-sidecar failures are verified in temporary test fixtures, but not in a running native app. SC-003 therefore remains UNVERIFIED. No schema, stable Task identity, reorder repair, or migration was added.
