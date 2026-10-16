# Phase 0 Research: Claude Session Archive

## Decision: Preserve raw Claude records; do not adopt a transcript crate as authority

**Decision**: Store each Hook Firing payload and each completed transcript JSONL line as raw bytes.
Extract a narrow nullable index with `serde_json::Value`. Do not deserialize into a closed enum
before the authoritative copy is committed.

**Rationale**: Claude documents transcript location and retention but does not publish a stable
transcript JSON schema. The current local corpus includes a `cost-state` record that is not in
`claude-code-transcripts` 0.1.12. That crate's `#[serde(other)] Unknown` variant preserves parse
continuity but not the unknown payload when reserialized, and its known structs do not flatten
unrecognized fields, so known variants also lose newly added members. Raw-first storage makes
schema drift a projection concern instead of a data-loss event. Claude also warns that transcripts
contain every prompt, tool read, command output, and possible credential, so this decision includes
explicit opt-in and local plaintext disclosure.

**Alternatives considered**:

- [`claude-code-transcripts`](https://crates.io/crates/claude-code-transcripts): closest match for a
  comprehensive typed Rust model and useful fixture/round-trip evidence, but unsuitable as the sole
  archival representation because unknown variants lose their bodies.
- [`claude-session-parser`](https://crates.io/crates/claude-session-parser): useful streaming usage,
  chat, and activity projections, but deliberately not a lossless storage model.
- [`txcript`](https://crates.io/crates/txcript): useful cross-harness conversion layer, but its
  published Rust 1.96 requirement exceeds the planned Rust 1.90 baseline and its
  conversion model is broader than this slice.
- [`smc-cli-cc`](https://crates.io/crates/smc-cli-cc): a search-oriented library, not a storage or
  round-trip schema authority.
- [`agent-sessions`](https://crates.io/crates/agent-sessions): useful reference for byte offsets,
  CR/LF-preserving framing, and incomplete-tail handling, but its model is not Ranch's durable
  schema and adopting it does not solve unknown-field preservation.
- [`cc-session-jsonl`](https://crates.io/crates/cc-session-jsonl): typed convenience parser with the
  same payload-free unknown/field-loss limitation as other closed models.
- Copy whole JSONL files: preserves bytes but provides weak idempotency and lookup, and duplicates
  entire files on every change unless another index is invented beside them.

## Decision: Model Claude's structures in two layers

**Decision**: Map only stable envelopes into columns and retain every event-specific structure in
the raw payload. The mapped transcript envelope includes record type, entry UUID, parent and logical
parent UUIDs, both observed `sessionId` and `session_id` spellings, timestamp, prompt id, agent id,
sidechain flag, message role, message id, and model. The mapped hook envelope includes session id,
prompt id, main/subagent transcript paths, cwd, event name, and receipt time.

**Rationale**: Current Claude transcript records include message-bearing `user`, `assistant`,
`system`, `attachment`, and `progress` shapes plus many metadata records such as titles, modes,
summaries, file-history snapshots, worktree state, queue operations, cost state, and internal
coordination markers. Their common envelope is useful for ancestry and correlation lookups; the
variant bodies change too quickly to normalize without a concrete consumer. Claude's official
[hook reference](https://code.claude.com/docs/en/hooks) defines the common hook fields and event
extensions, making that envelope a stronger contract than the transcript body.

**Alternatives considered**:

- Normalize every content block and metadata subtype now: rejected as a second implementation of a
  private schema with no current query consumer.
- Store only JSON: rejected because session, prompt, ancestry, type, and time lookups are already
  required to validate ingestion and are expensive to derive repeatedly.

## Decision: Hooks are receipts and synchronization hints, not the archive payload

**Decision**: Persist the raw Hook Receipt after successful queue admission, then use
`transcript_path` and `agent_transcript_path` to schedule incremental transcript synchronization.
Register `SubagentStop` with an archive-only transport verb so it does not affect live parent
Activity. Every hint gets immediate and bounded delayed retries; terminal events settle after two
unchanged observations or a retry cap. Do not manufacture Claude transcript rows from hook inputs.

**Rationale**: Hook JSON and transcript JSONL have different schemas and cadences. The official
[hook documentation](https://code.claude.com/docs/en/hooks#common-input-fields) states that the
transcript is written asynchronously and may not yet include the current turn. Hooks provide the
session identity and exact file path needed to synchronize, while Stop exposes
`last_assistant_message` for consumers that need current final text. A retryable cursor reconciles
the lag without blocking Claude's hook process.

**Alternatives considered**:

- Treat each hook as a transcript row: rejected because it invents a third schema and still omits
  most message and tool data.
- Read the entire transcript synchronously inside the hook client: rejected because it violates the
  client's bounded, dependency-light, silent-failure contract.
- Poll all Claude project directories: rejected for this slice because it expands privacy scope and
  couples Ranch to undocumented project-directory naming.

## Decision: Keep archive history separate from live Agent Session truth

**Decision**: Continue to apply Hook Firings to `LiveSessions` for current Activity and render
projection. Add archival as a sibling/decorator side effect. Never rebuild live Agent Sessions from
database rows on startup.

**Rationale**: A historical transcript proves that a session existed, not that its process still
exists. Rehydrating live state would create stale Animals and violate ADR-0001's independent domain
boundary. The database is evidence for future Claude features; `LiveSessions` remains the present
tense.

**Alternatives considered**:

- Replace `LiveSessions` with a SQL-backed repository: rejected because persistence cannot answer
  liveness after a restart.
- Keep the archive completely outside the hook pipeline: rejected because the hook provides the
  supported session id and transcript path correlation keys.

## Decision: Use embedded Turso in local-only mode

**Decision**: Pin `turso` 0.8.0-pre.14, disable default features, and open
`<data-root>/ranch.db` with `turso::Builder::new_local`. Do not enable the `sync` feature. Raise the
workspace Rust baseline to 1.90 and prove all packaged targets in Slice 1. Isolate the API inside
`crates/storage`.

**Rationale**: Turso's current [local-development guidance](https://github.com/tursodatabase/turso-docs/blob/main/local-development.mdx)
recommends an in-process local database and shows `Builder::new_local`; no server or credentials are
needed. The engine uses SQLite-compatible files, and Turso documents a path back to SQLite-compatible
readers in its [compatibility guarantees](https://github.com/tursodatabase/turso/blob/main/COMPAT.md).
Local-only mode fits the constitution's no-network runtime rule. With defaults and `sync` disabled,
the resolved graph has no HTTP/TLS runtime client, although Turso still unconditionally includes
sync protocol/data packages. The current graph requires Rust 1.90 (`roaring` is the highest known
floor), while this repository's existing lock already contains dependencies newer than its
declared Rust 1.77. Updating the honest workspace baseline is safer than pinning a brittle
transitive graph. The adapter boundary contains the risk of the crate's pre-1.0 API and undeclared
MSRV.

**Alternatives considered**:

- `libsql`: mature Turso lineage and stable releases, but the current Turso documentation now
  recommends the `turso` package for embedded local databases.
- `rusqlite`: mature and simple, but does not follow the owner's Turso direction.
- Turso Cloud or embedded replication: rejected because the product must make no runtime network
  calls and store no provider credentials.

## Decision: Keep schema version 1 relational and Claude-specific

**Decision**: Use ordinary Turso/SQLite tables, foreign keys, and indexes in the single
`ranch.db`. Add no graph-native engine, graph sidecar, generic node/edge layer, graph query API, or
cross-tool session/event/artifact/relationship tables. Generalize only when a second integration
provides concrete common queries.

**Rationale**: Parent UUIDs, logical parents, sessions, prompts, and subagents are already
queryable through focused relational indexes. A generic graph model today would be speculative and
would not improve raw-data fidelity. The user's future visualization goal is preserved by keeping
raw records and stable source identities, not by guessing cross-tool ontology in Claude-only v1.

**Alternatives considered**:

- Replace Turso with an embedded graph database: rejected by present owner direction and because it
  adds a second technology before a graph query exists.
- Add generic nodes/edges beside Claude tables: rejected until another integration demonstrates
  shared identity and relationship semantics.
- Derive a relationship visualization now: rejected; this feature exposes only aggregate health.

## Decision: One serialized archive worker owns database and transcript I/O

**Decision**: Start one Tokio single-consumer archive task through Tauri's existing runtime after
database migration. The socket path records the in-memory live change as today and uses bounded
`try_send` admission for an owned raw frame. The worker owns the Turso connection, serializes
receipt writes and transcript cursor updates in explicit transactions, sends blocking transcript
reads through `spawn_blocking`, and reports failures through content-free diagnostics. Disabling
archival closes the admission gate atomically while every job already accepted remains owned by the
worker and finishes exactly once.

**Rationale**: The current hook server accepts on a dedicated blocking thread and invokes a
synchronous sink. Turso's Rust API is async. A dedicated worker creates one meaningful boundary:
the hook server never holds an async runtime or database connection, and database ordering and
transaction ownership stay local. Transcript parsing may touch multi-megabyte files and therefore
must not occupy the socket accept loop.

**Alternatives considered**:

- Block the accept loop on Turso and transcript reads: rejected because one large transcript can
  delay every subsequent Hook Firing.
- Give each firing its own connection/task: rejected because it weakens ordering and complicates
  cursor races for no expected throughput benefit.
- Put Turso in domain types: rejected by the functional-core/I/O-edge rule.
- Cancel queued work when archival is disabled: rejected because accepted jobs would lose their
  receipt guarantee and could strand transcript batches between planned operations.

## Decision: Cursor by byte offset and retain file generations

**Decision**: Track canonical path, generation, observed file identity metadata, byte cursor,
completed-line ordinal, last size/mtime, and SHA-256 of the exact bytes
`[max(0, cursor - 4096), cursor)`. The cursor always identifies the first unread byte; an incomplete
tail is reread from there and is not persisted separately. Insert transcript rows under a unique
`(source_id, ordinal)` key. If the file shrinks, its identity changes, or the boundary window no
longer matches, supersede the old generation and create the new one in the same transaction.

**Rationale**: Byte offsets support streaming and exact raw preservation. UUIDs cannot be the sole
key because metadata rows may lack them and newer Claude versions may split one logical assistant
message over several records. Generations prevent a resumed/replaced file from silently rewriting
history. A transaction commits inserted rows and the new cursor together, making retries
idempotent.

**Alternatives considered**:

- Key by transcript UUID only: rejected because many record types have no UUID and repeated content
  blocks can share message identity.
- Key by raw hash only: rejected because identical records at different positions are distinct
  source facts.
- Update archived rows in place after truncation: rejected because it destroys evidence and makes
  recovery ambiguous.

## Decision: Resolve one trusted Claude projects root

**Decision**: Resolve Claude's config directory once at app startup from explicit
`claude.config_dir` in Ranch settings, then `CLAUDE_CONFIG_DIR` inherited by the app, then the
platform home default `.claude`. Validate every main or subagent transcript against the canonical
`projects/` child. A hook-provided path never defines or widens the root.

**Rationale**: Claude may run with a custom config directory that the desktop process did not
inherit. An explicit setting makes that case testable and avoids guessing a root from an untrusted
path hint. Canonical descendant checks still reject relative paths, symlink escapes, and unrelated
regular files.

**Alternatives considered**:

- Infer the root from each `transcript_path`: rejected because any local socket sender could widen
  the read boundary.
- Support only `~/.claude`: rejected because it silently breaks documented custom configurations.

## Decision: Make database migrations and engine escape executable contracts

**Decision**: Include explicit transaction bounds around each schema migration, set and verify
foreign keys on every writer connection, and test clean shutdown/checkpoint followed by reopen,
integrity check, and reads through a dev-only SQLite reader.

**Rationale**: Turso's `Connection::execute_batch` does not create a transaction and
`PRAGMA foreign_keys` is connection-local. Turso documents SQLite file compatibility but also has
SQL-semantic and journaling differences, so a tested escape is stronger than assuming that the file
format alone makes rollback safe.

**Alternatives considered**:

- Rely on a multi-statement batch being atomic: rejected because it can leave a partial schema.
- Describe SQLite compatibility without testing another reader: rejected because it overstates
  reversibility of a pre-1.0 engine.

## Decision: Migrate the whole default data root before opening stores

**Decision**: Add a startup preparation step that resolves explicit versus default roots. When only
the legacy default exists, rename the complete directory to the new root on the same volume. When
both exist, stop without writes. Only after preparation may Settings, Focuses, sockets, the stable
hook client, and `ranch.db` be opened.

**Rationale**: Moving individual children risks split-brain state and a plugin wrapper pointing at
the wrong socket/client. A same-parent directory rename is atomic on supported default layouts.
The explicit environment override is a user contract and must not be interpreted as legacy data.

**Alternatives considered**:

- Fall back to the legacy root forever: rejected because the rename would never complete and new
  installs would diverge from upgraded installs.
- Copy then delete: rejected because interruption can leave two apparently valid roots.
- Auto-merge when both exist: rejected because there is no safe winner for Focuses or Settings.

## Decision: Separate archive opt-in from Agent rendering

**Decision**: Add a Claude-specific archive preference, default false, disclosed alongside hook
installation. Agent rendering remains controlled by `agents.enabled`; archive writes are controlled
by the new preference. Existing users do not begin copying transcript content merely by upgrading.

**Rationale**: Displaying live session identity is materially different from duplicating complete
prompts and tool output. The constitution requires external integration effects to be explicit and
user-controlled. Separate switches also preserve the independent live/archive semantics.

**Alternatives considered**:

- Reuse `agents.enabled`: simpler, but silently broadens an existing display preference into a
  sensitive retention choice.
- Treat plugin installation as permanent archive consent: rejected because existing installations
  predate this behavior and no longer represent informed consent.

## Decision: Version one lockstep prototype plugin contract without compatibility adapters

**Decision**: Bump the exact Ranch Claude plugin contract to `0.2.0`, add the sixth
`SubagentStop -> observe` mapping, and require app validation/explicit Install or Repair to replace
every mismatched version, event table, wrapper, or executable. Keep the existing
`verb\n<verbatim payload>` frame and silent 250 ms client. Add no handshake, alias, old reader,
mixed-version path, or schema compatibility layer.

The existing validator already compares the complete event table, wrapper bytes, and executable
bit, but currently asserts the manifest version only in a test. The implementation must move the
exact `0.2.0` check into runtime validation so status and Install/Repair share one contract.

**Rationale**: The owner explicitly chose rapid lockstep prototyping over backward compatibility.
An exact manifest version still makes the breaking protocol explicit and satisfies the
constitution's versioning rule. The current installer already owns full-contract validation and
explicit repair; startup must report mismatch rather than mutating a user installation. Existing
Focus, Timer, Settings, and whole-root migration guarantees are outside this prototype exception.

At the socket boundary, parse a Claude-specific transport action. Archive admission independently
follows `claude.archive_enabled`; only start/working/idle/end project into harness-neutral
`HookAction` when `agents.enabled`; `observe` never reaches `LiveSessions`.

**Alternatives considered**:

- Support old and new verbs or negotiate a frame version: rejected because no released archive
  consumer needs compatibility and the user explicitly declined it.
- Put `Observe` into the live `HookAction` enum: rejected because it is archive evidence, not a
  live Activity transition.
- Auto-repair on startup: rejected because plugin mutation requires explicit user action.

## Decision: Publish aggregate archive health as a sanitized in-memory read model

**Decision**: After database bootstrap and each successful receipt or transcript commit, the
single worker publishes an immutable `ArchiveHealthSnapshot`. It contains availability, schema
version, aggregate counts for the four Claude tables, the latest successful commit timestamp, and
the combined byte size of existing `ranch.db`, WAL, and SHM files. A separate current-run metrics
object contributes rejected-admission count and counts keyed by a bounded, content-free error enum.
The Tauri command returns only this projection; it never performs arbitrary UI-directed SQL.

**Rationale**: The worker can establish a transaction-consistent initial count and update it only
after commits, so UI polling does not contend for the writer or expose raw rows. On restart, the
worker rebuilds persisted aggregates from the database while current-run counters naturally reset.
Separating persisted facts from runtime metrics also avoids pretending that database-unavailable
errors can always be written into the failed database.

`last_successful_ingest_at` is the latest successful receipt or transcript batch commit time, not a
source timestamp. Database size is the sum of metadata lengths for the main file and any existing
`-wal`/`-shm` sidecars at snapshot refresh. Persisted per-row `parse_error` values remain database
facts; runtime error counters are operational categories and never include their messages.

**Alternatives considered**:

- Let the webview issue aggregate SQL: rejected because React must remain view-only and must not
  receive a database capability.
- Persist diagnostic counters: rejected because they are session-local and cannot reliably record
  failures while the database is unavailable.
- Reuse the existing hook-firing debug feed: rejected because it exposes session identifiers and
  is a different diagnostic contract.

## Decision: Gate the aggregate view with non-persisted Developer Mode

**Decision**: Include Developer Mode in every build as process-local Tauri state initialized off.
An explicit check item is always present in the Window menu. Because Tauri menu items have no
visibility setter, enabling the mode appends a separate Archive Health item to the retained Window
submenu and disabling removes it and closes the window. Repeated transitions are idempotent. The
state is never written to Settings and resets off on every launch; both the open handler and the
read command check it directly.

**Rationale**: Packaged builds need the same diagnostic proof as development builds, but ordinary
users should not see a temporary test surface. A separate window follows the existing Agent Hooks
debug-window pattern and keeps diagnostics out of the ambient overlay. Session-only state avoids a
new durable preference and makes each use intentional.

**Alternatives considered**:

- Compile the view only in debug builds: rejected because packaged behavior could not be tested.
- Persist Developer Mode: rejected because a temporary diagnostic surface should not remain
  enabled across normal launches.
- Add archive health to the existing Agent Hooks window: rejected because that window currently
  exposes identifiers and event rows forbidden by the archive-health contract.
