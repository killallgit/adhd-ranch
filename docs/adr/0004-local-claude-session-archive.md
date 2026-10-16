# ADR-0004: Archive Claude session records locally without making history live state

**Status:** Accepted
**Date:** 2026-09-28
**Accepted:** 2026-09-28
**Deciders:** Ryan (project owner)

## Context

Ranch currently consumes five Claude Code lifecycle hooks, reduces each payload to the fields
needed for an in-memory `AgentSession`, and intentionally forgets both the payload and session when
the process ends. ADR-0002 accepted a stronger hook-state transport and rejected its Option C,
"Hooks become a doorbell; the transcript is the truth," because the renderer needed only live
Activity and reading a private transcript would expose prompts and tool output for no corresponding
product value.

The product direction has changed. Ranch now needs a durable, Claude-specific data foundation that
preserves the structures Claude writes for a session. The database is intended to support later
session features; it is not a reason to make historical rows decide whether an Animal is alive.
Claude's current hook contract already supplies `session_id`, `prompt_id`, `transcript_path`, `cwd`,
and `hook_event_name`; `SubagentStop` additionally supplies `agent_id` and
`agent_transcript_path`. It warns that transcript writes are asynchronous and may lag the hook.
Claude's transcript schema is private and changes between releases.

The same feature introduces the first Ranch database and renames the default application-data root
from `.adhd-ranch` to `.adhd`. Focus Markdown and timer sidecars remain canonical user-owned state;
only Claude archive data moves into the database in this decision.

This ADR supersedes ADR-0002 only where that ADR rejects transcript ingestion. ADR-0002's accepted
push transport, bounded silent hook client, and in-memory live-state reducer remain authoritative.

## Decision

Use one embedded, local Turso database at `<data-root>/ranch.db` and make raw Claude records the
archive's data-fidelity boundary. This describes schema-drift preservation, not a promise of
backward-compatible prototype wire or database schemas.

1. Ranch's default data root becomes `~/.adhd` on Unix and `adhd` under the Windows application-data
   directory. An explicit `ADHD_RANCH_HOME` remains unchanged. Startup atomically renames the legacy
   default root only when the new root does not exist; it never auto-merges two roots.
2. `ranch.db` uses `turso` 0.8.0-pre.14 in local-only mode with default and sync features disabled.
   The workspace Rust baseline rises to 1.90 to match the resolved dependency floor. Ordered
   migrations have explicit transaction bounds and every writer connection verifies foreign keys.
   A clean-shutdown/checkpoint plus SQLite reopen, integrity, and read test proves the engine escape
   path instead of assuming file compatibility alone makes rollback safe.
3. Every Claude Hook Firing successfully admitted to the bounded archive queue is stored as an
   immutable raw receipt with extracted common lookup fields and a digest. Duplicate deliveries
   remain separate receipts; rejected admissions increment a content-free dropped-job counter.
4. A Hook Firing is also a synchronization hint. Ranch incrementally reads newly completed records
   from validated `transcript_path` and `agent_transcript_path` values, stores each source line
   byte-for-byte, and extracts only a small stable index. `SubagentStop` uses an archive-only verb
   and never changes parent Activity. Unknown or malformed complete records are retained rather
   than discarded.
5. The archive records a cursor and file-generation fingerprint. Repeated reads are idempotent;
   truncation or replacement starts a new generation instead of mutating old records.
6. Archive I/O is a storage adapter behind the hook boundary. The dependency-light hook client does
   not open the database or transcript. A Ranch-owned Tokio single-consumer task serializes database
   writes and transcript reads so the socket listener and Claude process keep their existing bounded
   behavior. Every hint receives bounded delayed retries, with terminal hooks settling on two
   unchanged observations so asynchronous final rows do not require another hook.
7. The in-memory `LiveSessions` reducer remains the sole authority for present Agent Sessions and
   Activity. Startup does not reconstruct live Animals from `ranch.db`.
8. Archival is local-only, defaults off for users who have not explicitly opted in, and is disclosed
   as a plaintext copy of data that can include prompts, code, tool output, and secrets. Turning it
   off stops new writes without deleting existing rows.
9. Claude's trusted config root resolves from explicit Ranch settings, then the app-inherited
   `CLAUDE_CONFIG_DIR`, then the platform default. Hook paths cannot widen that root.
10. The first slice ingests only hook-observed sessions while Ranch is running. It does not crawl
   historical Claude projects, add browsing/search UI, sync to a cloud service, ingest Codex, or
   move Focus persistence into SQL.

## Options Considered

### Option A: Keep Claude state in memory only

| Dimension | Assessment |
|-----------|------------|
| Complexity | Low |
| Schema-drift risk | None |
| Supports durable Claude features | No |
| Privacy change | None |
| Reversibility | N/A |

**Pros:** preserves all current boundaries and stores no additional sensitive content.
**Cons:** cannot satisfy the requested durable Claude-data foundation and loses all evidence at app
exit.

### Option B: Make a typed transcript crate the canonical model

| Dimension | Assessment |
|-----------|------------|
| Complexity | Medium initially, high during drift |
| Schema-drift risk | High |
| Query ergonomics | High for known variants |
| Fidelity | Incomplete for unknown variants |
| Reversibility | Medium |

**Pros:** provides rich Rust types immediately. The `claude-code-transcripts` crate is broad and
includes a round-trip checker; `claude-session-parser` provides higher-level chat and usage views.
**Cons:** Claude's schema is not stable. The current `claude-code-transcripts` release maps unknown
entry variants to a payload-free `Unknown`, its known structs do not retain unknown fields, and a
current local transcript already contains a `cost-state` type absent from that release. A typed enum
therefore cannot be the only stored copy.

### Option C: Preserve raw records in Turso and derive narrow indexes (chosen)

| Dimension | Assessment |
|-----------|------------|
| Complexity | Medium |
| Schema-drift risk | Low at the storage boundary |
| Query ergonomics | Medium; indexed envelope fields now, projections later |
| Fidelity | Byte-preserving |
| Reversibility | High after the cross-engine reopen/integrity test passes |

**Pros:** unknown Claude data survives; known fields remain queryable; typed libraries may be added
or replaced without migrating the authoritative payload; live Agent behavior stays independent.
**Cons:** duplicates sensitive plaintext already held by Claude, needs cursor/generation logic, and
defers fully normalized queries until a consumer justifies them.

### Option D: Copy transcript files wholesale and scan or backfill `~/.claude/projects`

| Dimension | Assessment |
|-----------|------------|
| Complexity | Medium |
| Coverage | High, including sessions Ranch never observed |
| Coupling to Claude layout | High |
| Incremental query support | Low |
| Privacy blast radius | Highest |

**Pros:** simplest path to a complete historical corpus.
**Cons:** reads sessions without a Hook Firing or explicit per-feature need, couples Ranch to
Claude's project-directory naming, duplicates whole files before any consumer exists, and exceeds
the requested first slice.

## Trade-off Analysis

The durable boundary is the decisive choice. Claude's JSONL records are rich enough that a full
normalized schema would become a second implementation of Claude Code. Raw bytes plus stable
indexes preserve what arrived and delay semantic commitments until Ranch has a feature that needs
them. This is a deeper storage module than a thin repository: it owns migration, incremental file
tailing, generation detection, idempotency, path validation, and privacy-safe diagnostics.

Using the pre-1.0 `turso` Rust crate introduces dependency and platform risk. Local-only mode avoids
credentials and an HTTP/TLS runtime client, while a tested SQLite escape and an adapter-owned API
contain that risk. Current dependency resolution requires Rust 1.90, so the first slice updates the
workspace's stale 1.77 declaration and must pass all packaged-platform compile gates; otherwise the
decision returns for review rather than pinning transitive dependencies or silently switching
engines.

The privacy cost is real: Ranch creates another plaintext copy of conversations that may contain
secrets. Explicit opt-in, user-only file permissions, no network features, clear disclosure, and a
separate archive preference are requirements, not follow-up polish.

## Consequences

**Easier**

- Later Claude session features can query one stable local source rather than repeatedly parsing a
  private directory layout.
- New Claude record variants survive upgrades even before Ranch understands them.
- Transcript ingestion can be replayed and tested independently from the renderer.
- A future typed projection or full-text index can be rebuilt from raw authoritative records.

**Harder**

- Startup now owns a user-data directory migration and a database migration lifecycle.
- Hook processing gains a background worker, failure diagnostics, and transcript generation logic.
- Ranch duplicates sensitive plaintext and must treat archive permissions, disclosure, and future
  deletion/retention as product responsibilities.
- The Turso dependency is pre-1.0 and must be isolated behind the storage boundary.

**To revisit**

- User-controlled retention, deletion, and export.
- Historical backfill and recovery for Hook Firings missed while Ranch was stopped.
- Which typed Claude projections earn normalized tables or full-text indexes.
- Whether the same raw-record archive boundary generalizes to Codex without leaking Claude
  vocabulary into the harness-neutral Agent Session domain.

## Action Items

1. [x] Accept this ADR before implementation begins.
2. [ ] Raise the workspace Rust baseline to 1.90 and prove pinned local-only `turso` on all
       packaged targets.
3. [ ] Deliver the `.adhd` migration and empty `ranch.db` initialization as one rollback-safe slice.
4. [ ] Deliver raw Hook Receipt persistence and transcript cursoring with fixture proof.
5. [ ] Add explicit archive opt-in and plaintext-data disclosure without changing Agent display
       semantics.
6. [ ] Update README and current architecture/storage documentation in the data-root slice.
