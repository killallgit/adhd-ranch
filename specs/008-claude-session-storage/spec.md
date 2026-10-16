# Feature Specification: Claude Session Archive

**Feature Branch**: `008-claude-session-storage`

**Created**: 2026-09-28

**Status**: Approved for planning

**Input**: User description: "Create a clear Claude-data entity, map Claude Code's on-disk session structures, and use the existing hook path to write equivalent session data into a local Turso database. Rename the default Ranch application-data directory from `.adhd-ranch` to `.adhd` and store `ranch.db` there."

## Clarifications

### Session 2026-09-28

- Q: What should “graph database” mean for Ranch’s durable storage? → A: No graph database for
  this feature; use only the local Turso/SQLite database.
- Q: How much cross-tool structure should schema version 1 introduce now? → A: Keep version 1
  Claude-specific and generalize only when a second integration supplies concrete shared concepts.
- Q: Should this feature include any relationship visualization or browsing interface? → A: Add a
  basic developer-only debug overview that can be opened from the app menu for testing.
- Q: What information should the first archive debug overview display? → A: Aggregate health only:
  schema version, database size, counts, last ingest time, dropped jobs, and error categories.
- Q: If the archive worker is overloaded while Ranch is running, what loss guarantee should apply
  to raw Hook Receipts? → A: Use bounded best-effort admission, count rejected receipts, and never
  delay Claude.
- Q: In which builds should the archive-health view be available? → A: Include it in every build,
  hidden behind an explicit Developer Mode toggle in the app menu.
- Q: What time period should the debug view’s dropped-job and error-category counts cover? → A:
  Show current-app-run counters that reset when Ranch restarts.
- Q: How should the new `SubagentStop` archive verb remain compatible with older Ranch apps and
  installed plugins? → A: It does not need backward compatibility; the app, plugin, and unshipped
  archive schema may change together during rapid prototyping.
- Q: What should happen to archive jobs already accepted when the user disables archiving? → A:
  Finish accepted jobs atomically while rejecting every new Hook Firing after disable.
- Q: Should Developer Mode remain enabled after Ranch restarts? → A: No; it is session-only and
  resets to off on every app launch.

## User Scenarios & Testing

### User Story 1 - Upgrade to One Ranch Data Home (Priority: P1)

As an existing Ranch user, I keep my Focuses, Settings, and hook installation working after the
default application-data directory changes from `.adhd-ranch` to `.adhd`, and the new database is
created alongside that state.

**Why this priority**: Every later database write depends on one unambiguous data root. Losing or
splitting existing user-owned state would make the archive unsafe to ship.

**Independent Test**: Start the app against a fixture containing only the legacy data root, then
verify that the complete directory is moved to the new root, `ranch.db` is initialized there, and
the existing Focuses and Settings still load unchanged.

**Acceptance Scenarios**:

1. **Given** the default legacy root exists and the new root does not, **When** Ranch starts,
   **Then** Ranch atomically moves the legacy root to the new location before opening any stores.
2. **Given** no Ranch data root exists, **When** Ranch starts, **Then** it creates `.adhd`, creates
   `ranch.db`, and continues to create the existing Focus and Settings files below that root.
3. **Given** `ADHD_RANCH_HOME` is explicitly set, **When** Ranch starts, **Then** it uses that exact
   directory and performs no default-root rename.
4. **Given** both default roots already exist, **When** Ranch starts, **Then** it refuses to merge
   them automatically, leaves both unchanged, and reports an actionable conflict.

---

### User Story 2 - Preserve Claude Session Data (Priority: P2)

As a Ranch user who opted into the Claude integration, I have a local archive of the Claude hook
receipts and transcript records Ranch observed while it was running, so later Ranch features can
understand a session without re-reading Claude's private file layout.

**Why this priority**: This is the first durable Claude-data capability and the reason for adding
the database.

**Independent Test**: Replay fixture hook payloads that point to a representative Claude JSONL
transcript, then compare every complete source record byte-for-byte with its archived raw value and
query the session, event type, prompt, parent, and timestamp indexes.

**Acceptance Scenarios**:

1. **Given** a valid Hook Firing for a Claude session, **When** Ranch accepts it into the archive
   queue, **Then** Ranch records the raw hook payload and its stable common fields in `ranch.db`.
2. **Given** the firing names a valid Claude transcript under Claude's configured `projects/`
   directory, **When** Ranch processes the firing, **Then** it archives every newly completed JSONL
   record from that transcript without rewriting the raw record.
3. **Given** a `SubagentStop` firing names `agent_transcript_path`, **When** Ranch processes it,
   **Then** it archives that subagent transcript under the parent session without changing the
   parent's live Activity.
4. **Given** Claude adds a record type or field Ranch does not understand, **When** the record is
   ingested, **Then** its raw bytes remain queryable and the unknown shape does not abort the
   session ingest.
5. **Given** the same transcript is visited by several firings or after an app restart, **When** it
   is synchronized again, **Then** already archived transcript records are not duplicated.

---

### User Story 3 - Keep Live Agent Behavior Independent (Priority: P3)

As a Ranch user, Claude archiving does not make the overlay slower or resurrect stale Agent
Sessions: the existing live session reducer remains the authority for what is drawn now, while the
database is historical evidence.

**Why this priority**: Persisted history and live presence have different truth and retention
rules. Combining them would reverse the accepted Focus/Agent/renderer boundaries and create ghost
Animals after restart.

**Independent Test**: Ingest a complete archived session, restart Ranch, and verify that no Agent
Animal appears until a new live Hook Firing arrives; also verify that hook delivery remains silent
and bounded when the database or transcript cannot be read.

**Acceptance Scenarios**:

1. **Given** archived Claude sessions exist, **When** Ranch starts, **Then** no Agent Session is
   reconstructed solely from the archive.
2. **Given** a database write or transcript read fails, **When** a Claude hook runs, **Then** the
   hook client still exits silently and within its existing time bound.
3. **Given** Agents as Animals is disabled, **When** hooks fire, **Then** Ranch does not begin a new
   archive without the user-visible integration opt-in defined by this feature.

---

### User Story 4 - Inspect Archive Health During Development (Priority: P4)

As a Ranch developer or tester, I can open a basic archive-health overview from the app menu to
confirm that Claude data is being recorded without using a database shell.

**Why this priority**: The archive is otherwise invisible in this feature, making integration and
fixture testing unnecessarily indirect. This is a diagnostic surface, not an end-user archive.

**Independent Test**: Ingest synthetic main and subagent fixtures, open the debug overview from the
menu, and verify that its schema version, database size, aggregate row counts, last-ingest time,
dropped-job count, and error-category counts match direct observations without displaying session
identifiers or raw prompt, message, tool, or source-code content.

**Acceptance Scenarios**:

1. **Given** any development or packaged build, **When** the tester enables Developer Mode and
   selects the archive-health item from the app menu, **Then** Ranch opens the aggregate overview.
2. **Given** Developer Mode is off, **When** the app menu is displayed, **Then** the archive-health
   item is hidden from normal users.
3. **Given** Developer Mode was enabled, **When** Ranch restarts, **Then** Developer Mode is off and
   the archive-health item is hidden again.
4. **Given** no archived data or an unavailable database, **When** the overview opens, **Then** it
   shows an explicit empty or unavailable state without affecting hook ingestion.
5. **Given** archived records contain sensitive content, **When** the overview renders, **Then** it
   displays only the approved diagnostic projection and never raw payload or transcript content.

### Edge Cases

- Claude's transcript file does not exist yet or lags the Hook Firing that names it.
- Claude writes final main- or subagent-transcript rows after the last lifecycle hook has returned.
- The transcript ends with a partially written JSONL record.
- A complete transcript line is malformed JSON or contains a record type unknown to Ranch.
- Claude truncates, replaces, forks, or resumes a transcript at the same path.
- Two hook processes for one session arrive concurrently or duplicate the same logical event.
- The bounded archive queue is full; the receipt is rejected, the dropped-job count advances, and
  Claude continues without waiting for durable admission.
- A hook-supplied transcript path is relative, is a symlink escape, is outside the configured
  Claude `projects/` root, or names a non-regular file.
- The database is locked, corrupt, read-only, or the disk fills during a transaction.
- The app crashes between moving the data directory and opening the database.
- The user rolls back to an older Ranch build after the directory migration.
- The legacy and new data roots both exist because a previous manual move was incomplete.
- The debug overview opens while ingestion is active, before any rows exist, or while the database
  is unavailable.
- The installed plugin and running app use different prototype archive contracts; Ranch reports the
  integration as unhealthy and requires install/repair rather than translating either version.
- The user disables archiving while jobs are queued; no new jobs are admitted, and already accepted
  jobs finish without partial receipt or transcript transactions.

## Requirements

### Functional Requirements

- **FR-001**: The default Ranch data root MUST be `.adhd` under the user's home directory on Unix
  and `adhd` under the platform application-data directory on Windows.
- **FR-002**: An explicit `ADHD_RANCH_HOME` MUST remain authoritative and MUST NOT be renamed or
  migrated automatically.
- **FR-003**: When only the legacy default root exists, startup MUST move the complete legacy root
  to the new default root before any component reads or writes Ranch state.
- **FR-004**: When both default roots exist, startup MUST NOT merge, delete, or overwrite either
  root automatically.
- **FR-005**: The local database MUST be stored at `<data-root>/ranch.db`; the feature MUST use no
  remote database, credentials, replication, telemetry, or other outbound network behavior.
- **FR-006**: Database schema changes MUST be ordered, versioned, transactional, and safe to retry
  after interruption.
- **FR-007**: The Claude archive MUST distinguish a Claude-specific persisted session record from
  the harness-neutral, in-memory Agent Session used by the renderer.
- **FR-008**: A Hook Firing is accepted for archival only after successful admission to the archive
  worker queue. For every accepted firing, Ranch MUST preserve the complete input payload and
  extract the stable common fields needed for lookup: session id, hook event, prompt id when
  present, agent id when present, working directory, `transcript_path`, `agent_transcript_path`,
  and receipt time. Admission MUST be bounded and non-blocking; rejected admissions MUST increment
  a content-free dropped-job counter and MUST NOT use a durable overflow inbox or delay Claude.
  Graceful shutdown MUST close admission and drain all accepted jobs before stopping the worker;
  forced termination cannot guarantee uncommitted receipts.
- **FR-009**: Ranch MUST use hook-provided `transcript_path` and `agent_transcript_path` values as
  synchronization hints and MUST NOT discover sessions by crawling all of Claude's `projects/`
  directory in this feature. `SubagentStop` MUST use an archive-only transport action that never
  changes the parent session's live Activity. The prototype action requires a matching app/plugin
  contract and MUST NOT add a compatibility shim or mixed-version protocol path.
- **FR-010**: Ranch MUST read a transcript only when its canonical path is a regular file within
  the configured Claude `projects/` directory. The config directory MUST resolve from an explicit
  Ranch setting, then the app's inherited `CLAUDE_CONFIG_DIR`, then the platform default; a hook
  path MUST NOT widen this root. Rejected paths MUST be observable in diagnostics without exposing
  transcript content.
- **FR-011**: Every completed transcript record MUST retain its exact content and line-delimiter
  bytes plus nullable, best-effort indexes for record type, entry id, parent id, logical parent id,
  session id,
  timestamp, prompt id, agent id, sidechain flag, message role, message id, and model. Camel-case
  `sessionId` and snake-case `session_id` MUST remain separate nullable projections.
- **FR-012**: Unknown record types and unknown fields MUST be retained in full. Typed projections
  MUST never be the sole copy of a Claude record.
- **FR-013**: A malformed but complete transcript record MUST be retained with its parse error;
  every unterminated trailing record MUST remain pending until its newline delimiter arrives,
  even if stable reads parse it as a complete JSON value.
- **FR-014**: Transcript synchronization MUST be incremental and idempotent across duplicate Hook
  Firings and application restarts. Every valid path hint MUST schedule bounded delayed retries;
  terminal main/subagent events MUST settle on two unchanged observations so asynchronous final
  records do not require a later Hook Firing.
- **FR-015**: A detected transcript replacement or truncation MUST create a new source generation
  rather than overwriting or silently attaching new bytes to the old generation. Detection MUST
  compare file identity when available and the exact committed 4 KiB boundary window.
- **FR-016**: Hook receipts MAY record duplicate deliveries as separate observations, but each MUST
  carry a payload digest so downstream projections can identify equivalent deliveries.
- **FR-017**: Database writes and transcript reads MUST occur behind the existing storage and
  integration boundaries; domain modules MUST remain free of filesystem, database, Tauri, and
  async-runtime dependencies.
- **FR-018**: The in-memory live Agent Session reducer MUST remain the sole authority for current
  Agent Session presence and Activity. Archived sessions MUST NOT create Animals after restart.
- **FR-019**: The hook client MUST remain dependency-light, bounded, silent on every failure path,
  and unable to delay or fail Claude Code. It MUST NOT open the database or transcript itself.
- **FR-020**: Archival MUST be local-only and user-controlled. The integration surface MUST state
  that Claude transcripts can contain prompts, source code, tool output, and secrets and that
  Ranch stores a plaintext local copy while archival is enabled.
- **FR-021**: The database and its sidecar files MUST be created with user-only access on supported
  platforms wherever the platform exposes equivalent permissions.
- **FR-022**: Disabling archival MUST immediately reject new archive jobs while allowing previously
  accepted jobs to finish atomically. The integration surface MUST disclose that disabling stops
  new capture but does not cancel admitted work or delete existing history. Deletion and retention
  management are outside this feature.
- **FR-023**: The first release MUST archive only sessions whose Hook Firings reach a running Ranch
  after the user opts in. Historical backfill, cloud sync, Codex ingestion, general-purpose archive
  browsing, full text search, and archive-derived renderer behavior are out of scope.
- **FR-024**: Current executable tests, plugin wrapper paths, socket paths, client placement, Focus
  paths, Settings paths, README guidance, and current architecture documentation MUST use the new
  default data-root name in the root-migration slice. Historical ADRs/issues remain unchanged.
- **FR-025**: Every newly opened writer connection MUST enable and verify foreign keys. Every
  schema migration MUST include explicit transaction bounds because the selected Turso batch API
  does not add them, and an interrupted migration MUST be retryable without partial schema.
- **FR-026**: `ranch.db` MUST remain the feature's only database and MUST use ordinary relational
  Turso/SQLite tables, foreign keys, and indexes. A graph-native engine, separate graph sidecar,
  generic node/edge storage layer, and graph query API are outside this feature.
- **FR-027**: Schema version 1 MUST contain only Claude-specific archive entities and MUST NOT add
  speculative cross-tool session, event, artifact, or relationship tables. Shared relational
  entities require a later feature grounded in at least one additional integration.
- **FR-028**: Every build MUST include an explicit, session-only Developer Mode toggle in the app
  menu. Developer Mode MUST default to off on every launch and MUST NOT be persisted. The read-only
  Claude archive-health item MUST remain hidden until Developer Mode is enabled. Its overview is
  limited to schema version, database size, aggregate session/receipt/source/record counts,
  persisted last successful ingest time, and current-app-run dropped-job and content-free error-
  category counts that reset on restart and are labeled `this run`. It MUST expose no session
  identifiers, paths, raw payload, transcript content, or record-level rows and MUST remain
  independent from live rendering and ingestion success.
- **FR-029**: During rapid prototyping, the app-side archive protocol, installed Ranch plugin, hook
  client, and unshipped `ranch.db` archive schema MAY change in lockstep without backward-compatible
  readers, writers, migrations, or protocol negotiation. Installation/repair MUST validate and
  replace the complete plugin contract rather than support a mixed version. This exception MUST NOT
  weaken the `.adhd-ranch` migration or compatibility of existing Focus, Timer, and Settings data.

### Key Entities

- **Claude Archive Session**: Durable Claude-specific identity and metadata for one observed Claude
  Code run. It references Hook Receipts and one or more transcript source generations; it is not
  the live Agent Session shown by the renderer.
- **Claude Hook Receipt**: One immutable receipt of the exact JSON payload delivered by a Claude
  hook, together with arrival metadata and a digest. Duplicate deliveries remain separate facts.
- **Claude Transcript Source**: One physical transcript path and generation, including the cursor
  and fingerprint needed to resume incremental reads and detect replacement or truncation. It is
  labeled as a main or subagent source and carries the subagent id when available.
- **Claude Transcript Record**: One raw JSONL record from a transcript plus nullable lookup fields.
  Raw bytes are authoritative; extracted fields are disposable projections.
- **Archive Preference**: The user's explicit choice to permit or stop new local Claude archive
  writes. It is independent from whether archived data already exists.
- **Schema Migration**: One ordered, transactional database-version transition applied exactly
  once.

## Success Criteria

### Measurable Outcomes

- **SC-001**: A legacy-root migration fixture retains 100% of files and file contents and loads the
  same Focuses and Settings from the new root.
- **SC-002**: Replaying a representative Claude transcript archives 100% of its complete lines with
  byte-identical raw payloads, including main and subagent sources and at least one record type
  unknown to the typed projection.
- **SC-003**: Replaying the same Hook Firings and transcript three times, including once after
  reopening the database, creates zero duplicate transcript records.
- **SC-004**: A truncation/replacement fixture preserves both generations and never mutates the raw
  records archived from the first generation.
- **SC-005**: Archived sessions produce zero Agent Animals after restart until a new live Hook
  Firing establishes current presence.
- **SC-006**: Hook-client failure behavior and its existing 250 ms write bound remain unchanged
  when the app, database, or transcript is unavailable.
- **SC-007**: Runtime network instrumentation observes zero outbound connections while creating,
  migrating, writing, and querying `ranch.db`.
- **SC-008**: The repository quality gate and packaged-platform compile checks pass with the chosen
  database dependency and all generated artifacts current.
- **SC-009**: Database, WAL, and shared-memory artifacts have user-only permissions on supported
  platforms, and a network-denied/instrumented create-migrate-write-query run makes zero outbound
  connection attempts.
- **SC-010**: For synthetic fixtures, every aggregate value in the debug overview matches direct
  database/file observations, its empty/unavailable states render deterministically, and automated
  UI assertions find zero identifiers, paths, raw prompts, messages, tools, or source-code content.
  Packaged-build tests verify that the item is hidden before Developer Mode is enabled and opens
  afterward. Restart tests verify that Developer Mode returns to off, row counts and last successful
  ingest persist, and counters labeled `this run` reset to zero.
- **SC-011**: Saturating the archive queue increments the dropped-job count by the exact number of
  rejected jobs while the hook client's existing silent behavior and 250 ms write bound remain
  unchanged; a later accepted path hint can still synchronize transcript records.
- **SC-012**: A deliberately mismatched prototype plugin/app fixture is reported unhealthy and is
  corrected by the existing install/repair workflow; no compatibility adapter is invoked and the
  Claude hook still fails silently.
- **SC-013**: In a deterministic queued-job fixture, disabling archiving admits zero subsequent
  Hook Firings, completes every previously accepted job exactly once, and leaves no partial receipt
  or transcript batch.

## Assumptions

- "Use Turso" means an embedded, local, SQLite-compatible database file. Turso Cloud and sync are
  not requested and would conflict with the product's local-only runtime constraint.
- "All Claude data" means the exact Hook Firings plus complete main- and subagent-transcript
  records exposed by hooks for sessions Ranch observes while it is running after opt-in. It does
  not mean a census or backfill of every historical file under Claude's config directory.
- The user direction in this specification intentionally revisits ADR-0002's rejection of
  transcript ingestion. A new ADR must supersede only that rejected option while preserving the
  accepted live-state transport and renderer boundaries.
- Claude's transcript schema is private and changes over time. Raw-byte preservation is therefore
  the compatibility contract; extracted columns are best-effort indexes.
- Existing Focus Markdown and timer sidecars remain canonical user-owned state. `ranch.db` first
  owns only the Claude archive and its migrations.
- Relationship visualization and graph modeling may be reconsidered after additional harnesses or
  concrete cross-tool queries exist; this feature MUST NOT add speculative graph infrastructure.
- Stable raw payloads and source identities, rather than premature shared entities, are the
  compatibility path for future integrations.
- Hook Receipt completeness is explicitly best-effort under overload. Transcript synchronization
  remains retryable, but Ranch does not claim recovery of a rejected raw hook payload.
- The owner explicitly accepts lockstep breaking changes for the prototype archive protocol and
  unshipped archive schema. This does not authorize breaking existing user-owned Ranch data.

## Source Provenance

- **Present owner direction**: this request establishes the new `.adhd` root, local `ranch.db`,
  Claude structure mapping, and hook-fed persistence intent.
- **Accepted decisions retained**: ADR-0001 keeps Focus and Agent Session domains independent;
  ADR-0002 keeps hook delivery push-based, bounded, and independent from live transcript polling;
  ADR-0003 keeps renderer composition downstream of the harness-neutral Agent Session model.
- **Verified current observation**: current `main` reduces Claude payloads to session id, working
  directory, Pen, and Activity and stores live sessions only in memory.
- **External contract**: Claude's current hook documentation supplies `session_id`, `prompt_id`,
  `transcript_path`, `cwd`, and `hook_event_name`; `SubagentStop` additionally supplies
  `agent_id` and `agent_transcript_path`. The documentation warns that transcript writes are
  asynchronous.
