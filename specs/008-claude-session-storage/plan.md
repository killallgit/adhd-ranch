# Implementation Plan: Claude Session Archive

**Branch**: `008-claude-session-storage` | **Date**: 2026-09-28 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `specs/008-claude-session-storage/spec.md`

## Summary

Introduce the first Ranch database as a local-only Turso file at `~/.adhd/ranch.db`, with an
atomic migration of the complete legacy `.adhd-ranch` root before any application store opens.
Add a Claude-specific archive worker behind the existing hook boundary: it stores every opted-in
raw Hook Receipt, incrementally tails hook-provided main and subagent transcript paths, preserves
every complete JSONL record byte-for-byte, and extracts a small rebuildable index. Ranch owns this
tolerant mapping rather than adopting a closed third-party transcript model. The in-memory
`LiveSessions` reducer remains the sole live presence/Activity authority, so persisted history
never resurrects Agent Animals. Storage remains ordinary Claude-specific relational tables in the
single Turso/SQLite database—no graph engine, generic node/edge layer, or second database. A
session-only Developer Mode menu reveals a separate aggregate archive-health window backed by a
sanitized snapshot; it never exposes identifiers, paths, or record content.

Deliver in four vertical slices: data-root migration plus empty database bootstrap; explicit
archive opt-in plus raw Hook Receipts; incremental lossless transcript ingestion and schema indexes;
then the aggregate archive-health diagnostic surface.

## Technical Context

**Language/Version**: Rust 2021 with a workspace baseline raised from the currently inaccurate
declared 1.77 to Rust 1.90; POSIX shell for the Claude plugin wrapper; TypeScript 5.9/React 18 for
the integration preference and archive-health surfaces.

**Primary Dependencies**: Pin `turso` 0.8.0-pre.14 with default features disabled and no `sync`
feature; existing `serde`/`serde_json`; direct `sha2` dependency for stable 32-byte digests;
existing Tokio/Tauri async runtime and filesystem APIs; dev-only SQLite reader for the engine-
escape test. Do not add a Claude transcript crate. Current Turso resolution requires Rust 1.90
(`roaring` is the highest known floor), while the existing lock already exceeds the manifest's
declared 1.77. Slice 1 updates the workspace declaration and proves macOS/Linux/Windows targets;
failure returns to ADR-0004 rather than pinning a fragile transitive graph or changing engines.

**Storage**: Existing Markdown/YAML files below the Ranch data root remain authoritative for
Focuses, Timers, and Settings. New embedded Turso/SQLite-compatible file at `<data-root>/ranch.db`
stores schema version 1: Claude archive sessions, raw Hook Receipts, transcript source generations,
and raw transcript records plus nullable indexes. Version 1 has no cross-tool, graph, or generic
relationship tables. Aggregate health is a read model, not another persisted entity.

**Testing**: Rust unit tests for pure extraction, path/root resolution, migration decisions, and
cursor transitions; storage integration tests against temporary local databases and JSONL
fixtures; existing Unix hook socket tests; Settings/UI tests for explicit opt-in and disclosure;
menu/window tests for session-only Developer Mode and aggregate-only health; `task check`;
`task check:windows`; packaged Linux CI compile gate.

**Target Platform**: Tauri v2 desktop app. macOS is the primary runtime target; Unix hook ingestion
continues on macOS/Linux; Windows does not receive hooks yet but the workspace and database/path
code must compile and the data-root migration must behave correctly there. The archive is a Tokio
single-consumer task spawned through Tauri's existing runtime; blocking transcript reads use
`spawn_blocking`.

**Project Type**: Local-first desktop application with Rust domain/storage/command crates, a Rust
Tauri host, and a React/TypeScript webview.

**Performance Goals**: Keep the hook client's current 250 ms write bound and silent exit behavior;
perform no transcript reads on the socket accept thread; stream from the committed byte cursor with
memory bounded by the largest record plus one transaction batch; make a no-change sync proportional
to metadata checks rather than transcript size; ingest representative multi-megabyte transcripts
without loading the whole file.

**Constraints**: Zero runtime network calls, credentials, telemetry, remote database, or sync;
raw Claude content never appears in logs/events/UI diagnostics; user-only permissions on database
artifacts; exact raw-byte preservation; unknown/malformed complete records survive; no historical
directory census; no archive-to-render projection; database migrations and data-root move must be
rollback-safe. Archive admission is bounded and best-effort; disabling rejects new admissions while
accepted jobs finish. The unshipped archive schema and app/plugin protocol may change in lockstep
without compatibility shims, but existing user-owned Ranch data retains its migration guarantees.

**Scale/Scope**: Single-user local archive with one writer, concurrent hook producers, transcripts
from bytes to multi-megabytes, and a lifetime corpus that may reach gigabytes. Version 1 supports
only hook-observed Claude sessions, four tables, narrow indexes, and no browsing/search/export or
retention UI. The only archive UI is an aggregate diagnostic view hidden behind session-only
Developer Mode.

## Constitution Check

*GATE: Passed before Phase 0 research and re-checked after Phase 1 design.*

### I. User-Owned Local State — PASS

- Focus Markdown and timer sidecars remain canonical and are moved atomically with the entire data
  root.
- The archive is local plaintext under the user's data root, defaults off, has explicit disclosure,
  and enables neither Turso defaults nor its `sync` feature; the resolved graph has no HTTP/TLS
  runtime client.
- Both-root conflicts stop without merge, deletion, or overwrite. Database migrations are
  transactional and retryable.

### II. Independent Domains, Shared Renderer — PASS

- `ClaudeArchiveSession` is vendor-specific durable evidence and is not the harness-neutral live
  `AgentSession`.
- `LiveSessions` remains in memory and solely owns present Activity; database rows never create
  Animals after restart.
- No Focus, Task, Timer, renderer, Species, or Pen persistence is added to SQL.

### III. Functional Core, I/O at the Edges — PASS

- Envelope extraction, path/root decision rules, generation transitions, and cursor planning are
  pure functions over immutable inputs.
- Turso, filesystem reads, permissions, clocks, and worker/runtime details stay in `crates/storage`
  and the Tauri composition root.
- The worker and its archive sink hide meaningful ordering, transaction, and schema-drift
  complexity rather than forwarding calls.
- The archive-health command reads a sanitized snapshot published by the worker; React receives no
  database handle, raw row, identifier, or path.

### IV. Explicit, User-Controlled Integrations — PASS

- Archive consent is distinct from existing Agent rendering and defaults false for upgrades.
- `SubagentStop` adds one archive-only `observe` verb so its transcript can be captured without
  changing parent Activity; the wrapper also adopts the new data root.
- By explicit owner direction, the unshipped archive wire/schema contract changes in lockstep during
  rapid prototyping. The plugin manifest moves to exact contract version `0.2.0`; installation/repair
  replaces and validates the complete plugin. This satisfies protocol versioning without providing
  mixed-version adapters, negotiation, or backward readers/writers. Existing Focus, Timer, Settings,
  and data-root compatibility remain unaffected.
- The UI discloses the plaintext copy before opt-in. Disabling rejects new jobs immediately and
  drains already accepted work without silently removing history.

### V. Observable Completion in Small Slices — PASS

- The accepted spec provides behavior and measurable outcomes.
- Accepted ADR-0004 records the reversal of ADR-0002's transcript-ingestion rejection while keeping
  its live transport decision.
- Four review slices each have one observable promise and targeted proof before the full gate.

### VI. Evidence Is Not Authority — PASS

- Current code and local transcript inspection are recorded as observations, not inherited
  requirements.
- ADR-0001/0002/0003 are retained only for their accepted boundaries. ADR-0004 is the accepted new
  authority for raw archive ingestion.
- Third-party crates are research evidence; Ranch's own mapping is the chosen contract.
- The relational-only, Claude-specific v1 schema and aggregate-only debug view are present owner
  decisions recorded in the current spec rather than inferred future architecture.

### Post-design re-check

PASS. The database contract keeps raw records authoritative, the ingestion contract keeps I/O at
the storage edge, the archive-health contract exposes only aggregates, the exact plugin version
makes the lockstep prototype protocol explicit, and the data-root contract supplies conflict and
rollback behavior. No constitution exception remains.

## Project Structure

### Documentation (this feature)

```text
specs/008-claude-session-storage/
├── spec.md
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
└── contracts/
    ├── claude-ingestion.md
    ├── data-root-migration.md
    ├── archive-health.md
    └── ranch-db-v1.sql
```

### Source Code (repository root)

```text
Cargo.toml
Cargo.lock

crates/domain/src/
├── agents/
│   └── claude_code/
│       ├── archive.rs          # pure Claude archive entities/envelope projection
│       ├── archive_health.rs   # aggregate-only diagnostic read model
│       └── mod.rs
└── settings.rs                 # archive_enabled plus optional Claude config root

crates/storage/src/
├── claude_archive/
│   ├── mod.rs                  # narrow archive interface and construction
│   ├── database.rs             # Turso adapter and transactional writes
│   ├── migrations.rs           # ordered schema migrations/user_version
│   ├── transcript.rs           # streaming tail, path validation, generation detection
│   ├── health.rs               # sanitized snapshot + current-run counters
│   └── worker.rs               # DB ordering, drain semantics, health publication
├── agent_hooks/
│   ├── journal.rs              # existing bounded UI journal remains content-free
│   └── server/                 # submits owned frame to live and archive paths
└── lib.rs

crates/storage/tests/
├── claude_archive.rs           # byte fidelity, idempotency, lag, malformed/unknown rows
├── data_root_migration.rs      # path-state matrix and rollback behavior
├── fixtures/claude/            # synthetic, non-secret hook/transcript fixtures
└── agent_hook_socket.rs        # existing transport contract plus archive isolation

crates/commands/src/
└── archive_health.rs           # read-only command over sanitized storage snapshot

src-tauri/src/app/
├── mod.rs                      # prepare root, migrate DB, compose worker/sinks
├── menu.rs                     # Developer Mode toggle + dynamic health item lifecycle
├── paths.rs                    # explicit/default root resolution and ranch.db path
└── agent_hooks.rs              # independent live and archive routing

plugins/adhd-ranch-hooks/scripts/send-hook
                                      # resolves `.adhd` default; override unchanged

src/
├── archive-health.tsx          # separate diagnostic window entry
├── api/archiveHealth.ts        # typed Tauri reader
└── components/
    ├── ArchiveHealthWindow.tsx # aggregate-only ready/empty/unavailable states
    ├── ArchiveHealthWindow.test.tsx
    └── ...                     # existing integration preference/disclosure surface

archive-health.html
vite.config.ts                  # archive-health multi-page entry
knip.json                       # archive-health frontend entry

README.md
docs/architecture.md
docs/adr/0004-local-claude-session-archive.md
```

**Structure Decision**: Keep the Claude archive vocabulary under the existing Claude integration
namespace, place all Turso/filesystem/runtime behavior in one deep `claude_archive` storage module,
and compose it beside—not inside—the harness-neutral `LiveSessions` reducer. Publish one sanitized
health snapshot from that module rather than letting a UI query arbitrary rows. No new crate is
needed: the existing `domain`, `storage`, `commands`, and Tauri boundaries already match the pure-
model/side-effect split.

## Delivery Slices

### Slice 1 — One data root and a usable empty database

**Completion promise**: An upgrade with only the legacy default root starts from `.adhd` with every
existing file intact and a migrated, reopenable schema-v1 `ranch.db`; ambiguous roots are untouched
and rejected.

Implementation:

- Raise the workspace Rust baseline to 1.90 and prove pinned local-only Turso under package targets.
- Add pure default/explicit root resolution and migration decision tests.
- Prepare/move the root before current Settings/Focus path resolution.
- Add database path, explicit transactional schema-v1 migration, per-connection foreign keys,
  permissions, Turso reopen, and clean-shutdown SQLite reopen/integrity/read tests.
- Update the plugin wrapper's default path in the same slice so hooks do not split across roots.
- Update README/current architecture guidance in this slice; leave historical records untouched.

Proof: data-root state-matrix tests, database create/reopen/user-version tests, existing Focus and
Settings tests, Unix plugin contract tests, workspace/Windows compile checks.

### Slice 2 — Explicit opt-in writes lossless Hook Receipts

**Completion promise**: After the user explicitly enables Claude archiving, every Hook Firing
successfully admitted to the bounded archive queue creates a queryable raw receipt without changing
live Agent behavior or exposing content in diagnostics; rejected admissions are counted.

Implementation:

- Add `claude.archive_enabled` with false default, Settings round-trip, preference control, and
  plaintext disclosure.
- Add pure Hook Receipt envelope extraction and digesting.
- Add the bounded queue, dropped-job counter, and single async archive worker; upsert the session
  before its immediate-FK receipt insert in one Turso transaction.
- Route an owned payload copy to the archive worker beside the current `LiveSessions`/journal path.
- Split the current coupled gate: archive admission follows `claude.archive_enabled`, while live
  lifecycle projection alone follows `agents.enabled`.
- Gate admission from the latest archive preference: disabling rejects new work immediately while
  the worker drains every already accepted job exactly once.
- Keep the in-memory debug journal bounded and content-free.

Proof: opt-in/off tests, exact receipt-byte/digest tests, duplicate-receipt tests, database failure
isolation, deterministic saturation/drop-count and disable-while-queued tests, existing hook socket
timeout/silent-failure tests, live session/render regression tests.

### Slice 3 — Incremental transcript generations and Ranch-owned mapping

**Completion promise**: Hook-observed Claude main and subagent transcripts are archived
incrementally with exact raw records, unknown shapes preserved, bounded settle retries idempotent,
and detected truncation/replacement retained as a new generation.

Implementation:

- Resolve the Claude config root from `claude.config_dir`, app-inherited `CLAUDE_CONFIG_DIR`, or the
  platform default, then validate canonical paths against its `projects/` child.
- Register `SubagentStop` with the archive-only `observe` verb and ingest
  `agent_transcript_path` beside main `transcript_path` without touching `LiveSessions`.
- Parse the wire verb into a Claude-specific transport action and project only lifecycle verbs into
  the existing harness-neutral `HookAction`; keep `observe` out of `LiveSessions` entirely.
- Bump the exact plugin contract to `0.2.0`; extend runtime validation to check the manifest version,
  event table, wrapper bytes, and executable bit, and make only explicit Install/Repair replace a
  mismatch. Startup reports an unhealthy installation without mutating it. Add no old-verb shim,
  handshake, mixed-version path, or schema compatibility code.
- Add streaming JSONL tailer, partial-tail handling, source cursors, generation detection, and
  transactional batch commits.
- Extract the stable nullable envelope defined in `data-model.md`; never gate storage on type.
- Coalesce repeated sync hints by path and schedule immediate plus bounded delayed retries; terminal
  hooks settle only after two unchanged metadata/boundary observations or the retry cap.
- Add synthetic fixtures for known, unknown (`cost-state`), malformed, partial, split-message,
  duplicate, replaced, resumed, and sidechain records.

Proof: byte-for-byte fixture comparison, three-pass/reopen idempotency, generation preservation,
custom-config-root/path escape rejection, delayed-final-row capture, multi-megabyte streaming test,
zero archive-derived Animals, user-only DB/WAL/SHM permissions, network-denied runtime proof, and
full gates.

### Slice 4 — Aggregate archive health behind session-only Developer Mode

**Completion promise**: In every build, enabling Developer Mode for the current app run reveals a
read-only archive-health menu item whose window reports only approved aggregates; restarting hides
the item and resets run-local counters without changing persisted archive facts.

Implementation:

- Add a non-persisted `DeveloperModeState`, unchecked on every launch, plus an always-present
  `Developer Mode` check item. Enabling dynamically appends `Claude Archive Health…` to the Window
  submenu; disabling removes it and closes the window. Retain the submenu and item handles/IDs so
  repeated toggles are idempotent and never change Settings YAML.
- Have the archive worker publish a sanitized snapshot after bootstrap and each committed receipt
  or transcript batch: schema version, aggregate table counts, last successful ingest time, and
  total bytes for `ranch.db` plus existing WAL/SHM sidecars.
- Track dropped jobs and a bounded enum of content-free error categories in memory for the current
  run; label them `this run` and reset them naturally at process start.
- Add a read-only commands/Tauri boundary over that snapshot and a separate archive-health window
  following the existing Agent Hooks debug-window pattern. Both the menu open handler and command
  authorize against live `DeveloperModeState`, so a stale or forged webview cannot read after the
  toggle is disabled.
- Render loading, empty, ready, and unavailable states. Never return identifiers, paths, record
  rows, raw JSON, prompts, tool data, or source content to the webview.

Proof: pure aggregate/snapshot tests, command redaction tests, component state tests, dynamic-menu
idempotency tests, release-profile host/menu tests plus a packaged smoke, restart/reset proof,
direct SQL/file-size parity against synthetic fixtures, and ingestion-isolation tests while the
window polls.
