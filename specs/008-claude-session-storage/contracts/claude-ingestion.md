# Claude Archive Ingestion Contract

## Input boundary

The lockstep prototype plugin contract is exactly version `0.2.0`: the existing five lifecycle
registrations plus `SubagentStop` mapped to archive-only verb `observe`. Contract validation rejects
every other manifest version, event table, wrapper, or executable until the user explicitly runs
Install/Repair. There is no handshake, verb alias, or mixed-version adapter. The hook client's
bounded/silent behavior and wire frame remain:

```text
ASCII transport verb + newline + exact UTF-8 JSON payload bytes
```

The archive path consumes an owned copy after the full bounded frame has been read. It does not
change Claude's hook stdout, stderr, or exit status.

## Processing order

For one received frame:

1. Parse a Claude-specific transport action. Unknown verbs are silently ignored. `observe` has no
   harness-neutral `HookAction` projection.
2. If archival is on, attempt to admit an immutable job containing action, receipt time, and exact
   payload to the bounded worker queue. Successful admission is an "accepted archive firing." A
   full/closed queue rejects the archive job, increments a content-free dropped-job counter, and
   never blocks or fails Claude.
3. Independently, if Agents as Animals is on and the verb is a lifecycle action, apply it to the
   journal/`LiveSessions`. `observe` always bypasses this path. Neither preference gates the other.
4. The single archive worker parses only lookup fields. When `session_id` is non-empty, it upserts
   the Claude Archive Session before inserting the Hook Receipt in the same transaction so the
   immediate foreign key is satisfied. Missing/invalid session ids are stored as NULL.
5. Treat `transcript_path` as a main-transcript hint and `agent_transcript_path` as a subagent-
   transcript hint. Validate and schedule each present hint independently.
6. Synchronize newly completed records in bounded batches.
7. Commit transcript rows and the new cursor together. A failure rolls back that batch and leaves
   the previous cursor retryable.

The live path does not wait for transcript synchronization. Archive failure does not reverse a
live Agent Session update and never reaches Claude Code.

## Path validation

A transcript is readable only when all conditions hold:

- the payload value is an absolute path;
- the path exists and names a regular file at read time;
- both the candidate and configured Claude `projects/` directory are canonicalized;
- the candidate is a descendant of that canonical projects directory;
- the filename/session correlation is not contradictory to the Hook Firing;
- symlink resolution does not escape the projects directory.

The authoritative Claude configuration directory is resolved once at Ranch startup in this order:

1. non-empty `claude.config_dir` in `settings.yaml`;
2. non-empty `CLAUDE_CONFIG_DIR` inherited by the Ranch process;
3. `$HOME/.claude` (or the platform home equivalent).

The trusted projects root is `<resolved-config-dir>/projects`. A custom Claude environment that is
not inherited by Ranch must set `claude.config_dir`; a hook-provided transcript path never defines
or widens the trusted root.

Missing or lagging paths are retryable, not corrupt. Invalid/escaping paths are rejected and logged
without reading their content.

## Incremental JSONL rules

- `byte_cursor` points to the first byte not committed into a transcript record.
- Read from that offset and split on newline delimiters without loading the complete file.
- Split the delimiter (`\n` or `\r\n`) from JSON/content bytes without trimming either; persist the
  delimiter separately so concatenating `raw_json || record_delimiter` reconstructs source framing.
- A newline-terminated record is complete even when JSON parsing fails; store it with a parse error.
- A non-newline tail does not advance `byte_cursor` and is not persisted separately. The next sync
  rereads it from that cursor, so bytes are never both prepended and reread.
- Every valid main or subagent path hint schedules an immediate sync plus bounded delayed retries.
  `Stop`, `StopFailure`, `SessionEnd`, and `SubagentStop` use the terminal retry profile: retry until
  two consecutive `(length, mtime, boundary digest)` observations are unchanged, subject to the
  documented attempt/time cap. This lets asynchronous final rows land without polling forever.
- Stable observations end the retry schedule; they do not prove record framing is final. Every
  non-newline tail remains pending, even if it parses as a complete JSON value. A later newline
  commits that value and its delimiter as one record, never a separate empty record. The cursor
  stays at the tail start and diagnostics remain content-free.
- Insert transcript rows and advance cursor/ordinal in one transaction.
- `boundary_sha256` is SHA-256 over `[max(0, byte_cursor - 4096), byte_cursor)`. If the file shrinks
  below the cursor, the platform file identity changes, or that exact window no longer matches,
  mark the active generation superseded and create the replacement generation in the same
  transaction before reading from offset zero.

## Tolerant projection

Parse each complete raw record to `serde_json::Value` only after retaining the original bytes for
the transaction. Extract fields by path and type. Missing, unknown, or type-changed fields become
NULL plus an error category when appropriate; they do not reject the raw record.

No record-type allowlist controls persistence.

## Backpressure and lifecycle

- One worker owns the Turso connection and cursor state.
- The worker is one Tokio single-consumer task spawned by the Tauri composition root on the
  existing runtime; blocking transcript reads run through `spawn_blocking` and never occupy the
  socket thread.
- Job ordering follows socket receipt order.
- Transcript synchronization is coalesced by canonical path so a burst of hooks does not re-read
  the same range concurrently.
- Retry schedules survive coalescing and are bounded to a named attempt/time policy in code; tests
  use a paused/injected clock rather than sleeps.
- Disabling archival and checking the admission gate share one linearization point. Every firing
  after disable is rejected; every job admitted before it remains worker-owned and finishes exactly
  once. Re-enabling allows only new firings and performs no historical census.
- Graceful app shutdown closes admission, drains every accepted queued job and its transactions,
  then stops the worker. It does not make the hook client wait. Forced termination is best effort:
  committed transactions survive, and later syncs recover transcript rows; uncommitted receipts
  are not guaranteed.
- Queue saturation/closure before admission increments `dropped_jobs_this_run`. An accepted-job,
  database, or transcript failure increments only its bounded content-free error category. Neither
  blocks or fails Claude; a later transcript sync can recover transcript rows, while a firing
  rejected before queue admission has no receipt.
- After bootstrap and every successful archive transaction, the worker publishes an immutable
  aggregate `ArchiveHealthSnapshot`; UI reads never execute arbitrary SQL or alter job ordering.

## Diagnostics contract

Allowed diagnostic values:

- receipt id, session id, event name, canonical path;
- source id, generation, ordinal, byte offset, byte length;
- payload/record digest;
- error category and database migration version.

Forbidden diagnostic values:

- raw payloads or transcript lines;
- prompts, assistant text, tool inputs/results, command output, or source-file contents.

The internal diagnostics above are not the archive-health UI contract. That UI may receive only
the aggregate fields in [archive-health.md](archive-health.md), never receipt/session/source/record
identifiers, paths, offsets, or digests.
