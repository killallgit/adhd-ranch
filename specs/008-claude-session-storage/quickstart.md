# Quickstart Validation: Claude Session Archive

This guide validates the implemented feature without reading or modifying the user's real Claude
or Ranch data. Run commands from the repository root.

## Prerequisites

- Rust 1.90 toolchain and frontend dependencies required by the repository.
- `task`, `cargo-shear`, Node/npm, and the Windows cross-check prerequisites documented by
  `Taskfile.yaml`.
- Synthetic fixtures under `crates/storage/tests/fixtures/claude/`; do not use a personal
  `~/.claude/projects` transcript in automated tests or commits.

## 1. Prove the database dependency and schema

```sh
cargo check -p adhd-ranch-storage --all-targets
cargo test -p adhd-ranch-storage claude_archive -- --nocapture
```

Expected:

- local-only Turso compiles with defaults and the `sync` feature disabled and no HTTP/TLS runtime
  client in the resolved graph;
- a temporary `ranch.db` opens, migrates to `PRAGMA user_version = 1`, closes, and reopens;
- the four tables and indexes in [ranch-db-v1.sql](contracts/ranch-db-v1.sql) match the runtime
  migration;
- `.tables` contains only the four Claude-specific archive tables—no graph, node/edge, or generic
  cross-tool tables—and `ranch.db` is the only database;
- a cleanly closed/checkpointed Turso file reopens through the dev-only SQLite reader, passes an
  integrity check, and returns the seeded raw row;
- every writer connection reports foreign keys enabled, and an injected mid-migration failure
  leaves no partial schema before a successful retry;
- no test needs Turso credentials or a network service.

## 2. Validate default data-root migration

```sh
cargo test -p adhd-ranch data_root -- --nocapture
cargo test -p adhd-ranch-storage data_root_migration -- --nocapture
```

Expected state matrix:

| Initial state | Outcome |
|---------------|---------|
| legacy only | complete directory renamed to new root; contents byte-identical |
| new only | new root used unchanged |
| neither | new root created |
| both | conflict returned; neither root changed |
| explicit `ADHD_RANCH_HOME` | exact override used; no default migration |

Also confirm the plugin contract test expects `.adhd/adhd-ranch-hook` and
`.adhd/agent-hooks.sock` while preserving the explicit override path. Start with a synthetic installed
legacy wrapper: startup reports it unhealthy without rewriting it; explicit Install/Repair replaces
the managed wrapper, and a test firing reaches only the new-root socket.

## 3. Validate raw Hook Receipt persistence

```sh
cargo test -p adhd-ranch-storage hook_receipt -- --nocapture
cargo test -p adhd-ranch-storage --test agent_hook_socket -- --nocapture
```

Expected:

- archive disabled: no database receipt, existing live-session behavior unchanged;
- archive enabled: exact payload bytes and SHA-256 digest match the fixture;
- a first receipt upserts its session before the immediate foreign-key insert;
- duplicate deliveries create separate receipts with the same digest;
- queue saturation rejects admission, increments only the content-free dropped-job counter, and
  does not block the socket client;
- accepted-job/database/transcript failures increment their current-run error categories without
  incrementing the dropped-admission count;
- disabling archiving at a controlled queue barrier rejects every later firing while all earlier
  accepted jobs finish exactly once with no partial transaction;
- graceful shutdown at a controlled queue barrier closes admission and persists every accepted
  queued job exactly once before worker exit; reopen shows no partial transactions;
- invalid JSON is retained with a parse error and no raw content is logged;
- an unavailable database does not change the hook client's silent failure/timeout contract.

Also run the plugin contract tests and verify exact lockstep version `0.2.0`, six events including
`SubagentStop -> observe`, `.adhd` wrapper paths, rejection of every mismatched contract, and
explicit Install/Repair replacement. There is no compatibility adapter or implicit startup repair.

## 4. Validate transcript mapping and cursor recovery

```sh
cargo test -p adhd-ranch-storage transcript -- --nocapture
cargo test -p adhd-ranch-storage transcript_generation -- --nocapture
```

Expected:

- every completed fixture line is byte-identical after concatenating `raw_json` and its preserved
  `record_delimiter`, including LF and CRLF;
- main `transcript_path` and `SubagentStop.agent_transcript_path` fixtures both ingest, while the
  archive-only subagent verb leaves parent live Activity unchanged;
- known envelope fields are indexed and unknown fields remain in the raw record;
- the unknown `cost-state` fixture persists without a closed-enum failure;
- a malformed complete line persists with `parse_error`;
- every unterminated tail waits, including a valid JSON value observed unchanged twice; a later
  newline produces exactly one correctly framed row and no spurious empty row;
- a row appended after the terminal hook is captured by bounded settle retries without another
  firing, and retry tests use an injected/paused clock;
- three replays, including one database reopen, add no duplicate transcript records;
- file truncation/replacement supersedes generation 1 and creates generation 2 without altering
  generation 1;
- default, inherited-env, and explicit `claude.config_dir` roots resolve in order; paths outside the
  canonical selected projects fixture are rejected without being read.

## 5. Validate live/archive separation

```sh
cargo test -p adhd-ranch-storage live_sessions -- --nocapture
cargo test -p adhd-ranch-commands agent_sessions -- --nocapture
cargo test -p adhd-ranch-commands render_scene -- --nocapture
```

Expected:

- archived rows alone yield no live Agent Sessions after restart;
- a new Hook Firing still updates `LiveSessions` and the render scene as before;
- disabling Agent rendering does not alter archive history;
- disabling Claude archive drains accepted work, stops new receipts, and does not delete existing
  rows;
- `observe` never creates, updates, or ends a live Agent Session, including when archiving is on and
  Agents as Animals is off.

## 6. Validate aggregate archive health and Developer Mode

```sh
cargo test -p adhd-ranch-commands archive_health -- --nocapture
cargo test -p adhd-ranch menu -- --nocapture
npm test -- --run ArchiveHealthWindow
```

Expected:

- Developer Mode exists in development and release configurations, begins off, is never persisted,
  and the Archive Health item is absent until the menu toggle enables it;
- disabling Developer Mode removes the item and closes the window; restart returns it to off;
- ready/empty/unavailable states follow [archive-health.md](contracts/archive-health.md);
- schema version, combined DB/WAL/SHM byte size, four row counts, and last successful commit match
  direct fixture observations;
- dropped-job and bounded error-category counts are labeled `this run`, are not double-counted, and
  reset after restart while persisted aggregates remain;
- generated command types and rendered DOM contain no identifiers, paths, offsets, digests, raw
  rows, prompts, messages, tool data, or source content;
- polling/opening the window neither changes counters nor delays archive ingestion.

## 7. Run repository gates

```sh
task check
task check:windows
```

Expected: lint, type checking, Rust/web tests, generated-type drift checks, release-profile clippy,
and the Windows workspace check all pass. CI must also pass its packaged Linux build/compile lane.

Also run the archive integration suite in its network-denied/instrumented harness and verify zero
outbound connection attempts. On supported platforms, assert user-only modes for `ranch.db` and any
WAL/SHM sidecars after create, write, close, and reopen.

## 8. Manual local smoke test

Use separate temporary Ranch and Claude roots. Start Ranch from terminal A with both exported:

```sh
export ADHD_RANCH_HOME="$(mktemp -d)"
export CLAUDE_CONFIG_DIR="$(mktemp -d)"
printf 'Ranch root: %s\nClaude root: %s\n' "$ADHD_RANCH_HOME" "$CLAUDE_CONFIG_DIR"
task dev
```

In the app:

1. Confirm Claude archiving is off by default.
2. Read the plaintext-data disclosure and enable archiving explicitly.
3. Install/Repair the Ranch hooks only into this temporary `CLAUDE_CONFIG_DIR`. In terminal B,
   export both variables to the exact paths printed by terminal A, then start `claude` there with
   a synthetic prompt. Never copy personal settings, credentials, or transcripts into the fixture.
4. Confirm `$ADHD_RANCH_HOME/ranch.db` exists and the Agent appears only while the live hook state
   says it is present.
5. Quit and reopen Ranch with both variables unchanged. Confirm archive rows remain but no historical
   Agent appears until another live Hook Firing arrives.
6. Disable archiving and confirm a subsequent Hook Firing creates no new archive receipt.
7. Enable Developer Mode from the app menu, open Claude Archive Health, and confirm only aggregates
   appear. Disable Developer Mode and confirm the item/window disappears.
8. Restart and confirm Developer Mode and all `this run` counters are off/zero while database counts
   and last successful ingest remain.

Remove both temporary directories after Ranch, the disposable Claude session, and any database
shell are closed. Do not point this smoke test at personal Claude transcripts when capturing logs
or test artifacts.
