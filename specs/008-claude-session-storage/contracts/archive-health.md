# Claude Archive Health Contract

## Purpose

Provide one aggregate-only testing surface for the Claude archive without creating an archive
browser, graph view, raw-row query API, or second database. The contract is read-only and cannot
enable, disable, repair, delete, or otherwise mutate ingestion.

## Menu and window lifecycle

- Every build contains an unchecked `Developer Mode` item in the app's Window menu.
- On launch, Developer Mode is off and `Claude Archive Health…` is absent from the menu.
- Enabling Developer Mode appends the health item for this process only.
- Disabling Developer Mode removes the item and closes the health window if it is open.
- Repeated enable/disable operations are idempotent and do not create duplicate items or windows.
- Developer Mode is never written to Settings and is off after every restart.
- Opening the item focuses the existing health window or creates one separate from the overlay and
  the existing Agent Hooks debug window.
- Both the open handler and Tauri command verify current Developer Mode state; a stale/forged window
  cannot read the projection after the mode is disabled.

## Read projection

The generated Rust/TypeScript projection contains exactly:

| Field | Shape | Meaning |
|-------|-------|---------|
| `availability` | `ready`, `empty`, or `unavailable` | Database-read state; empty means all four counts are zero |
| `schema_version` | integer or NULL | `PRAGMA user_version`; NULL only when unavailable |
| `database_bytes` | non-negative integer | Sum of existing main DB, WAL, and SHM metadata lengths |
| `session_count` | non-negative integer or NULL | Aggregate `claude_archive_sessions` rows |
| `receipt_count` | non-negative integer or NULL | Aggregate `claude_hook_receipts` rows |
| `source_count` | non-negative integer or NULL | Aggregate `claude_transcript_sources` rows |
| `record_count` | non-negative integer or NULL | Aggregate `claude_transcript_records` rows |
| `last_successful_ingest_at` | RFC3339 string or NULL | Latest committed receipt or transcript-batch time |
| `dropped_jobs_this_run` | non-negative integer | Queue admissions rejected since process start |
| `errors_this_run` | ordered category/count rows | Operational errors since process start |

The worker initializes persisted aggregates transaction-consistently when it opens the database and
publishes a new immutable snapshot only after successful commits. `last_successful_ingest_at` is
derived from the maximum committed receipt `received_at` and transcript source `last_synced_at`.
The byte count is refreshed from filesystem metadata for `ranch.db` and existing `ranch.db-wal` and
`ranch.db-shm` files; paths themselves never cross IPC.

Allowed error categories are a closed enum:

- `database_unavailable`
- `database_write_failed`
- `transcript_path_rejected`
- `transcript_read_failed`
- `transcript_retry_exhausted`
- `worker_closed`

Queue-full/closed rejection increments `dropped_jobs_this_run`; it is not also counted as an error.
Persisted record `parse_error` values are not runtime errors. Merely opening or polling the window
must not increment any counter.

## Presentation states

- `ready`: show the schema, file footprint, four counts, last successful ingest, and `this run`
  counters.
- `empty`: show the same zero aggregates plus an explicit “no archived data” state.
- `unavailable`: show a content-free database-unavailable state and any current-run counters; never
  turn the failure into zero persisted counts.
- Loading remains distinct from all three returned states.

## Forbidden projection

The command type and rendered view contain none of the following:

- session, receipt, source, record, prompt, message, or agent identifiers;
- filesystem paths, offsets, ordinals, hashes, or raw rows;
- prompts, assistant messages, tool names/inputs/results, command output, or source content;
- database handles, arbitrary query strings, or mutation commands.

Tests inspect both the generated type shape and rendered DOM so adding a forbidden field fails the
contract rather than relying on CSS to hide it.
